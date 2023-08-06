use crate::{
    table::{self, Frame},
    BACKING_STORE_FILENAME,
};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Error(io::Error);
pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error(value)
    }
}

pub struct BackingStore {
    filename: String,
    reader: BufReader<File>,
    size_bytes: u64,
}

impl BackingStore {
    pub fn build() -> Self {
        let file = File::open(BACKING_STORE_FILENAME).unwrap();
        let metadata = file.metadata().unwrap();

        Self {
            filename: String::from(BACKING_STORE_FILENAME),
            reader: BufReader::new(file),
            size_bytes: metadata.len(),
        }
    }

    pub fn read_frame(&mut self, seek_multiplier: u64, frame: &mut Frame) -> Result<()> {
        self.reader
            .seek(SeekFrom::Start(frame.size() * seek_multiplier))?;
        self.reader.read(&mut frame.buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[cfg(test)]
    mod backing_store_tests {

        use super::*;

        #[test]
        fn build() {
            let store = BackingStore::build();
            assert_eq!(store.size_bytes, 0x10000);
        }

        #[test]
        fn read_frame() {
            let mut frame = table::Frame::new(256);
            let mut store = BackingStore::build();
            store.read_frame(0, &mut frame).unwrap();
            assert_eq!(frame.buffer[7], 0x01);
            assert_eq!(frame.buffer[11], 0x02);
            assert_eq!(frame.buffer[15], 0x03);
        }
    }
}
