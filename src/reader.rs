#[cfg(test)]
use mockall::automock;

use regex::Regex;

use crate::serial::err::SerialError;
use crate::serial::RFIDSerialTraits;

pub mod constants;
pub mod err;

use constants::{
    AGC, AGC_RES, AGC_RES_2, AM, AM_RES, AM_RES_2, BLOCK_CHARS, EXT_ANT, EXT_ANT_RES, INV_REQ, ISO,
    ISO_RES, RF_HALF_DATA, RF_HALF_DATA_RES, SINGLE_BLK_CHARS, SINGLE_BLK_OFFSET, SINGLE_BLK_REGEX,
    SINGLE_BLK_REQ, SINGLE_BLK_REQ_END, SINGLE_BLK_START, UUID_CHARS, UUID_REGEX, UUID_START,
};
use err::ReaderError;

/* for mocking of reader functions */
#[cfg_attr(test, automock)]
pub trait ReaderTraits: Send + Sync {
    fn read_uuid(&mut self) -> Result<String, ReaderError>;
    //returns a single block of data of information
    fn read_single_block(&mut self, block_idx: u32) -> Result<String, ReaderError>;
    // returns num chars of data
    fn read_multiple_block(
        &mut self,
        block_idx: u32,
        num_blocks: u32,
    ) -> Result<String, ReaderError>;
}

pub struct Reader {
    serial: Box<dyn RFIDSerialTraits>,
}

impl ReaderTraits for Reader {
    fn read_uuid(&mut self) -> Result<String, ReaderError> {
        let raw_uuid = self.read_raw_uuid()?;
        let reversed = reverse_uuid(&raw_uuid);
        Ok(reversed)
    }

    fn read_single_block(&mut self, block_idx: u32) -> Result<String, ReaderError> {
        let raw_uuid = self.read_raw_uuid()?;
        //get the block representation in hex
        let block_hex = format!("{:02X}", block_idx);
        if block_hex.len() != BLOCK_CHARS {
            return Err(ReaderError::BlockIdxTooLarge(block_idx));
        }

        let cmd = format!(
            "{}{}{}{}",
            SINGLE_BLK_REQ, raw_uuid, block_hex, SINGLE_BLK_REQ_END
        );

        let raw_data = self.send_read_regex(&cmd, &vec![SINGLE_BLK_REGEX])?;

        let start = raw_data.find(SINGLE_BLK_START).unwrap() + SINGLE_BLK_OFFSET;
        let end = start + SINGLE_BLK_CHARS;
        let data = &raw_data[start..end];

        Ok(String::from(data))
    }

    //TODO
    fn read_multiple_block(
        &mut self,
        _block_idx: u32,
        _num_blocks: u32,
    ) -> Result<String, ReaderError> {
        Err(ReaderError::SerialError(SerialError::NoSerialPortsFound))
    }
}

fn get_uuid(raw_str: &str) -> String {
    // let it panic, if uuid regex matched but not the UUID start
    let start_idx = raw_str.find(UUID_START).unwrap();

    let s: Vec<char> = raw_str.chars().collect();
    let start = start_idx - (UUID_CHARS - 2);
    let mut uuid = String::new();

    for i in 0..(UUID_CHARS / 2) {
        let char_idx = start + i * 2;
        uuid.push(s[char_idx]);
        uuid.push(s[char_idx + 1]);
    }
    uuid
}

fn reverse_uuid(uuid: &str) -> String {
    assert!(uuid.len() == UUID_CHARS);

    let s: Vec<char> = uuid.chars().collect();
    let mut reversed = String::new();

    for i in 1..(uuid.len() / 2) + 1 {
        let char_idx = uuid.len() - i * 2;
        reversed.push(s[char_idx]);
        reversed.push(s[char_idx + 1]);
    }
    reversed
}

impl Reader {
    pub fn new(serial: Box<dyn RFIDSerialTraits>) -> Reader {
        let mut reader = Reader { serial: serial };

        if let Err(e) = reader.initialize() {
            log::error!("{}", e);
            panic!("{}", e.to_string());
        }
        reader
    }

    fn read_raw_uuid(&mut self) -> Result<String, ReaderError> {
        let res = self.send_read_regex(INV_REQ, &vec![UUID_REGEX])?;
        let raw_uuid = get_uuid(&res);
        Ok(raw_uuid)
    }

    //send a command, and check whether output matches any of the regex
    fn send_read_regex(&mut self, cmd: &str, regex: &Vec<&str>) -> Result<String, ReaderError> {
        let read = self.serial.send_recv(cmd)?;

        for r in regex.iter() {
            //panic if any regex is invalid
            let re = match Regex::new(r) {
                Ok(re) => re,
                Err(_) => {
                    log::error!("{}", ReaderError::InvalidRegex(r.to_string()));
                    panic!("{}", ReaderError::InvalidRegex(r.to_string()).to_string());
                }
            };
            if let Some(cap) = re.captures(&read) {
                return Ok(String::from(cap.get(0).unwrap().as_str()));
            }
        }
        return Err(ReaderError::NoMatchingTargets(read));
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
        self.send_read_regex(ISO, &vec![ISO_RES])?;
        Ok(())
    }

    fn set_half_data(&mut self) -> Result<(), ReaderError> {
        self.send_read_regex(RF_HALF_DATA, &vec![RF_HALF_DATA_RES])?;
        Ok(())
    }

    fn set_agc(&mut self) -> Result<(), ReaderError> {
        self.send_read_regex(AGC, &vec![AGC_RES, AGC_RES_2])?;
        Ok(())
    }

    fn set_am(&mut self) -> Result<(), ReaderError> {
        self.send_read_regex(AM, &vec![AM_RES, AM_RES_2])?;
        Ok(())
    }

    fn set_antenna(&mut self) -> Result<(), ReaderError> {
        self.send_read_regex(EXT_ANT, &vec![EXT_ANT_RES])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::serial::MockRFIDSerialTraits;
    use mockall::predicate::eq;

    mod init {
        use super::*;

        #[test]
        #[should_panic(expected = "Serial device did not respond to read request")]
        fn serial_port_error() {
            let mut serial = MockRFIDSerialTraits::new();
            serial
                .expect_send_recv()
                .with(eq(ISO))
                .returning(|_| Err(SerialError::NoReplyAfterMultipleTries));
            let mut _reader = Reader::new(Box::new(serial));
        }

        #[test]
        #[should_panic(expected = "Targets not found")]
        fn mismatch_output() {
            let mut serial = MockRFIDSerialTraits::new();
            serial
                .expect_send_recv()
                .with(eq(ISO))
                .returning(|_| Ok(String::from("Gibberish")));
            let mut _reader = Reader::new(Box::new(serial));
        }

        //tests whether reader can initialize even with the alternate reply for agc
        #[test]
        fn agc_res_two_ok() {
            let mut serial = MockRFIDSerialTraits::new();
            serial
                .expect_send_recv()
                .with(eq(ISO))
                .returning(|_| Ok(String::from(ISO_RES)));
            serial
                .expect_send_recv()
                .with(eq(RF_HALF_DATA))
                .returning(|_| Ok(String::from(RF_HALF_DATA_RES)));
            serial
                .expect_send_recv()
                .with(eq(AGC))
                .returning(|_| Ok(String::from(AGC_RES_2)));
            serial
                .expect_send_recv()
                .with(eq(AM))
                .returning(|_| Ok(String::from(AM_RES)));
            serial
                .expect_send_recv()
                .with(eq(EXT_ANT))
                .returning(|_| Ok(String::from(EXT_ANT_RES)));
            let mut _reader = Reader::new(Box::new(serial));
        }

        //tests whether reader can initialize even with the alternate reply for am
        #[test]
        fn am_res_two_ok() {
            let mut serial = MockRFIDSerialTraits::new();
            serial
                .expect_send_recv()
                .with(eq(ISO))
                .returning(|_| Ok(String::from(ISO_RES)));
            serial
                .expect_send_recv()
                .with(eq(RF_HALF_DATA))
                .returning(|_| Ok(String::from(RF_HALF_DATA_RES)));
            serial
                .expect_send_recv()
                .with(eq(AGC))
                .returning(|_| Ok(String::from(AGC_RES)));
            serial
                .expect_send_recv()
                .with(eq(AM))
                .returning(|_| Ok(String::from(AM_RES_2)));
            serial
                .expect_send_recv()
                .with(eq(EXT_ANT))
                .returning(|_| Ok(String::from(EXT_ANT_RES)));
            let mut _reader = Reader::new(Box::new(serial));
        }

        #[test]
        fn init_ok() {
            let mut serial = MockRFIDSerialTraits::new();
            serial
                .expect_send_recv()
                .with(eq(ISO))
                .returning(|_| Ok(String::from(ISO_RES)));
            serial
                .expect_send_recv()
                .with(eq(RF_HALF_DATA))
                .returning(|_| Ok(String::from(RF_HALF_DATA_RES)));
            serial
                .expect_send_recv()
                .with(eq(AGC))
                .returning(|_| Ok(String::from(AGC_RES)));
            serial
                .expect_send_recv()
                .with(eq(AM))
                .returning(|_| Ok(String::from(AM_RES)));
            serial
                .expect_send_recv()
                .with(eq(EXT_ANT))
                .returning(|_| Ok(String::from(EXT_ANT_RES)));
            let mut _reader = Reader::new(Box::new(serial));
        }
    }

    //TODO check whether captures match the test input
    mod uuid_regex {

        use super::*;

        #[test]
        fn ok() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(re.is_match("[FFFFFFFFFFFFFFFF,00]"))
        }

        #[test]
        fn no_square_brackets() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(!re.is_match("FFFFFFFFFFFFFFFF,00"))
        }

        #[test]
        fn non_hex() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(!re.is_match("[XXXXXXXXXXXXXXXX,XX]"))
        }

        #[test]
        fn insufficient_len() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(!re.is_match("[FFFF,XX]"))
        }

        #[test]
        fn empty() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(!re.is_match(""))
        }

        #[test]
        fn missing_signal() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(!re.is_match("[FFFFFFFFFFFFFFFF, ]"))
        }

        #[test]
        fn chars_infront() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(re.is_match("XXXXXX[FFFFFFFFFFFFFFFF,FF]"))
        }

        #[test]
        fn chars_atback() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(re.is_match("[FFFFFFFFFFFFFFFF,FF]XXXXX"))
        }

        #[test]
        fn chars_frontandback() {
            let re = Regex::new(UUID_REGEX).unwrap();
            assert!(re.is_match("XXXXXX[FFFFFFFFFFFFFFFF,FF]XXXXX"))
        }
    }

    mod single_block_regex {

        use super::*;

        #[test]
        fn ok() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(re.is_match("[0012345678]"))
        }

        #[test]
        fn no_square_brackets() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(!re.is_match("0012345678"))
        }

        #[test]
        fn non_hex() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(!re.is_match("[XXXXXXXXXX]"))
        }

        #[test]
        fn insufficient_len() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(!re.is_match("[FFFF]"))
        }

        #[test]
        fn empty() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(!re.is_match(""))
        }

        #[test]
        fn chars_infront() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(re.is_match("XXXXXX[0012345678]"))
        }

        #[test]
        fn chars_atback() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(re.is_match("[0012345678]XXXXX"))
        }

        #[test]
        fn chars_frontandback() {
            let re = Regex::new(SINGLE_BLK_REGEX).unwrap();
            assert!(re.is_match("XXXXXX[0012345678]XXXXX"))
        }
    }

    //test fixture to setup device for other tests besides init
    fn init_helper(serial: &mut MockRFIDSerialTraits) {
        serial
            .expect_send_recv()
            .with(eq(ISO))
            .returning(|_| Ok(String::from(ISO_RES)));
        serial
            .expect_send_recv()
            .with(eq(RF_HALF_DATA))
            .returning(|_| Ok(String::from(RF_HALF_DATA_RES)));
        serial
            .expect_send_recv()
            .with(eq(AGC))
            .returning(|_| Ok(String::from(AGC_RES)));
        serial
            .expect_send_recv()
            .with(eq(AM))
            .returning(|_| Ok(String::from(AM_RES)));
        serial
            .expect_send_recv()
            .with(eq(EXT_ANT))
            .returning(|_| Ok(String::from(EXT_ANT_RES)));
    }

    mod uuid {
        use super::*;
        #[test]
        fn non_matching() {
            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);
            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[CAFE,FF]")));
            let mut reader = Reader::new(Box::new(serial));
            let res = reader.read_uuid();
            assert!(res.is_err());

            if let Err(e) = res {
                assert_eq!(
                    e.to_string(),
                    ReaderError::NoMatchingTargets(String::from("[CAFE,FF]")).to_string()
                );
            }
        }

        #[test]
        fn serial_error() {
            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);
            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Err(SerialError::NoReplyAfterMultipleTries));
            let mut reader = Reader::new(Box::new(serial));
            let res = reader.read_uuid();
            assert!(res.is_err());

            if let Err(e) = res {
                assert_eq!(
                    e.to_string(),
                    ReaderError::SerialError(SerialError::NoReplyAfterMultipleTries).to_string()
                );
            }
        }

        #[test]
        fn matching() {
            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);
            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[CAFEBABEDEADBEE0,FF]")));
            let mut reader = Reader::new(Box::new(serial));
            let res = reader.read_uuid();
            assert!(!res.is_err());
            assert_eq!(res.unwrap().to_string(), "E0BEADDEBEBAFECA");
        }

        #[test]
        fn chars_in_front_matching() {
            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);
            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("XXXXXXXX[CAFEBABEDEADBEE0,FF]")));
            let mut reader = Reader::new(Box::new(serial));
            let res = reader.read_uuid();
            assert!(!res.is_err());
            assert_eq!(res.unwrap().to_string(), "E0BEADDEBEBAFECA");
        }

        #[test]
        fn chars_at_the_back_matching() {
            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);
            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[CAFEBABEDEADBEE0,FF]XXXXXX")));
            let mut reader = Reader::new(Box::new(serial));
            let res = reader.read_uuid();
            assert!(!res.is_err());
            assert_eq!(res.unwrap().to_string(), "E0BEADDEBEBAFECA");
        }

        #[test]
        fn chars_at_the_front_and_back_matching() {
            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);
            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("XXXXXXXX[CAFEBABEDEADBEE0,FF]XXXXXX")));
            let mut reader = Reader::new(Box::new(serial));
            let res = reader.read_uuid();
            assert!(!res.is_err());
            assert_eq!(res.unwrap().to_string(), "E0BEADDEBEBAFECA");
        }
    }

    mod single_block {

        use super::*;

        #[test]
        fn serial_error_on_data_call() {
            let expected_cmd = "0113000304182220CAFEDEADBEEFB0E0FF0000";

            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);

            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[CAFEDEADBEEFB0E0,FF]")));
            serial
                .expect_send_recv()
                .with(eq(expected_cmd))
                .returning(|_| Err(SerialError::NoReplyAfterMultipleTries));

            let mut reader = Reader::new(Box::new(serial));
            let block_idx = 255;
            let res = reader.read_single_block(block_idx);

            assert!(res.is_err());
            if let Err(e) = res {
                assert_eq!(
                    e.to_string(),
                    ReaderError::SerialError(SerialError::NoReplyAfterMultipleTries).to_string()
                );
            }
        }

        #[test]
        fn non_matching_uuid_on_data_call() {
            let expected_cmd = "0113000304182220CAFEDEADBEEFB0E0FF0000";

            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);

            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[00E0,FF]")));
            serial
                .expect_send_recv()
                .with(eq(expected_cmd))
                .returning(|_| Err(SerialError::NoReplyAfterMultipleTries));

            let mut reader = Reader::new(Box::new(serial));
            let block_idx = 255;
            let res = reader.read_single_block(block_idx);

            assert!(res.is_err());
            if let Err(e) = res {
                assert_eq!(
                    e.to_string(),
                    ReaderError::NoMatchingTargets(String::from("[00E0,FF]")).to_string()
                );
            }
        }

        #[test]
        fn non_matching_data_on_data_calll() {
            let expected_cmd = "0113000304182220CAFEDEADBEEFB0E0FF0000";

            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);

            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[CAFEDEADBEEFB0E0,FF]")));
            serial
                .expect_send_recv()
                .with(eq(expected_cmd))
                .returning(|_| Ok(String::from("[00]")));

            let mut reader = Reader::new(Box::new(serial));
            let block_idx = 255;
            let res = reader.read_single_block(block_idx);

            assert!(res.is_err());
            if let Err(e) = res {
                assert_eq!(
                    e.to_string(),
                    ReaderError::NoMatchingTargets(String::from("[00]")).to_string()
                );
            }
        }

        #[test]
        fn ok() {
            let expected_cmd = "0113000304182220CAFEDEADBEEFB0E0FF0000";

            let mut serial = MockRFIDSerialTraits::new();
            init_helper(&mut serial);

            serial
                .expect_send_recv()
                .with(eq(INV_REQ))
                .returning(|_| Ok(String::from("[CAFEDEADBEEFB0E0,FF]")));
            serial
                .expect_send_recv()
                .with(eq(expected_cmd))
                .returning(|_| Ok(String::from("[0012345678]")));

            let mut reader = Reader::new(Box::new(serial));
            let block_idx = 255;
            let res = reader.read_single_block(block_idx);
            assert!(!res.is_err());
            assert_eq!(res.unwrap(), "12345678");
        }
    }
}
