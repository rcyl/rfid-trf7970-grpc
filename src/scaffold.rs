#[cfg(test)]
pub mod scaffold {

    use crate::include::read_info_client::ReadInfoClient;
    use crate::include::read_info_server::ReadInfoServer;
    use crate::rfid::Rfid;
    use futures_util::FutureExt;
    use std::time::Duration;
    use tokio::sync::oneshot;
    use tokio::task::JoinHandle;
    use tonic::transport::Server;

    const IP_ADDR: &'static str = "http://[::]:50051";

    pub struct TestStruct {
        tx: oneshot::Sender<()>,
        jh: JoinHandle<()>,
    }

    impl TestStruct {
        pub async fn new(rfid: Rfid) -> TestStruct {
            let (tx, rx) = oneshot::channel::<()>();
            let jh = start_server(rfid, rx).await;

            TestStruct { tx: tx, jh: jh }
        }

        pub async fn end(self) {
            self.tx.send(()).unwrap();
            self.jh.await.unwrap();
        }
    }

    /* the one shot channel is just used to terminate the server after
    one call by dropping rx*/
    pub async fn start_server(rfid: Rfid, rx: oneshot::Receiver<()>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let addr = "[::]:50051".parse().unwrap();
            Server::builder()
                .add_service(ReadInfoServer::new(rfid))
                .serve_with_shutdown(addr, rx.map(drop))
                .await
                .unwrap();
        })
    }

    pub async fn start_client() -> ReadInfoClient<tonic::transport::Channel> {
        tokio::time::delay_for(Duration::from_millis(100)).await;
        ReadInfoClient::connect(IP_ADDR).await.unwrap()
    }
}
