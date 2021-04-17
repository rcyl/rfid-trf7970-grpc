use super::include::read_info_server::ReadInfo;
use super::include::{ClientActions, Empty, Payload, SingleBlockRequest, StreamPayload};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tonic::{Request, Response, Status, Streaming};

use super::reader::ReaderTraits;
use futures::lock::Mutex;

const MPSC_BUFFER_SIZE: usize = 0xFFFF;

macro_rules! get_reader {
    ($var:ident) => {{
        match $var.reader.try_lock() {
            Some(ret) => ret,
            None => return Err(Status::internal("Unable to obtain reader")),
        }
    }};
}

macro_rules! get_reader_async {
    ($var:ident) => {{
        match $var.try_lock() {
            Some(ret) => ret,
            None => return Err(Status::internal("Unable to obtain reader")),
        }
    }};
}

type Result<T> = std::result::Result<T, Status>;

pub struct Rfid {
    reader: Arc<Mutex<Box<dyn ReaderTraits>>>,
}

impl Rfid {
    pub fn new(reader: Box<dyn ReaderTraits>) -> Rfid {
        Rfid {
            reader: Arc::new(Mutex::new(reader)),
        }
    }
}

//wait for message from client within certain timeout
pub async fn get_client_message(
    request: &mut Streaming<StreamPayload>,
    timeout_ms: u64,
) -> Result<StreamPayload> {
    let delayer = async {
        match request.message().await {
            Ok(payload) => match payload {
                Some(msg) => Ok(msg),
                None => Err(Status::invalid_argument("No message from client")),
            },
            Err(e) => Err(e),
        }
    };

    let timeout = Duration::from_millis(timeout_ms);
    match tokio::time::timeout(timeout, delayer).await {
        Ok(message) => match message {
            Ok(ret) => return Ok(ret),
            Err(e) => return Err(e),
        },
        Err(_) => {
            return Err(Status::deadline_exceeded(
                "No message received within deadline",
            ))
        }
    };
}

#[tonic::async_trait]
impl ReadInfo for Rfid {
    type ReadUuidContinousStream = mpsc::Receiver<Result<Payload>>;
    type ReadBlockContinousStream = mpsc::Receiver<Result<Payload>>;

    async fn read_uuid(&self, _request: Request<Empty>) -> Result<Response<Payload>> {
        let mut reader = get_reader!(self);

        match reader.read_uuid() {
            Ok(uuid) => return Ok(Response::new(Payload { info: uuid })),
            Err(e) => return Err(Status::internal(e.to_string())),
        }
    }

    async fn read_single_block(
        &self,
        request: Request<SingleBlockRequest>,
    ) -> Result<Response<Payload>> {
        let mut reader = get_reader!(self);

        match reader.read_single_block(request.get_ref().block_index) {
            Ok(data) => return Ok(Response::new(Payload { info: data })),
            Err(e) => return Err(Status::internal(e.to_string())),
        }
    }

    //bi-directional stream, wait for user to ack for after every read
    async fn read_uuid_continous(
        &self,
        mut request: Request<Streaming<StreamPayload>>,
    ) -> Result<Response<Self::ReadUuidContinousStream>> {
        let (mut tx, rx): (Sender<Result<Payload>>, Receiver<Result<Payload>>) =
            mpsc::channel(MPSC_BUFFER_SIZE);

        let reader_arc = self.reader.clone();
        tokio::spawn(async move {
            loop {
                /* wait for ack prior to starting read */
                match get_client_message(request.get_mut(), 1000).await {
                    Ok(message) => match message.action {
                        act if act == ClientActions::Cancel as i32 => {
                            let e = Status::cancelled("Cancelled by user");
                            log::error!("{}", e.to_string());
                            if let Err(send_err) = tx.send(Err(e)).await {
                                log::error!("{}", send_err.to_string());
                            }
                            break;
                        }
                        act if act == ClientActions::Unknown as i32 => {
                            let e = Status::invalid_argument("Unknown user action");
                            log::error!("{}", e.to_string());
                            if let Err(send_err) = tx.send(Err(e)).await {
                                log::error!("{}", send_err.to_string());
                            }
                            break;
                        }
                        _ => {}
                    },
                    Err(e) => {
                        if let Err(err) = tx.send(Err(e)).await {
                            log::error!("{}", err.to_string());
                        }
                        break;
                    }
                }

                let mut reader = get_reader_async!(reader_arc);
                match reader.read_uuid() {
                    Ok(uuid) => {
                        if let Err(e) = tx.send(Ok(Payload { info: uuid })).await {
                            log::error!("{}", e.to_string());
                            return Err(Status::internal(e.to_string()));
                        }
                    }
                    Err(e) => {
                        if let Err(e) = tx.send(Err(Status::internal(e.to_string()))).await {
                            log::error!("{}", e.to_string());
                        }
                        return Err(Status::internal(e.to_string()));
                    }
                }
            }
            Ok(())
        });

        Ok(Response::new(rx))
    }

    async fn read_block_continous(
        &self,
        _request: Request<Streaming<StreamPayload>>,
    ) -> Result<Response<Self::ReadBlockContinousStream>> {
        //TODO: will be similar to the read uuid continous version
        Err(Status::invalid_argument("123"))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::reader::err::ReaderError;
    use crate::reader::MockReaderTraits;
    use crate::scaffold::scaffold::*;
    use crate::serial::err::SerialError;
    use futures::stream;
    use mockall::{predicate::eq, Sequence};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use serial_test::*;

    #[tokio::test]
    #[serial]
    async fn read_uuid_serial_error() {
        let mut reader = MockReaderTraits::new();
        reader.expect_read_uuid().returning(|| {
            Err(ReaderError::SerialError(
                SerialError::NoReplyAfterMultipleTries,
            ))
        });

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let res = client.read_uuid(Request::new(Empty {})).await;
        ts.end().await;

        match res {
            Ok(_) => panic!("{}", "Should have been an error"),
            Err(e) => assert!(e.message().contains("multiple tries")),
        }
    }

    #[tokio::test]
    #[serial]
    async fn read_uuid_ok() {
        let mut reader = MockReaderTraits::new();
        reader
            .expect_read_uuid()
            .returning(|| Ok(String::from("CAFEDEADBEEFB0B0")));

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let res = client.read_uuid(Request::new(Empty {})).await;
        ts.end().await;

        assert!(!res.is_err());
        assert_eq!(res.unwrap().get_ref().info, "CAFEDEADBEEFB0B0");
    }

    #[tokio::test]
    #[serial]
    async fn read_single_block_serial_error() {
        let block_idx: u32 = 255;
        let mut reader = MockReaderTraits::new();

        reader
            .expect_read_single_block()
            .with(eq(block_idx))
            .returning(|_| {
                Err(ReaderError::SerialError(
                    SerialError::NoReplyAfterMultipleTries,
                ))
            });

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let res = client
            .read_single_block(Request::new(SingleBlockRequest {
                block_index: block_idx,
            }))
            .await;

        ts.end().await;

        match res {
            Ok(_) => panic!("{}", "Should have been an error"),
            Err(e) => assert!(e.message().contains("multiple tries")),
        }
    }

    #[tokio::test]
    #[serial]
    async fn read_single_block_ok() {
        let block_idx: u32 = 255;
        let mut reader = MockReaderTraits::new();

        reader
            .expect_read_single_block()
            .with(eq(block_idx))
            .returning(|_| Ok(String::from("12345678")));

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let res = client
            .read_single_block(Request::new(SingleBlockRequest {
                block_index: block_idx,
            }))
            .await;

        ts.end().await;

        assert!(!res.is_err());
        assert_eq!(res.unwrap().get_ref().info, "12345678");
    }

    /* tests 1000 calls with correct acks*/
    #[tokio::test]
    #[serial]
    async fn read_uuid_continuous_ok() {
        let mut reader = MockReaderTraits::new();
        let mut v: Vec<String> = Vec::new();
        let mut seq = Sequence::new();
        let n = 1000;

        for _ in 0..n {
            let rstr: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            v.push(rstr.clone());
            reader
                .expect_read_uuid()
                .times(1)
                .in_sequence(&mut seq)
                .returning(move || Ok(rstr.clone()));
        }

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let mut requests: Vec<StreamPayload> = Vec::new();
        for _ in 0..n {
            let sp = StreamPayload {
                action: ClientActions::Ack as i32,
                request: 0,
            };
            requests.push(sp)
        }
        let stream = stream::iter(requests);

        let mut res = client
            .read_uuid_continous(Request::new(stream))
            .await
            .unwrap()
            .into_inner();
        let mut payloads: Vec<Payload> = Vec::new();
        ts.end().await;

        loop {
            match res.message().await {
                Ok(val) => {
                    if let Some(payload) = val {
                        payloads.push(payload);
                    }
                    continue;
                }
                Err(_) => {
                    break;
                }
            }
        }

        let infos: Vec<String> = payloads.into_iter().map(|p| p.info).collect();
        assert_eq!(infos, v);
    }

    #[tokio::test]
    #[serial]
    async fn read_uuid_unknown_action_at_start() {
        let reader = MockReaderTraits::new();

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let sp = StreamPayload {
            action: ClientActions::Unknown as i32,
            request: 0,
        };
        let stream = stream::iter(vec![sp]);

        let mut res = client
            .read_uuid_continous(Request::new(stream))
            .await
            .unwrap()
            .into_inner();

        ts.end().await;

        loop {
            match res.message().await {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    assert_eq!(e.code(), tonic::Code::InvalidArgument);
                    assert!(e.message().contains("Unknown user action"));
                    break;
                }
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn read_uuid_cancelled_at_start() {
        let reader = MockReaderTraits::new();
        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let sp = StreamPayload {
            action: ClientActions::Cancel as i32,
            request: 0,
        };
        let stream = stream::iter(vec![sp]);

        let mut res = client
            .read_uuid_continous(Request::new(stream))
            .await
            .unwrap()
            .into_inner();

        ts.end().await;

        loop {
            match res.message().await {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    assert_eq!(e.code(), tonic::Code::Cancelled);
                    assert!(e.message().contains("Cancelled by user"));
                    break;
                }
            }
        }
    }

    /* tests 1000 calls with correct acks with cancellation request at the end*/
    #[tokio::test]
    #[serial]
    async fn read_uuid_n_packets_cancel_end() {
        let mut reader = MockReaderTraits::new();
        let mut v: Vec<String> = Vec::new();
        let mut seq = Sequence::new();
        let n = 1000;

        for _ in 0..n {
            let rstr: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            v.push(rstr.clone());
            reader
                .expect_read_uuid()
                .times(1)
                .in_sequence(&mut seq)
                .returning(move || Ok(rstr.clone()));
        }

        let rfid = Rfid::new(Box::new(reader));

        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let mut requests: Vec<StreamPayload> = Vec::new();
        for _ in 0..n {
            let sp = StreamPayload {
                action: ClientActions::Ack as i32,
                request: 0,
            };
            requests.push(sp)
        }

        /* last cancellation package */
        requests.push(StreamPayload {
            action: ClientActions::Cancel as i32,
            request: 0,
        });

        let stream = stream::iter(requests);

        let mut res = client
            .read_uuid_continous(Request::new(stream))
            .await
            .unwrap()
            .into_inner();
        let mut payloads: Vec<Payload> = Vec::new();
        ts.end().await;

        loop {
            match res.message().await {
                Ok(val) => {
                    if let Some(payload) = val {
                        payloads.push(payload);
                    }
                    continue;
                }
                Err(e) => {
                    assert_eq!(e.code(), tonic::Code::Cancelled);
                    assert!(e.message().contains("Cancelled by user"));
                    break;
                }
            }
        }

        let infos: Vec<String> = payloads.into_iter().map(|p| p.info).collect();
        assert_eq!(infos, v);
    }
}
