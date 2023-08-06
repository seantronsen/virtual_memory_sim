use crate::address::VirtualAddress;
use crate::backing_store::{self, BackingStore};
use crate::{SIZE_FRAME, SIZE_TABLE};
use std::collections::LinkedList;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub enum Error {
    StoreError(backing_store::Error),
    FreeFrameUnavailable,
}
type Result<T> = std::result::Result<T, Error>;

impl From<backing_store::Error> for Error {
    fn from(value: backing_store::Error) -> Self {
        Error::StoreError(value)
    }
}

pub struct Page {
    pub frame_index: usize,
    pub valid: bool,
}

impl Page {
    fn new(frame_index: usize) -> Self {
        Self {
            frame_index,
            valid: false,
        }
    }
}

pub struct PageTable {
    table_size: usize,
    entries: Vec<Page>,
}

impl PageTable {
    pub fn build(table_size: usize) -> Self {
        let mut entries: Vec<Page> = Vec::with_capacity(table_size);
        (0..table_size).for_each(|_| entries.push(Page::new(0)));
        Self {
            table_size,
            entries,
        }
    }
}

impl Index<usize> for PageTable {
    type Output = Page;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub struct Frame {
    pub buffer: Vec<u8>,
    pub valid: bool,
}

impl Frame {
    // todo: shouldn't be able to make these outside of a table
    fn new(frame_size: u64) -> Self {
        Self {
            buffer: vec![0 as u8; frame_size as usize],
            valid: false,
        }
    }

    pub fn size(&self) -> u64 {
        self.buffer.len() as u64
    }
}

impl Index<usize> for Frame {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl IndexMut<usize> for Frame {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

struct FreeFrameQueue(LinkedList<usize>);

impl FreeFrameQueue {
    fn new() -> Self {
        Self(LinkedList::new())
    }

    fn enqueue(&mut self, free_index: usize) {
        self.0.push_front(free_index);
    }

    fn dequeue(&mut self) -> Option<usize> {
        self.0.pop_back()
    }
}

pub struct FrameTable {
    table_size: usize,
    frame_size: u64,
    entries: Vec<Frame>,
    available: FreeFrameQueue,
}

impl FrameTable {
    pub fn build(table_size: usize, frame_size: u64) -> Self {
        let mut entries: Vec<Frame> = Vec::with_capacity(table_size);
        let mut available = FreeFrameQueue::new();

        (0..table_size).for_each(|index| {
            let frame = Frame::new(frame_size);
            entries.push(frame);
            available.enqueue(index);
        });

        Self {
            frame_size,
            table_size,
            entries,
            available,
        }
    }

    pub fn next_available(&mut self) -> Option<usize> {
        self.available.dequeue()
    }

    pub fn reclaim(&mut self, index: usize) {
        self.entries[index].valid = false;
        self.available.enqueue(index);
    }
}

impl Index<usize> for FrameTable {
    type Output = Frame;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for FrameTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub struct VirtualMemory {
    pages: PageTable,
    frames: FrameTable,
    bstore: BackingStore,
}

impl VirtualMemory {
    pub fn build(page_table_size: usize, frame_table_size: usize, frame_size: u64) -> Self {
        Self {
            pages: PageTable::build(page_table_size),
            frames: FrameTable::build(frame_table_size, frame_size),
            bstore: BackingStore::build(),
        }
    }

    pub fn access(&mut self, logical_address: VirtualAddress) -> Result<u8> {
        let page_number = logical_address.number_page as usize;
        let offset = logical_address.number_offset as usize;

        if !self.pages[page_number].valid {
            self.retrieve_page_frame(logical_address.number_page as u64)?;
        }
        let page = &self.pages[page_number];
        let byte_value = &self.frames[page.frame_index][offset];
        Ok(*byte_value)
    }

    fn retrieve_page_frame(&mut self, page_number: u64) -> Result<()> {
        let page = &mut self.pages[page_number as usize];
        let frame_index = self
            .frames
            .next_available()
            .ok_or(Error::FreeFrameUnavailable)?;
        let frame = &mut self.frames[frame_index];
        self.bstore.read_frame(page_number, frame)?;
        frame.valid = true;
        page.frame_index = frame_index;
        page.valid = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[cfg(test)]
    mod page_tests {

        use super::*;

        #[test]
        fn new() {
            let page = Page::new(0xF);
            assert_eq!(page.valid, false);
            assert_eq!(page.frame_index, 0xF);
        }
    }

    #[cfg(test)]
    mod page_table_tests {

        use super::*;

        #[test]
        fn build() {
            let page_table = PageTable::build(SIZE_TABLE);
            assert_eq!(page_table.table_size, SIZE_TABLE);
            assert!(page_table.entries.iter().all(|x| !x.valid));
        }
    }

    #[cfg(test)]
    mod frame_tests {

        use super::*;

        #[test]
        fn new() {
            let frame = Frame::new(SIZE_FRAME);
            assert_eq!(frame.valid, false);
            assert_eq!(frame.buffer.len(), SIZE_FRAME as usize);
            assert!(frame.buffer.iter().all(|x| *x == 0));
        }
    }

    #[cfg(test)]
    mod free_frame_queue_tests {

        use super::*;

        #[test]
        fn new() {
            let ffq = FreeFrameQueue::new();
            assert_eq!(ffq.0.len(), 0);
        }

        #[test]
        fn enqueue_dequeue() {
            let mut ffq = FreeFrameQueue::new();
            let values = 0..10;
            values.clone().for_each(|x| ffq.enqueue(x));
            values
                .clone()
                .for_each(|x| assert_eq!(ffq.dequeue(), Some(x)));
        }
    }

    #[cfg(test)]
    mod frame_table_tests {

        use super::*;

        #[test]
        fn new() {
            let ft = FrameTable::build(SIZE_TABLE, SIZE_FRAME);
            assert_eq!(ft.available.0.len(), SIZE_TABLE);
            assert_eq!(ft.entries.len(), SIZE_TABLE);
            assert_eq!(ft.table_size, SIZE_TABLE);
            assert_eq!(ft.frame_size, SIZE_FRAME);
        }
    }
}
