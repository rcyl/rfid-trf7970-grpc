#[cfg(test)]
use mockall::automock;

use serialport::{SerialPortInfo, SerialPortSettings};
use serialport::prelude::*;
use std::io::{Write, BufRead, BufReader};

/* for mocking of open serial functions */
#[cfg_attr(test, automock)]
pub trait SerialCrateTraits: Send + Sync {
    fn get_ports(&self) -> Result<Vec<SerialPortInfo>, serialport::Error>;
    fn open(&self, path: &str, settings: &SerialPortSettings) 
        -> Result<Box<dyn SerialPort>, serialport::Error>;
    fn send(&self, msg: &str, device: &mut Box <dyn SerialPort>) 
        -> Result<String, std::io::Error>;
    fn read(&self, device: &mut Box <dyn SerialPort>) 
        -> Result<String, std::io::Error>;
}

pub struct SerialCrate {}

impl SerialCrate {
    pub fn new() -> SerialCrate {
        SerialCrate {}
    }
}

impl dyn SerialCrateTraits {
    pub fn new() -> SerialCrate {
        SerialCrate {}
    }
}



impl SerialCrateTraits for SerialCrate {
    
    fn get_ports(&self) -> Result<Vec<SerialPortInfo>, serialport::Error> {
        return serialport::available_ports();
    }

    fn open(&self, path: &str, settings: &SerialPortSettings) 
        -> Result<Box<dyn SerialPort>, serialport::Error> {
        return serialport::open_with_settings(path, settings);
    }

    fn send(&self, msg: &str, device: &mut Box <dyn SerialPort>) 
        -> Result<String, std::io::Error> {

        let cmd = msg.as_bytes();
        device.write_all(cmd)?;
        return self.read(device);
    }

    fn read(&self, device: &mut Box <dyn SerialPort>) 
        -> Result<String, std::io::Error> {

        let mut reader = BufReader::new(device);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }
}

