use crate::FILENAME_STORAGE;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};

pub struct Storage(BufReader<File>);

impl Storage {
    pub fn build() -> Self {
        let file = File::open(FILENAME_STORAGE).unwrap();
        Self(BufReader::new(file))
    }

    pub fn read(&mut self, seek_multiplier: u64, buffer: &mut Vec<u8>) -> Result<(), io::Error> {
        let seek_pos = SeekFrom::Start(buffer.len() as u64 * seek_multiplier);
        self.0.seek(seek_pos)?;
        self.0.read(buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[cfg(test)]
    mod storage_tests {

        use super::*;

        #[test]
        fn read() {
            let mut store = Storage::build();
            let mut buffer = vec![0 as u8; 256];
            store.read(0, &mut buffer).unwrap();
            assert_eq!(buffer[7], 0x01);
            assert_eq!(buffer[11], 0x02);
            assert_eq!(buffer[15], 0x03);
        }
    }
}
