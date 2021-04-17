
mod serial;
mod reader;
mod utils;
mod rfid;
mod include;

use tonic::transport::Server;
use include::read_info_server::ReadInfoServer;

use serial::low::SerialCrate;
use serial::RFIDSerial;
use reader::Reader;
use rfid::RFID;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    let addr = "[::]:50051".parse().unwrap();
    let sc = SerialCrate::new();
    
    let serial = RFIDSerial::new(Box::new(sc));
    let reader = Reader::new(Box::new(serial));
    let rfid = RFID::new(Box::new(reader));

    Server::builder().add_service(ReadInfoServer::new(rfid)).serve(addr).await?;

    Ok(())
}