
use std::fmt;
use std::fmt::Debug;
use regex::Regex;

use crate::serial::err::SerialError;
use crate::serial::RFIDSerialTraits;
use crate::utils::StopWatch;

mod err;

use err::ReaderError;

/* for mocking of reader functions */
#[cfg_attr(test, automock)]
pub trait ReaderTraits: Send + Sync {
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
        
        let mut reader = Reader {
            serial: serial,
            read_timeout_ms: read_timeout_ms
        };

        if let Err(e) = reader.initialize() {
            log::error!("{}", e);
            panic!(e.to_string());
        }

        reader
    }

    //send a command, and check whether output matches any of the regex
    fn send_read_regex(&mut self, cmd: &str, regex: &Vec<&str>) 
        -> Result<(), ReaderError> {
        
        let read = self.serial.send_recv(cmd)?;
        
        for r in regex.iter() {
            //panic if any regex is invalid
            let re = match Regex::new(r) {
                Ok(re) => { re },
                Err(e) => {
                    log::error!("{}", ReaderError::InvalidRegex(r.to_string()));
                    panic!(ReaderError::InvalidRegex(r.to_string()).to_string());
                }
            };
            if let Some(_) = re.captures(&read) {
                return Ok(())
            }
        }

        return Err(ReaderError::NoMatchingTargets(read))
    }

    fn initialize(&mut self) -> Result<(), ReaderError> {
        self.set_iso()?;
        self.set_half_data()?;
        self.set_agc()?;
        self.set_am()?;
        self.set_antenna()?;
        Ok(())
    }

    fn set_iso(&mut self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_half_data(&mut self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_agc(&mut self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_am(&mut self) -> Result<(), ReaderError> {
        Ok(())
    }

    fn set_antenna(&mut self) -> Result<(), ReaderError> {
        Ok(())
    }
}