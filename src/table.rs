use crate::address::VirtualAddress;
use crate::backing_store::{self, BackingStore};
use crate::{SIZE_FRAME, SIZE_TABLE, SIZE_TLB};
use std::collections::{HashMap, LinkedList};
use std::ops::{Index, IndexMut};

// read from the configuration file to determine which algorithm to use
// this can be enforced within the build method of the table structs where flags will be set
// to choose the algorithm features such as a fifo queue or hashtable

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
struct Fifo<T>(LinkedList<T>);

impl<T> Fifo<T> {
    fn new() -> Self {
        Self(LinkedList::new())
    }

    fn enqueue(&mut self, free_index: T) {
        self.0.push_front(free_index);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.0.pop_back()
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

/// todo: upgrade this to use a hashed lookup
struct TLB {
    table_size: usize,
    map: HashMap<usize, usize>,
    victimizer: Fifo<usize>,
}

impl TLB {
    fn build(table_size: usize) -> Self {
        let fifo = Fifo::new();
        Self {
            table_size,
            map: HashMap::new(),
            victimizer: fifo,
        }
    }

    /// return tlb mapping as ref if possible, else none
    /// returning none implies a tlb fault
    fn find(&self, page_number: usize) -> Option<&usize> {
        self.map.get(&page_number)
    }

    /// if not at size, simple insert, else replace
    fn replace(&mut self, key: usize, value: usize) {
        if self.map.len() == self.table_size {
            let victim = self.victimizer.dequeue().unwrap();
            self.map.remove(&victim).unwrap();
        }
        self.map.insert(key, value);
        self.victimizer.enqueue(key);
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

pub struct FrameTable {
    table_size: usize,
    frame_size: u64,
    entries: Vec<Frame>,
    available: Fifo<usize>,
}

impl FrameTable {
    pub fn build(table_size: usize, frame_size: u64) -> Self {
        let mut entries: Vec<Frame> = Vec::with_capacity(table_size);
        let mut available = Fifo::new();

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

    pub fn claim(&mut self) -> Option<usize> {
        self.available.dequeue()
    }

    pub fn discard(&mut self, index: usize) {
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
    tlb: TLB,
    pages: PageTable,
    frames: FrameTable,
    bstore: BackingStore,
}

impl VirtualMemory {
    pub fn build(
        tlb_size: usize,
        page_table_size: usize,
        frame_table_size: usize,
        frame_size: u64,
    ) -> Self {
        Self {
            tlb: TLB::build(tlb_size),
            pages: PageTable::build(page_table_size),
            frames: FrameTable::build(frame_table_size, frame_size),
            bstore: BackingStore::build(),
        }
    }

    /// idea:
    /// 1. give a logical address, we first need to parse it into the correct physical parts for
    /// memory access.
    /// 2. check the page number against the TLB
    /// 3. check the page number against the page table
    /// 4. obtain the target frame and adjust the tables in case of any faults
    /// 5. return the value
    pub fn access(&mut self, logical_address: VirtualAddress) -> Result<u8> {
        let page_number = logical_address.number_page as usize;
        let offset = logical_address.number_offset as usize;

        let frame_index = match self.tlb.find(page_number) {
            Some(x) => *x,
            None => {
                if !self.pages[page_number].valid {
                    self.retrieve_page_frame(logical_address.number_page as u64)?;
                }
                let index = self.pages[page_number].frame_index;
                self.tlb.replace(page_number, index);
                index
            }
        };

        let byte_value = &self.frames[frame_index][offset];
        Ok(*byte_value)
    }

    fn retrieve_page_frame(&mut self, page_number: u64) -> Result<()> {
        let page = &mut self.pages[page_number as usize];
        let frame_index = self.frames.claim().ok_or(Error::FreeFrameUnavailable)?;
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
    mod fifo_tests {

        use super::*;

        #[test]
        fn new() {
            let ffq = Fifo::<usize>::new();
            assert_eq!(ffq.0.len(), 0);
        }

        #[test]
        fn enqueue_dequeue() {
            let mut ffq = Fifo::new();
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

    #[cfg(test)]
    mod tlb_tests {

        use super::*;
        const SIZE_TEST: usize = 3;

        #[test]
        fn build() {
            let tlb = TLB::build(SIZE_TEST);
            assert_eq!(tlb.victimizer.0.len(), 0);
            assert_eq!(tlb.map.len(), 0);
            assert_eq!(tlb.table_size, SIZE_TEST);
        }

        #[test]
        fn find_and_replace() {
            let mut tlb = TLB::build(SIZE_TEST);

            (0..5).for_each(|x| {
                assert!(tlb.find(x).is_none());
                tlb.replace(x, x);
                assert!(tlb.find(x).is_some());
            });

            assert!(tlb.find(0).is_none());
        }
    }
}
