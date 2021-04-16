#[cfg(test)]
use mockall::automock;

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
            log::error!("{}", SerialError::MultipleSerialPorts);
            panic!(SerialError::MultipleSerialPorts.to_string());
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

#[cfg(test)]
mod test {
    type Result<T> = std::result::Result<T, serialport::Error>;
    use serialport::{SerialPortInfo, SerialPortSettings, SerialPortType, ErrorKind};
    use mockall::{mock, Sequence};
    use std::io::{Read, Write};
    use crate::serial::MockSerialCrateTraits;
    use super::*;

    mock! {
        SerialPort {}
        pub trait SerialPort {
            fn name(&self) -> Option<String>;
            fn settings(&self) -> SerialPortSettings;
            fn baud_rate(&self) -> Result<u32>;
            fn data_bits(&self) -> Result<DataBits>;
            fn flow_control(&self) -> Result<FlowControl>;
            fn parity(&self) -> Result<Parity>;
            fn stop_bits(&self) -> Result<StopBits>;
            fn timeout(&self) -> Duration;
            fn set_all(&mut self, settings: &SerialPortSettings) -> Result<()>;
            fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()>;
            fn set_data_bits(&mut self, data_bits: DataBits) -> Result<()>;
            fn set_flow_control(&mut self, flow_control: FlowControl) -> Result<()>;
            fn set_parity(&mut self, parity: Parity) -> Result<()>;
            fn set_stop_bits(&mut self, stop_bits: StopBits) -> Result<()>;
            fn set_timeout(&mut self, timeout: Duration) -> Result<()>;
            fn write_request_to_send(&mut self, level: bool) -> Result<()>;
            fn write_data_terminal_ready(&mut self, level: bool) -> Result<()>;
            fn read_clear_to_send(&mut self) -> Result<bool>;
            fn read_data_set_ready(&mut self) -> Result<bool>;
            fn read_ring_indicator(&mut self) -> Result<bool>;
            fn read_carrier_detect(&mut self) -> Result<bool>;
            fn bytes_to_read(&self) -> Result<u32>;
            fn bytes_to_write(&self) -> Result<u32>;
            fn clear(&self, buffer_to_clear: ClearBuffer) -> Result<()>;
            fn try_clone(&self) -> Result<Box<dyn SerialPort>>;
        }

        trait Write {
            fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error>;
            fn flush(&mut self) -> std::result::Result<(), std::io::Error>;
        }

        trait Read {
            fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error>;
        }
    }

    #[test]
    #[should_panic(expected = "No serial ports found. Please ensure device is connected")]
    fn empty_serial_port() {
      
        let mut s = MockSerialCrateTraits::new();
        s.expect_get_ports().returning(|| {
            //empty vector of serial ports
            let v: Vec<SerialPortInfo> = Vec::new();
            Ok(v)
        });
        let _rs = RFIDSerial::new(Box::new(s));        
    }

    #[test]
    #[should_panic(expected = "No USB serial port found. Please ensure device is connected")]
    fn no_usb_serial_port() {

        let mut s = MockSerialCrateTraits::new();
        s.expect_get_ports().returning(|| {
            let mut v: Vec<SerialPortInfo> = Vec::new();
            v.push(SerialPortInfo{
                    port_name: String::from("/dev/ttyS1"),
                    port_type: SerialPortType::Unknown
                });
            Ok(v)
        });

        let _rs = RFIDSerial::new(Box::new(s));          
    }

    #[test]
    #[should_panic(expected = "More than one USB serial device found")]
    fn more_than_one_usb_serial_port() {

        let mut s = MockSerialCrateTraits::new();
        s.expect_get_ports().returning(|| {
            let mut v: Vec<SerialPortInfo> = Vec::new();
            v.push(SerialPortInfo{
                    port_name: String::from("/dev/USB0"),
                    port_type: SerialPortType::Unknown
                });
            v.push(SerialPortInfo{
                port_name: String::from("/dev/ttyUSB1"),
                port_type: SerialPortType::Unknown
            });
            Ok(v)
        });
        let _rs = RFIDSerial::new(Box::new(s));          
    }

    #[test]
    #[should_panic(expected = "SerialPortError")]
    fn serialport_error() {

        let mut s = MockSerialCrateTraits::new();
        s.expect_get_ports().returning(|| {
            let e = serialport::Error {
                kind: ErrorKind::NoDevice,
                description: String::from("SerialPortError")
            };
            Err(e)
        });

        let _rs = RFIDSerial::new(Box::new(s));          
    }

    #[test]
    #[should_panic(expected = "Failed to open")]
    fn failed_to_open() {

        let mut s = MockSerialCrateTraits::new();
        s.expect_get_ports().returning(|| {
            let mut v: Vec<SerialPortInfo> = Vec::new();
            v.push(SerialPortInfo{
                port_name: String::from("/dev/ttyUSB0"),
                port_type: SerialPortType::Unknown
            });
            Ok(v)
        });
        s.expect_open().returning(|_, _| {
            let e = serialport::Error {
                kind: ErrorKind::NoDevice,
                description: String::from("Failed to open")
            };
            Err(e)
        });

       let _rs = RFIDSerial::new(Box::new(s));           
    }

    //TODO: not sure how to fake a legit serial port object
    // #[test]
    // fn ok() {

    // }
}