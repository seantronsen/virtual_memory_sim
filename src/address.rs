use crate::{MASK_OFFSET, MASK_PAGE};
use std::fs::File;
use std::io::{BufRead, BufReader};

/// `VirtualAddress` is an abstraction which represents the components of a virtual address within
/// a single structure. It includes includes elements such as the page number, offset, and any
/// extra bits (which have no meaning at the time of writing).
#[derive(PartialEq, Debug)]
pub struct VirtualAddress {
    pub number_page: u8,
    pub number_offset: u8,
    extra_bits: u16,
}

impl From<u32> for VirtualAddress {
    /// Provided an address in the form of a 32-bit unsigned integer, translate said address into a
    /// struct with fields storing information relative to the address components.
    ///
    /// # Arguments
    ///
    /// * `value` - 32-bit unsigned integer representing a virtual address location
    ///
    /// # Examples
    ///
    /// ```
    /// use virtual_memory_sim::address::VirtualAddress;
    /// let x: u32 = 0x00000f0f;
    /// let y = VirtualAddress::from(x);
    /// println!("{:?}", y);
    /// assert_eq!(y.number_page, 15);
    /// assert_eq!(y.number_offset, 15);
    /// ```
    fn from(value: u32) -> Self {
        Self {
            number_page: ((value & MASK_PAGE) >> 8) as u8,
            number_offset: (value & MASK_OFFSET) as u8,
            extra_bits: (((!(MASK_OFFSET | MASK_PAGE)) & value) >> 16) as u16,
        }
    }
}

/// `AddressReader` is a utility type responsible for sequentially obtaining "raw" address numbers
/// from a text file. Those obtained can be used to access data from a virtual memory system.
pub struct AddressReader {
    reader: BufReader<File>,
    pub line_number: u64,
}

impl AddressReader {
    /// Instantiate a new `AddressReader` struct for working with the provided text file. Ensure
    /// the content of the file contains only address numbers (no header information) and each line
    /// contains only one address.
    ///
    /// # Panics
    ///
    /// Instantiating a new address reader will fail if the file does not exist.
    pub fn new(filename: &str) -> Self {
        match File::open(filename) {
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

    use crate::config::Config;
    use clap::Parser;

    fn standard_reader() -> AddressReader {
        let config = Config::parse();
        AddressReader::new(&config.file_address)
    }

    #[cfg(test)]
    mod address_reader_tests {

        use super::*;

        #[test]
        fn new() {
            let mut reader = standard_reader();
            assert_eq!(reader.line_number, 0);
            assert_eq!(reader.reader.stream_position().unwrap(), 0);
        }

        #[test]
        fn iterator() {
            let mut reader = standard_reader();
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
