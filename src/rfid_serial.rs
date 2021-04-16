
use serialport::{SerialPortInfo, SerialPortSettings};
use serialport::prelude::*;

use std::time::Duration;
use std::sync::Mutex;

use super::serial_err::SerialError;
use super::serial::SerialCrateTraits;

static TRIES: u32 = 3;
static SERIAL_TIMEOUT_MS: u64 = 2000;

#[cfg_attr(test, automock)]
pub trait RFIDSerialTraits: Send + Sync {
    /* send and return received string */
    fn send_recv(&mut self, msg: &str) -> Result<String, SerialError>; 
}

pub struct RFIDSerial {
    port: Mutex<Box <dyn SerialPort>>,
    lib: Mutex<Box<dyn SerialCrateTraits>>
}

impl RFIDSerialTraits for RFIDSerial {

    fn send_recv(&mut self, cmd: &str) -> Result<String, SerialError> {

        let mut attempts = TRIES;
        while attempts > 0 {
            match self.send(cmd) {
                Ok(recv) => { return Ok(recv); }
                Err(_) => { attempts -= 1; }
            }
        }
        Err(SerialError::NoReplyAfterMultipleTries)
    }
}

impl RFIDSerial {

    pub fn new(lib: Box<dyn SerialCrateTraits>) -> RFIDSerial {

        let ports = match lib.get_ports() {
            Ok(ports) => ports, 
            Err(e) => { 
                //error!("{}", SerialError::SerialError(e));
                panic!(SerialError::SerialError(e).to_string());
            }
        };

        if ports.is_empty() {
            log::error!("{}", SerialError::NoSerialPortsFound);
            panic!(SerialError::NoSerialPortsFound.to_string());
        }
    
        let usb_ports: Vec<&SerialPortInfo> 
            = ports.iter().filter(|p| p.port_name.contains("USB")).collect();
        
        if usb_ports.is_empty() {
            log::error!("{}", SerialError::USBSerialNotFound);
            panic!(SerialError::USBSerialNotFound.to_string());
        }

        if usb_ports.len() > 1 {
            log::error!("{}", SerialError::USBSerialNotFound);
            panic!(SerialError::USBSerialNotFound.to_string());
        }

        let s = SerialPortSettings {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(SERIAL_TIMEOUT_MS),
        };

        /* try to open port */
        let port = match lib.open(&usb_ports[0].port_name, &s) {
            Ok(port) => { port },
            Err(e) => {
                //log::error!("{}", SerialError::SerialError(e));
                panic!(SerialError::SerialError(e).to_string());
            }
        };

        RFIDSerial {
            port: Mutex::new(port),
            lib: Mutex::new(lib),
        }
    }   

    pub fn send(&mut self, msg: &str) -> Result<String, std::io::Error> {
        
        /* let the program panic if lock fails */
        let lib = self.lib.lock().unwrap();
        let mut port = self.port.lock().unwrap();
        return lib.send(&msg, &mut port);
    }
}