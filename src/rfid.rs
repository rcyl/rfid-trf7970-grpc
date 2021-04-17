use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tonic::{Request, Response, Status, Streaming};
use super::include::read_info_server::ReadInfo;
use super::include:: {
    ClientActions, StreamPayload, SingleBlockRequest, DataPayload,
    UuidPayload, Empty
};

use futures::lock::{Mutex, MutexGuard};
use super::reader::ReaderTraits;

type Result<T> = std::result::Result<T, Status>;

pub struct RFID {
    reader: Mutex<Box <dyn ReaderTraits>>
}

impl RFID {
    pub fn new(reader: Box<dyn ReaderTraits>) -> RFID {
        RFID {
            reader: Mutex::new(reader)
        }
    }
}

#[tonic::async_trait]
impl ReadInfo for RFID {

    type ReadUUIDContinousStream = mpsc::Receiver<Result<UuidPayload>>;
    type ReadBlockContinousStream = mpsc::Receiver<Result<DataPayload>>;

    async fn read_uuid(&self, _request: Request<Empty>) 
        -> Result<Response<UuidPayload>> {
        
        Ok(Response::new(UuidPayload { uuid: String::from("123")}))
    }

    async fn read_single_block(&self, request: Request<SingleBlockRequest>) 
        -> Result<Response<DataPayload>> {
        
        Ok(Response::new(DataPayload { 
            uuid: String::from("123"),
            data: String::from("456")
        }))
    }

    async fn read_uuid_continous(&self, request: Request<Streaming<StreamPayload>>)
        -> Result<Response<Self::ReadUUIDContinousStream>> {
        
        Err(Status::invalid_argument("123"))
    }

    async fn read_block_continous(&self, request: Request<Streaming<StreamPayload>>)
        -> Result<Response<Self::ReadBlockContinousStream>> {
    
        Err(Status::invalid_argument("123"))
    }
}
