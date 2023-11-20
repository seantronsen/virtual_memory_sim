use crate::{address::VirtualAddress, virtual_memory::AccessResult};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

// Similar to `address::AddressReader`, `ValidationReader` is used to read addresses into memory.
// Where the former is used to read the raw address, the latter is used to validate whether an
// implementation accessed and returned data from the correct segment of virtual memory.
pub struct ValidationReader {
    reader: BufReader<File>,
    pub line_number: u64,
}

impl ValidationReader {
    /// Create a new `ValidationReader` instance for checking virtual memory implementation
    /// results.
    ///
    /// # Panics
    ///
    /// Panics if the provided filename in the configuration does not exist.
    ///
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

impl Iterator for ValidationReader {
    type Item = AccessResult;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = String::new();
        match self.reader.read_line(&mut buffer) {
            Err(err) => panic!("error: {:?}", err),
            Ok(0) => None,
            Ok(_) => {
                self.line_number += 1;
                let values = buffer.trim().split(' ').collect::<Vec<&str>>();
                Some(AccessResult {
                    virtual_address: VirtualAddress::from(values[2].parse::<u32>().unwrap()),
                    physical_address: values[5].parse::<u32>().unwrap(),
                    value: values[7].parse::<i8>().unwrap(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::config::Config;
    use clap::Parser;

    fn standard_reader() -> ValidationReader {
        let config = Config::parse();
        ValidationReader::new(&config.file_validation)
    }

    #[cfg(test)]
    mod validation_entry_tests {
        use super::*;

        #[test]
        fn equals() {
            let a = AccessResult {
                virtual_address: VirtualAddress::from(32),
                physical_address: 64,
                value: 14,
            };
            let b = AccessResult {
                virtual_address: VirtualAddress::from(32),
                physical_address: 64,
                value: 14,
            };
            let c = AccessResult {
                virtual_address: VirtualAddress::from(33),
                physical_address: 64,
                value: 14,
            };

            assert_eq!(a, b);
            assert_ne!(a, c);
        }
    }

    #[cfg(test)]
    mod validation_reader_tests {
        use super::*;

        #[test]
        fn iterator() {
            let mut reader = standard_reader();
            assert_eq!(
                reader.next().unwrap(),
                AccessResult {
                    virtual_address: VirtualAddress::from(16916),
                    physical_address: 20,
                    value: 0,
                }
            );

            assert_eq!(
                reader.last().unwrap(),
                AccessResult {
                    virtual_address: VirtualAddress::from(12107),
                    physical_address: 2635,
                    value: -46,
                }
            );
        }
    }
}
