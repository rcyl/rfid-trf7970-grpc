
use serialport::{SerialPortInfo, SerialPortSettings};
use serialport::prelude::*;

use std::io::{Write, BufRead, BufReader};
use std::time::Duration;
use std::sync::Mutex;

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

impl PLCSerialTraits for RFIDSerial {

    fn send_recv(&mut self, cmd: &str) -> Result<String, SerialError> {

        let mut attempts = TRIES;
        while attempts > 0 {
            if self.send_ok(cmd).is_err() {
                attempts -= 1;
            } else {
                return Ok(());
            }
        }
        Err(PLCError::NoAckAfterMultipleTries)
    }
}

impl PLCSerial {

    pub fn new(lib: Box<dyn SerialCrateTraits>) -> PLCSerial {

        let ports = match lib.get_ports() {
            Ok(ports) => ports, 
            Err(e) => { 
                //unable to Clone error
                //error!("{}", PLCError::SerialError(e));
                panic!(PLCError::SerialError(e).to_string());
            }
        };

        if ports.is_empty() {
            log::error!("{}", PLCError::NoSerialPortsFound);
            panic!(PLCError::NoSerialPortsFound.to_string());
        }
    
        let usb_ports: Vec<&SerialPortInfo> 
            = ports.iter().filter(|p| p.port_name.contains("USB")).collect();
        
        if usb_ports.is_empty() {
            log::error!("{}", PLCError::USBSerialNotFound);
            panic!(PLCError::USBSerialNotFound.to_string());
        }

        let s = SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::Even,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(SERIAL_TIMEOUT_MS),
        };


        /* Iterate through all and find which one responds to ping */
        let mut res: Result<Box <dyn SerialPort>, PLCError> 
            = Err(PLCError::PLCNotFound);

        for p in usb_ports {
            match lib.open(&p.port_name, &s) {
                Ok(mut port) => {
                    if lib.ping(&mut port) { 
                        res = Ok(port);
                        break; 
                    } else {
                        continue;
                    }
                }
                /* just continue to open other ports if failed 
                to open this particular port */
                Err(e) => { 
                    res = Err(PLCError::SerialError(e));
                    continue; 
                }
            }
        }

        let port = match res {
            Ok(port) => port,
            Err(e) => {
                log::error!("{}", e);
                panic!(e.to_string());
            } 
        };

        PLCSerial {
            port: Mutex::new(port),
            lib: Mutex::new(lib),
        }
    }   

    pub fn send(&mut self, msg: &str) -> Result<String, std::io::Error> {
        
        /* let the program panic if lock fails */
        let lib = self.lib.lock().unwrap();
        let mut port = self.port.lock().unwrap();
        let concat_msg = dtf_rs::concat_str(msg, "\n");
        return lib.send(&concat_msg, &mut port);
    }
}