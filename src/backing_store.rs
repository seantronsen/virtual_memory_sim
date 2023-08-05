use crate::table;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

pub struct BackingStore {
    filename: String,
    reader: BufReader<File>,
    size_bytes: u64,
}

impl BackingStore {
    pub fn build() -> Self {
        let filename = "BACKING_STORE.bin";
        let file = File::open(filename).unwrap();
        let metadata = file.metadata().unwrap();
        let size_bytes = metadata.len();
        let reader = BufReader::new(file);

        Self {
            filename: String::from(filename),
            reader,
            size_bytes,
        }
    }

    pub fn read_frame(&mut self, seek_multiplier: usize, frame: &mut table::Frame) {
        let buffer = frame.buffer_mut();
        let size = buffer.len();
        let seek_position = SeekFrom::Start((size * seek_multiplier) as u64);
        self.reader.seek(seek_position).unwrap();
        self.reader.read(buffer).unwrap();
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
            store.read_frame(0, &mut frame);
            assert_eq!(frame.buffer[7], 0x01);
            assert_eq!(frame.buffer[11], 0x02);
            assert_eq!(frame.buffer[15], 0x03);
        }
    }
}
