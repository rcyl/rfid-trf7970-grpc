use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub enum SerialError<> {
    NoSerialPortsFound,
    USBSerialNotFound,
    MultipleSerialPorts,
    NoReplyAfterMultipleTries,
    IOError(std::io::Error),
    SerialError(serialport::Error),
}

impl fmt::Display for SerialError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerialError::NoSerialPortsFound =>
                write!(f, "No serial ports found. Please ensure device is connected"),
            SerialError::USBSerialNotFound =>
                write!(f, "No USB serial port found. Please ensure device is connected"),
            SerialError::MultipleSerialPorts =>
                write!(f, "More than one USB serial device found"),
            SerialError::NoReplyAfterMultipleTries =>
                write!(f, "Serial device did not respond to read request \
                            after multiple tries"),            
                SerialError::IOError(ref e) => 
                std::fmt::Display::fmt(&e, f),
            SerialError::SerialError(ref e) => 
                std::fmt::Display::fmt(&e, f),
        }
    }
}

impl From<std::io::Error> for SerialError {
    fn from(err: std::io::Error) -> SerialError {
        SerialError::IOError(err)
    }
}

impl From<serialport::Error> for SerialError {
    fn from(err: serialport::Error) -> SerialError {
        SerialError::SerialError(err)
    }
}
    