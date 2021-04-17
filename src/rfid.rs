use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tonic::{Request, Response, Status, Streaming};
use super::include::read_info_server::ReadInfo;
use super::include:: {
    ClientActions, StreamPayload, SingleBlockRequest, Payload, Empty
};
use std::sync::Arc;
use std::time::Duration;

use futures::lock::{Mutex, MutexGuard};
use super::reader::ReaderTraits;

const MPSC_BUFFER_SIZE: usize = 0xFFFF;

macro_rules! get_reader {
    ($var:ident) => {
        {   
            match $var.reader.try_lock() { 
                Some(ret) => ret, 
                None => return Err(Status::internal("Unable to obtain reader"))
            }
        }
    };
}

macro_rules! get_reader_async {
    ($var:ident) => {
        {   
            match $var.try_lock() { 
                Some(ret) => ret, 
                None => return Err(Status::internal("Unable to obtain reader"))
            }
        }
    };
}

type Result<T> = std::result::Result<T, Status>;

pub struct RFID {
    reader: Arc<Mutex<Box <dyn ReaderTraits>>>
}

impl RFID {
    pub fn new(reader: Box<dyn ReaderTraits>) -> RFID {
        RFID {
            reader: Arc::new(Mutex::new(reader))
        }
    }
}

//wait for message from client within certain timeout
pub async fn get_client_message(request: &mut Streaming<StreamPayload>, timeout_ms: u64)
    -> Result<StreamPayload> {

    let delayer = async {
        match request.message().await {
            Ok(payload) => {
                match (payload) {
                    Some(msg) => { Ok(msg) },
                    None => {
                        Err(Status::invalid_argument("No message from client"))
                    }
                }
            },
            Err(e) => { Err(e) }
        }
    };

    let timeout = Duration::from_millis(timeout_ms);
    match tokio::time::timeout(timeout, delayer).await {
        Ok(message) => {
            match message {
                Ok(ret) => { return Ok(ret) },
                Err(e) => { return Err(e) },
            }
        },
        Err(_) => {
            return Err(Status::deadline_exceeded("No message received within deadline"))
        }
    };
}

#[tonic::async_trait]
impl ReadInfo for RFID {

    type ReadUUIDContinousStream = mpsc::Receiver<Result<Payload>>;
    type ReadBlockContinousStream = mpsc::Receiver<Result<Payload>>;

    async fn read_uuid(&self, _request: Request<Empty>) 
        -> Result<Response<Payload>> {
        
        let mut reader = get_reader!(self);
        
        match reader.read_uuid() {
            Ok(uuid) => {
                return Ok(Response::new(Payload { info : uuid }))
            },
            Err(e) => {
                return Err(Status::internal(e.to_string()))
            }
        }
    }

    async fn read_single_block(&self, request: Request<SingleBlockRequest>) 
        -> Result<Response<Payload>> {
        
        let mut reader = get_reader!(self);

        match reader.read_single_block(request.get_ref().block_index) {
            Ok(data) => {
                return Ok(Response::new(Payload { info: data }))
            },
            Err(e) => {
                return Err(Status::internal(e.to_string()))
            }
        }
    }

    //bi-directional stream, wait for user to ack for after every read
    async fn read_uuid_continous(&self, mut request: Request<Streaming<StreamPayload>>)
        -> Result<Response<Self::ReadUUIDContinousStream>> {
        
        let (mut tx, rx) : 
            (Sender<Result<Payload>>, Receiver<Result<Payload>>) 
                = mpsc::channel(MPSC_BUFFER_SIZE);
        
        let reader_arc = self.reader.clone();
        tokio::spawn(async move {

            loop {
                /* wait for ack prior to starting read */
                match get_client_message(request.get_mut(), 1000).await {
                    Ok(message) => {

                        match message.action {
                            act if act == ClientActions::Cancel as i32 => {
                                let e = Status::cancelled("Cancelled by user");
                                log::error!("{}", e.to_string());
                                return Err(e)
                            },
                            act if act == ClientActions::Unknown as i32 => {
                                let e = Status::invalid_argument("Unknown user action");
                                log::error!("{}", e.to_string());
                                return Err(e)
                            },
                            _ => { }
                        }
                    },
                    Err(e) => {
                        if let Err(err) = tx.send(Err(e)).await {
                            log::error!("{}", err.to_string());
                        }
                        //TODO: figure out a way to copy and return the error
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
                    },
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

    async fn read_block_continous(&self, request: Request<Streaming<StreamPayload>>)
        -> Result<Response<Self::ReadBlockContinousStream>> {
    
        Err(Status::invalid_argument("123"))
    }
}

#[cfg(test)]
mod test {

    use mockall::{mock, predicate::eq, Sequence};
    use crate::reader::MockReaderTraits;
    use crate::reader::err::ReaderError;
    use crate::serial::err::SerialError;
    use crate::scaffold::scaffold::*;
    use serial_test::*;
    use super::*;

    #[tokio::test]
    #[serial]
    async fn read_uuid_serial_error() {
        
        let mut reader = MockReaderTraits::new();
        reader.expect_read_uuid().returning(|| {
            Err(ReaderError::SerialError(SerialError::NoReplyAfterMultipleTries))
        });

        let rfid = RFID::new(Box::new(reader));
        
        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let mut res = client.read_uuid(Request::new(Empty{})).await;
        ts.end().await;

        match res {
            Ok(_) => { panic!("Should have been an error") }, 
            Err(e) => { assert!(e.message().contains("multiple tries"))}
        }       
    }

    #[tokio::test]
    #[serial]
    async fn read_uuid_ok() {
        
        let mut reader = MockReaderTraits::new();
        reader.expect_read_uuid().returning(|| {
            Ok(String::from("CAFEDEADBEEFB0B0"))
        });

        let rfid = RFID::new(Box::new(reader));
        
        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let mut res = client.read_uuid(Request::new(Empty{})).await;
        ts.end().await;

        assert!(!res.is_err());
        assert_eq!(res.unwrap().get_ref().info, "CAFEDEADBEEFB0B0");
    }

    #[tokio::test]
    #[serial]
    async fn read_single_block_serial_error() {
        
        let block_idx: u32 = 255;
        let mut reader = MockReaderTraits::new();
        
        reader.expect_read_single_block().with(eq(block_idx)).returning(|_| {
            Err(ReaderError::SerialError(SerialError::NoReplyAfterMultipleTries))
        });

        let rfid = RFID::new(Box::new(reader));
        
        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let mut res = client.read_single_block(
            Request::new(SingleBlockRequest{
                block_index: block_idx
            })).await;
        
        ts.end().await;

        match res {
            Ok(_) => { panic!("Should have been an error") }, 
            Err(e) => { assert!(e.message().contains("multiple tries"))}
        }       
    }

    #[tokio::test]
    #[serial]
    async fn read_single_block_ok() {
        
        let block_idx: u32 = 255;
        let mut reader = MockReaderTraits::new();
        
        reader.expect_read_single_block().with(eq(block_idx)).returning(|_| {
            Ok(String::from("12345678"))
        });

        let rfid = RFID::new(Box::new(reader));
        
        let ts = TestStruct::new(rfid).await;
        let mut client = start_client().await;

        let mut res = client.read_single_block(
            Request::new(SingleBlockRequest{
                block_index: block_idx
            })).await;
        
        ts.end().await;

        assert!(!res.is_err());
        assert_eq!(res.unwrap().get_ref().info, "12345678");
    }

}