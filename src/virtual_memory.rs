use crate::address::VirtualAddress;
use crate::storage::Storage;
use crate::tracker::Tracker;
use std::collections::{HashMap, LinkedList};
use std::ops::Index;

// read from the configuration file to determine which algorithm to use
// this can be enforced within the build method of the table structs where flags will be set
// to choose the algorithm features such as a fifo queue or hashtable

/// Type Alias: Simple rebranding of the `Result` enum from the standard library with a focus on the errors
/// that may result from the use of this module (at least improperly).
type Result<T> = std::result::Result<T, Error>;

// The `Error` enum here is merely a formal declaration and generalization of the error kinds that
// may occur from improper use of other structures later in the module.
#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    FreeFrameUnavailable,
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value)
    }
}

/// The `Fifo<T>` struct is an implementation of the simple queue data structure that uses the
/// standard libraries linked list implementation as a backend. I've written a linked list in Rust
/// before and it was more of a pain in the ass than I prefer to admit.
/// See [here](https://rust-unofficial.github.io/too-many-lists/) for more details about the
/// nuisances with creating node structures using the standard safe subset of the Rust language.
struct Fifo<T>(LinkedList<T>, usize);

impl<T> Fifo<T> {
    /// Create a new instance of the `Fifo` struct with nodes of type `T`.
    ///
    fn new() -> Self {
        Self(LinkedList::new(), 0)
    }

    /// Returns the length of the queue.
    ///
    fn len(&self) -> usize {
        self.1
    }

    /// Add an element to the back of the queue.
    ///
    /// # Arguments
    ///
    /// * `element` - a value of type `T` to be added to the queue.
    ///
    fn enqueue(&mut self, element: T) {
        self.1 += 1;
        self.0.push_front(element);
    }

    /// Remove and return the element at the head of the queue (hence dequeue).
    ///
    fn dequeue(&mut self) -> Option<T> {
        self.1 -= 1;
        self.0.pop_back()
    }
}

/// The `AccessResult` struct stitches several values together for later use in tracking the
/// accuracy of the virtual memory implementation. Elements include the virtual address provided to
/// an operation, the corresponding physical address, and the value read in using the information. 
///
#[derive(Debug)]
pub struct AccessResult {
    pub virtual_address: VirtualAddress,
    pub physical_address: u32,
    pub value: i8,
}

impl PartialEq for AccessResult {
    fn eq(&self, other: &Self) -> bool {
        self.virtual_address == other.virtual_address && self.value == other.value
    }
}


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

struct Page {
    frame_index: usize,
    valid: bool,
}

struct PageTable(HashMap<usize, Page>);
impl PageTable {
    fn build() -> Self {
        Self(HashMap::new())
    }
    fn find(&self, id: usize) -> Option<&Page> {
        self.0.get(&id)
    }
    fn find_mut(&mut self, id: usize) -> Option<&mut Page> {
        self.0.get_mut(&id)
    }
    fn insert(&mut self, id: usize, page: Page) {
        self.0.insert(id, page);
    }
}

struct Frame {
    buffer: Vec<u8>,
    page_id: usize,
}

impl Frame {
    fn new(frame_size: u64) -> Self {
        Self {
            buffer: vec![0 as u8; frame_size as usize],
            page_id: 0,
        }
    }
}

impl Index<usize> for Frame {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

struct FrameTable {
    table_size: usize,
    frame_size: u64,
    entries: Vec<Frame>,
    available: Fifo<usize>,
    victimizer: Fifo<usize>,
}

impl FrameTable {
    fn build(table_size: usize, frame_size: u64) -> Self {
        let mut entries: Vec<Frame> = Vec::with_capacity(table_size);
        let mut available = Fifo::new();
        let victimizer = Fifo::new();
        (0..table_size).for_each(|index| {
            entries.push(Frame::new(frame_size));
            available.enqueue(index);
        });

        Self {
            frame_size,
            table_size,
            entries,
            victimizer,
            available,
        }
    }

    fn allocate(&mut self) -> usize {
        if self.victimizer.len() == self.table_size {
            self.reaper();
        }
        let alloc_index = self
            .available
            .dequeue()
            .expect("should have an available frame");
        self.victimizer.enqueue(alloc_index);
        alloc_index
    }

    pub fn reaper(&mut self) {
        let target = self
            .victimizer
            .dequeue()
            .expect("should have victims at the ready");
        self.available.enqueue(target);
    }
}

pub struct VirtualMemory {
    tlb: TLB,
    pages: PageTable,
    frames: FrameTable,
    storage: Storage,
}

impl VirtualMemory {
    pub fn build(tlb_size: usize, frame_table_size: usize, frame_size: u64) -> Self {
        Self {
            tlb: TLB::build(tlb_size),
            pages: PageTable::build(),
            frames: FrameTable::build(frame_table_size, frame_size),
            storage: Storage::build(),
        }
    }

    pub fn access(
        &mut self,
        virtual_address: VirtualAddress,
        _tracker: &mut Tracker,
    ) -> Result<AccessResult> {
        let page_number = virtual_address.number_page as usize;
        let offset = virtual_address.number_offset as usize;
        let frame_index = match self.tlb.find(page_number) {
            Some(x) => {
                _tracker.tlb_hits += 1;
                *x
            }
            _ => {
                _tracker.tlb_faults += 1;
                match self.pages.find(page_number) {
                    Some(page) if page.valid => {
                        _tracker.page_hits += 1;
                        self.tlb.replace(page_number, page.frame_index);
                        page.frame_index
                    }
                    _ => {
                        _tracker.page_faults += 1;
                        let fi = self.retrieve_frame(virtual_address.number_page as usize)?;
                        self.tlb.replace(page_number, fi);
                        fi
                    }
                }
            }
        };

        Ok(AccessResult {
            virtual_address,
            physical_address: frame_index as u32 * self.frames.frame_size as u32 + offset as u32,
            value: self.frames.entries[frame_index][offset] as i8,
        })
    }

    fn retrieve_frame(&mut self, page_number: usize) -> Result<usize> {
        let frame_index = self.frames.allocate();
        let frame = &mut self.frames.entries[frame_index];
        if let Some(page) = self.pages.find_mut(frame.page_id) {
            page.valid = false;
        }
        frame.page_id = page_number;
        self.storage.read(page_number as u64, &mut frame.buffer)?;
        self.pages.insert(
            page_number as usize,
            Page {
                frame_index,
                valid: true,
            },
        );

        Ok(frame_index)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{SIZE_FRAME, SIZE_TABLE};

    #[cfg(test)]
    mod page_tests {

        use super::*;

        #[test]
        fn new() {
            let page = Page {
                frame_index: 0xF,
                valid: false,
            };
            assert_eq!(page.valid, false);
            assert_eq!(page.frame_index, 0xF);
        }
    }

    #[cfg(test)]
    mod page_table_tests {}

    #[cfg(test)]
    mod frame_tests {

        use super::*;

        #[test]
        fn new() {
            let frame = Frame::new(SIZE_FRAME);
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
