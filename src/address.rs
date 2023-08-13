use crate::{FILENAME_ADDRESS, MASK_OFFSET, MASK_PAGE};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(PartialEq, Debug)]
pub struct VirtualAddress {
    pub number_page: u8,
    pub number_offset: u8,
    extra_bits: u16,
}

impl From<u32> for VirtualAddress {
    fn from(value: u32) -> Self {
        Self {
            number_page: ((value & MASK_PAGE) >> 8) as u8,
            number_offset: (value & MASK_OFFSET) as u8,
            extra_bits: (((!(MASK_OFFSET | MASK_PAGE)) & value) >> 16) as u16,
        }
    }
}

pub struct AddressReader {
    reader: BufReader<File>,
    pub line_number: u64,
}

impl AddressReader {
    pub fn new() -> Self {
        match File::open(FILENAME_ADDRESS) {
            Err(e) => panic!("error: {:?}", e),
            Ok(ptr) => Self {
                reader: BufReader::new(ptr),
                line_number: 0,
            },
        }
    }
}

impl Iterator for AddressReader {
    type Item = VirtualAddress;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = String::new();
        match self.reader.read_line(&mut buffer) {
            Err(err) => panic!("error: {:?}", err),
            Ok(0) => None,
            Ok(_) => {
                let value = buffer
                    .trim()
                    .parse::<u32>()
                    .expect("expected an integer value");
                self.line_number += 1;
                Some(VirtualAddress::from(value))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Seek;

    #[cfg(test)]
    mod address_reader_tests {

        use super::*;

        #[test]
        fn new() {
            let mut reader = AddressReader::new();
            assert_eq!(reader.line_number, 0);
            assert_eq!(reader.reader.stream_position().unwrap(), 0);
        }

        #[test]
        fn iterator() {
            let mut reader = AddressReader::new();
            assert_eq!(reader.next(), Some(VirtualAddress::from(16916)));
            assert_eq!(reader.last(), Some(VirtualAddress::from(12107)));
        }
    }

    #[cfg(test)]
    mod address_tests {

        use super::*;

        #[test]
        fn from() {
            let original: u32 = 0xabcd1234;
            let address = VirtualAddress::from(original);
            assert_eq!(address.number_offset, 0x34);
            assert_eq!(address.number_page, 0x12);
            assert_eq!(address.extra_bits, 0xabcd);
        }

        #[test]
        fn eq() {
            let mut address = VirtualAddress::from(0);
            address.extra_bits = 0xabcd;
            address.number_page = 0x12;
            address.number_offset = 0x34;
            assert_eq!(address, VirtualAddress::from(0xabcd1234))
        }
    }
}
