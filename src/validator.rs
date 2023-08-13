use crate::{address::VirtualAddress, table::AccessResult, FILENAME_VALIDATION};
use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader},
};

// idea: so for validation, we mostly just need to compare the target byte values for each virtual
// address. however, it would be a plus to ensure that it also got the correct physical address,
// but this would require a few additional properties to be implemented elsewhere (namely value
// tracking during virtual memory access.
// => validation_entry vs. access_entry
//
//
// for now, just implement a struct to store the 'target' results and compare against the expected
// value
pub struct ValidationReader {
    filename: String,
    reader: BufReader<File>,
    pub line_number: u64,
}

impl ValidationReader {
    pub fn new() -> Self {
        match File::open(FILENAME_VALIDATION) {
            Err(e) => panic!("error: {:?}", e),
            Ok(ptr) => Self {
                filename: String::from(FILENAME_VALIDATION),
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
            let mut reader = ValidationReader::new();
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
