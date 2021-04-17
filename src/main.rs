mod include;
mod reader;
mod rfid;
mod scaffold;
mod serial;

use include::read_info_server::ReadInfoServer;
use tonic::transport::Server;

use reader::Reader;
use rfid::RFID;
use serial::low::SerialCrate;
use serial::RFIDSerial;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse().unwrap();
    let sc = SerialCrate::new();

    let serial = RFIDSerial::new(Box::new(sc));
    let reader = Reader::new(Box::new(serial));
    let rfid = RFID::new(Box::new(reader));

    Server::builder()
        .add_service(ReadInfoServer::new(rfid))
        .serve(addr)
        .await?;

    Ok(())
}
