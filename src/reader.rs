
use std::fmt;
use std::fmt::Debug;

use crate::serial::err::SerialError;
use crate::serial::RFIDSerialTraits;

#[derive(Debug)]
pub enum ReaderError<> {
    SerialError(SerialError),
    //returns the whatever is read when no matching targets
    NoMatchingTargets(String)
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReaderError::SerialError(ref e) =>
                std::fmt::Display::fmt(&e, f),
            ReaderError::NoMatchingTargets(ref e) =>
                std::fmt::Display::fmt(&e, f),
        }
    }
}

impl From<SerialError> for ReaderError {
    fn from(err: SerialError) -> ReaderError {
        ReaderError::SerialError(err)
    }
}

/* for mocking of reader functions */
#[cfg_attr(test, automock)]
pub trait ReaderTraits: Send + Sync {
    fn initialize(&self) -> Result<(), ReaderError>;
    fn read_uuid(&self) -> Result<String, ReaderError>; 
    //returns a single block of data of information
    fn read_single_block(&self, block_idx: u32) 
        -> Result<String, ReaderError>;
    // returns num chars of data
    fn read_multiple_block(&self, num_chars: u32, block_size: u32)
        -> Result<String, ReaderError>;
}

pub struct Reader {
    serial: Box<dyn RFIDSerialTraits>,
    read_timeout_ms: u64,
}

impl ReaderTraits for Reader {

    fn initialize(&self) -> Result<(), ReaderError> {
        //Ok(())
        //Err(ReaderError::SerialError(SerialError::NoSerialPortsFound))
        Err(ReaderError::NoMatchingTargets("123".to_string()))
    }

    fn read_uuid(&self) -> Result<String, ReaderError> {
        Err(ReaderError::SerialError(SerialError::NoSerialPortsFound))
    }

    fn read_single_block(&self, block_idx: u32) 
        -> Result<String, ReaderError> {
        Err(ReaderError::SerialError(SerialError::NoSerialPortsFound))
    }

    fn read_multiple_block(&self, num_chars: u32, block_size: u32) 
        -> Result<String, ReaderError> {
        Err(ReaderError::SerialError(SerialError::NoSerialPortsFound))
    }
}

impl Reader {

    pub fn new(serial: Box<dyn RFIDSerialTraits>, read_timeout_ms: u64) 
        -> Reader {
        Reader {
            serial: serial,
            read_timeout_ms: read_timeout_ms
        }
    }

    fn send_read_regex(&self, send: &str, regex: &Vec<&str>) {

    }

    fn initialize(&self) -> Result<(), ReaderError> {
        self.set_iso()?;
        self.set_half_data()?;
        self.set_agc()?;
        self.set_am()?;
        self.set_antenna()?;
        Ok(())
    }

    fn set_iso(&self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_half_data(&self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_agc(&self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_am(&self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_antenna(&self) -> Result<(), ReaderError> {
        Ok(())
    }
}