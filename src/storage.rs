use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};

/// The `Storage` struct is a simply utility wrapper around the Rust standard library's `BufReader`
/// API. Instances of the structure are used to perform random reads on a backing store binary
/// file. Relative to the context of the project, this struct is used to read chunks of data from a
/// backing storage pool. Conceptually, this can be anything along the lines of actual file data,
/// swap space, or program instructions that have yet to be paged in.
pub struct Storage(BufReader<File>);

impl Storage {
    /// Create a new instance of the `Storage` struct.
    ///
    /// # Panics
    /// The call will panic if the file referenced by `FILENAME_STORAGE` does not exist in the
    /// location provided.
    ///
    pub fn build(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        Self(BufReader::new(file))
    }

    /// Seeks to a position in the backing store and reads a chunk of data into the the buffer
    /// passed by mutable reference. A `Result` is returned to indicate the success of the
    /// operation.
    ///
    /// The seek position is determined by the value obtained from `seek_multipler * buffer.len()`.
    ///
    /// # Arguments
    ///
    /// * `seek_multiplier` - the number of times `buffer.len()` will be multiplied to obtain the
    /// start position for the read operation.
    /// * `buffer` - a mutable reference to a buffer in which the data is to be written.
    ///
    /// # Errors
    ///
    /// Errors may occur if the buffer is not a correct size or the seek_multiplier is improperly
    /// set. Typically, such errors are the result of attempting to read past the bounds of the
    /// file provided as the backing store.
    ///
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

    use crate::config::Config;
    use clap::Parser;

    #[cfg(test)]
    mod storage_tests {

        fn standard_storage() -> Storage {
            let config = Config::parse();
            Storage::build(&config.file_storage)
        }

        use super::*;

        #[test]
        fn read() {
            let mut store = standard_storage();
            let mut buffer = vec![0 as u8; 256];
            store.read(0, &mut buffer).unwrap();
            assert_eq!(buffer[7], 0x01);
            assert_eq!(buffer[11], 0x02);
            assert_eq!(buffer[15], 0x03);
        }
    }
}
