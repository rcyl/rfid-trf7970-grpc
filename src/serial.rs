#[cfg(test)]
use mockall::automock;

use serialport::{SerialPortInfo, SerialPortSettings};
use serialport::prelude::*;

use std::io::{Write, BufRead, BufReader};
use std::time::Duration;
use std::sync::Mutex;

static SERIAL_TIMEOUT_MS: u64 = 2000;

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

impl SerialCrateTraits for MySerialCrate {
    
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

#[cfg_attr(test, automock)]
pub trait PLCSerialTraits: Send + Sync {
    /* send and expect ack in the form of OK */
    fn send_ack(&mut self, msg: &str) -> Result<(), PLCError>; 
    
    /* just send without expecting ACK */
    fn send_nack(&mut self, msg: &str) -> Result<String, PLCError>;
}

pub struct PLCSerial {
    port: Mutex<Box <dyn SerialPort>>,
    lib: Mutex<Box<dyn SerialCrateTraits>>
}
