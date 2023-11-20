use crate::address::VirtualAddress;
use crate::storage::Storage;
use crate::tracker::Tracker;
use linked_hash_map::LinkedHashMap;
use std::collections::{HashMap, LinkedList};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

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

// idea to fix this is to completely shed away the victimizer list and the original map in
// favor of the linked hash map struct. has all the features of a regular hash map while keeping
// elements in insertion order. it will also make the creation of LRU much simpler since the struct
// possesses all the features that we already need.

/// The `TLB` struct is intended to be a simple virtualization of the translation look aside buffer
/// commonly found on most CPUs today.
struct TLB {
    table_size: usize,
    map: LinkedHashMap<usize, usize>,
}

impl TLB {
    /// Create and return a new `TLB` instance with the provided cache size.
    ///
    /// # Arguments
    ///
    /// * `table_size` - an unsigned integer value representing the maximum number of elements the
    /// TLB is allowed to cache.
    ///
    fn build(table_size: usize) -> Self {
        Self {
            table_size,
            map: LinkedHashMap::with_capacity(table_size),
        }
    }

    /// Search the TLB for the requested page and return the result as an `Option`. Note that
    /// returning a `None` value implies a TLB fault (a cache miss) has occurred.
    ///
    /// # Arguments
    ///
    /// * `page_number` - [TODO:description]
    ///
    fn find(&self, page_number: usize) -> Option<&usize> {
        self.map.get(&page_number)
    }

    /// Provided a key (logical page number) and value (physical frame number), cache the data for
    /// future use in efforts to avoid a full page table lookup. In physical computers, this action
    /// is performed with the knowledge that requested data is often used frequently. Storing a
    /// cache in this manner eliminates the need to search the page table on a cache hit and
    /// thereby eliminates 2+ load (dereference) instructions. Realize one dereference occurs when loading
    /// the value (address) stored in the page table, another occurs when loading the data
    /// referenced by that value. This pattern continues $n$ times for a page table with $n$ levels
    /// of indirection.
    ///
    /// # Arguments
    ///
    /// * `key` - the logical page number to cache
    /// * `value` - the actual frame number which exists in memory.
    ///
    fn cache_element(&mut self, key: usize, value: usize) {
        if self.map.len() == self.table_size {
            self.map.pop_back();
        }
        self.map.insert(key, value);
    }

    fn flush_element(&mut self, key: usize) -> bool {
        self.map.remove(&key).is_some()
    }
}

/// The `Page` struct represents the simplest element of the simulated page table. It serves as
/// little more than a wrapper that specifies which physical frame the page is associated with and
/// whether that frame is still valid (meaning whether it has been victimized or paged out).
#[derive(Debug, PartialEq)]
struct Page {
    frame_index: usize,
    valid: bool,
}

/// The `PageTable` struct is little more than a wrapper around the standard Rust library `HashMap`
/// that maintains only the most essential operations. A seemingly infinite number of pages can be
/// added to this table, but understand that it only contains logical references to physical
/// memory. Whether that memory is actually allocated, available, and/or still valid entirely
/// depends on the victimization algorithm and the total amount of physical memory available (or in
/// this case, configured).
struct PageTable(HashMap<usize, Page>);
impl PageTable {
    /// Create a new instance of the `PageTable` struct for use in simulating virtual memory.
    ///
    fn build() -> Self {
        Self(HashMap::new())
    }

    /// Provided a page number, attempt to find the corresponding page in the table and return an
    /// `Option` containing the result. Note that a return value of `None` implies the requested
    /// page has yet to be entered into the page table and is more aptly defined as a cache miss.
    ///
    /// # Arguments
    ///
    /// * `id` - an unsigned integer value representing the requested page number.
    ///
    fn find(&self, id: usize) -> Option<&Page> {
        self.0.get(&id)
    }

    /// The behavior of `find_mut` is identical to that of the `find` method with the only
    /// exception being that it returns an option with a mutable reference to the page number when
    /// the `Some` variant is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - an unsigned integer value representing the requested page number.
    ///
    fn find_mut(&mut self, id: usize) -> Option<&mut Page> {
        self.0.get_mut(&id)
    }

    /// Insert a new element into the page table. Note that this page will never be removed from
    /// the table relative to the virtual memory simulation as it contains only logical values
    /// (references). Such is the case with most page table implementations since the size taken up
    /// by the table is insignificant in comparison to the amount of data the table is used to
    /// reference by way of virtual memory.
    ///
    /// # Arguments
    ///
    /// * `id` - an unsigned integer value representing the requested page number.
    /// * `page` - A `Page` instance containing information about the physical in-memory frame.
    ///
    fn insert(&mut self, id: usize, page: Page) {
        self.0.insert(id, page);
    }

    fn as_vec(&self) -> Vec<(&usize, &Page)> {
        let mut holder = self.0.iter().collect::<Vec<_>>();
        holder.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        holder
    }
}

impl ToString for PageTable {
    fn to_string(&self) -> String {
        let mut string = String::new();
        self.as_vec()
            .iter()
            .for_each(|x| string.push_str(format!("{:?}\n", x).as_str()));
        string
    }
}

/// The `Frame` struct contains a buffer with a length defined as the frame size in bytes (`u8`).
/// It is intended to be the simplest element of the `FrameTable` and represents memory that can be
/// swapped in and out via demand paging. An associated `page_id` element is kept simply for record
/// keeping and to minimize the effort required to invalidate the corresponding entry in the page
/// table when a frame is victimized (paged-out).
struct Frame {
    buffer: Vec<u8>,
    associated_page_id: usize,
}

impl Frame {
    /// Create a new instance of the `Frame` struct with a buffer size reflecting the specified
    /// frame size.
    ///
    /// # Arguments
    ///
    /// * `frame_size` - an unsigned integer representing the sized of the frame in bytes (`u8`).
    ///
    fn new(frame_size: u64) -> Self {
        Self {
            buffer: vec![0 as u8; frame_size as usize],
            associated_page_id: usize::MAX,
        }
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

/// The `FrameTable` struct is used to simulate the behavior of physical memory frames relative to
/// the operating system. Although a `PageTable` may possess a seemingly infinite number of pages,
/// a `FrameTable` is limited to a finite amount to mimic the characteristics of real physical
/// memory. Here, the benefits of virtual memory begin to unfold as processes naively see a bottomless pool
/// of possible allocations due to the size possible with virtual address spaces.
///
/// Instances of the `FrameTable` struct are predominantly buffers containing references to other
/// buffers (frames). Additional elements within the struct exist merely for housekeeping or for
/// the sake of the victimization algorithm responsible for ensuring continued allocation
/// operations at the expense of infrequently used chunks of memory.
struct FrameTable {
    table_size: usize,
    frame_size: u64,
    entries: Vec<Frame>,
    victimizer: LinkedHashMap<usize, usize>,
}

impl FrameTable {
    /// Provided sizes for the table and associated memory frames, construct a new `FrameTable`
    /// instance.
    ///
    /// # Arguments
    ///
    /// * `table_size` - an unsigned integer which specifies the size of the frame table.
    /// * `frame_size` - an unsigned integer used to set the size of all frames within the table.
    ///
    fn build(table_size: usize, frame_size: u64) -> Self {
        let mut entries: Vec<Frame> = Vec::with_capacity(table_size);
        let mut victimizer = LinkedHashMap::new();
        (0..table_size).for_each(|index| {
            entries.push(Frame::new(frame_size));
            victimizer.insert(index, index);
        });

        Self {
            frame_size,
            table_size,
            entries,
            victimizer,
        }
    }

    fn victimizer_vector(&self) -> Vec<&usize> {
        self.victimizer.iter().map(|(x, _)| x).collect()
    }

    /// Instruct the frame table to allocate a free frame regardless of whether one is available.
    /// Should it be the case that no free frames are available, the victimization algorithm is
    /// used to select an allocated frame to be replaced with new data. In other words, the victim
    /// frame is paged-out. Often in consumer computer systems, the victim frame's data is moved to
    /// swap space assuming the system is configured to use it. Although significantly slower,
    /// there are still merits to using a system-managed raw partition relative to the virtual
    /// memory implementation.
    ///
    fn allocate(&mut self) -> usize {
        let value = self
            .victimizer
            .pop_front()
            .expect("should have victims at the ready")
            .0;
        self.victimizer.insert(value, value);
        value
    }

    fn reference(&mut self, index: usize) {
        self.victimizer.remove(&index).unwrap();
        self.victimizer.insert(index, index);
    }
}

/// The `VirtualMemory` struct is the culmination of all other structures and procedures in this
/// module. The core purpose of each instance is to simulate the actions of a virtual memory system
/// with only a modest amount of configuration. Ideally, it also should behave as a standard
/// testing system for different algorithms with minor reconfiguration to the struct definition and
/// initializer function calls.
pub struct VirtualMemory {
    tlb: TLB,
    pages: PageTable,
    frames: FrameTable,
    storage: Storage,
    pub tracker: Tracker,
}

impl VirtualMemory {
    /// Provided the nessecary arguments, create a new `VirtualMemory` instance.
    ///
    /// # Arguments
    ///
    /// * `tlb_size` - size of the TLB cache.
    /// * `frame_table_size` - number of entries in the simulated frame table (physical memory)
    /// * `frame_size` - size of each simulated frame of physical memory
    ///
    pub fn build(tlb_size: usize, frame_table_size: usize, frame_size: u64, file_storage: &str) -> Self {
        Self {
            tlb: TLB::build(tlb_size),
            pages: PageTable::build(),
            frames: FrameTable::build(frame_table_size, frame_size),
            storage: Storage::build(file_storage),
            tracker: Tracker::new(),
        }
    }

    /// Using the simulated virtual memory system, access the data stored at the provided logical
    /// address and return the value to the caller. Along the way a series of hit/miss stats are
    /// recorded for analysis of algorithmic performance. Note that performance is directly related
    /// to the implementation employed as well as the nature of the overall collection of requests
    /// made over the lifetime of the instance. Regarding the latter, if the address requests are
    /// randomly generated then there is little hope in having meaningful performance at any cache
    /// level. On the other hand, if the access requests are more sequential in nature such as a
    /// sequential read of bytes or programmatic instructions, then the performance gains will be
    /// more noticable.
    ///
    /// # Arguments
    ///
    /// * `virtual_address` - the process facing logical address used for indirect data access
    ///
    /// # Errors
    ///
    /// Errors will occur if an invalid frame retrieval request is generated (e.g. accessing a
    /// frame number greater than the possible number of entries in the table).
    ///
    pub fn access(&mut self, virtual_address: VirtualAddress) -> Result<AccessResult> {
        self.tracker.attempted_memory_accesses += 1;
        let page_number = virtual_address.number_page as usize;
        let offset = virtual_address.number_offset as usize;
        let frame_index = match self.tlb.find(page_number) {
            Some(x) => {
                self.tracker.tlb_hits += 1;
                *x
            }
            _ => match self.pages.find(page_number) {
                Some(page) if page.valid => {
                    self.tracker.page_hits += 1;
                    self.tlb.cache_element(page_number, page.frame_index);
                    page.frame_index
                }
                _ => {
                    let fi = self.retrieve_frame(virtual_address.number_page as usize)?;
                    self.tlb.cache_element(page_number, fi);
                    fi
                }
            },
        };

        self.frames.reference(frame_index);
        Ok(AccessResult {
            virtual_address,
            physical_address: ((frame_index * self.frames.frame_size as usize) + offset) as u32,
            value: self.frames.entries[frame_index][offset] as i8,
        })
    }

    /// Provided a logical page number, allocate a free frame and read the data referenced by the
    /// page into the frame buffer to maintain the illusion of unmanaged memory access from the
    /// perspective of the process.
    ///
    /// # Arguments
    ///
    /// * `page_number` - an unsigned integer representing the number of the requested page.
    ///
    /// # Errors
    ///
    /// An error will occur if the storage read operation is passed invalid arguments (e.g. reading
    /// past the end of the simulated backing store). The error value is returned to the caller in
    /// the form of the `Error` enum variant.
    ///
    fn retrieve_frame(&mut self, page_number: usize) -> Result<usize> {
        let frame_index = self.frames.allocate();
        let frame = &mut self.frames.entries[frame_index];
        if let Some(page) = self.pages.find_mut(frame.associated_page_id) {
            page.valid = false;
            if self.tlb.flush_element(frame.associated_page_id) {
                self.tracker.tlb_flushes += 1;
            }
        }
        frame.associated_page_id = page_number;
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

    const SIZE_FRAME: u64 = 256;
    const SIZE_TABLE: usize = 256;

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
    mod page_table_tests {
        use super::*;

        fn make_standard_table() -> PageTable {
            let mut table = PageTable::build();

            (0..10).for_each(|x| {
                table.insert(
                    x,
                    Page {
                        frame_index: x,
                        valid: true,
                    },
                )
            });
            table
        }

        // module level tests

        /// Ensure the page table is built to expectations.
        #[test]
        fn build() {
            // arrange
            let table = PageTable::build();

            // assert
            assert!(table.0.len() == 0)
        }

        #[test]
        fn find() {
            // arrange
            let table = make_standard_table();
            let range_max = 10;

            (0..range_max).for_each(|x| {
                let page: &Page = table.find(x).unwrap();
                // assert
                assert_eq!(x, page.frame_index);
            });

            assert_eq!(table.find(range_max + 1), None);
        }

        #[test]
        fn find_mut() {
            // arrange
            let mut table = make_standard_table();
            let range_max = 10;

            (0..range_max).for_each(|mut x| {
                let page: &mut Page = table.find_mut(x).unwrap();
                // assert
                assert_eq!(&mut x, &mut page.frame_index);
            });

            assert_eq!(table.find(range_max + 1), None);
        }

        #[test]
        fn insert() {
            // arrange
            let mut table = make_standard_table();
            println!("current table:");
            println!("{}", table.to_string());
            let page_id = 5;
            let frame_index = 55;
            let new_page = Page {
                frame_index,
                valid: true,
            };

            // act
            table.insert(page_id, new_page);
            println!("modified table:");
            println!("{}", table.to_string());

            // assert
            assert_eq!(
                table.find(page_id),
                Some(&Page {
                    frame_index,
                    valid: true
                })
            );
        }
    }

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
    mod frame_table_tests {

        use super::*;
        const TEST_TABLE_SIZE: usize = 4;
        const TEST_FRAME_SIZE: u64 = 64;

        fn make_standard_table() -> FrameTable {
            let mut table = FrameTable::build(TEST_TABLE_SIZE, TEST_FRAME_SIZE);

            (0..TEST_TABLE_SIZE).for_each(|x| {
                let frame_number = table.allocate();
                let frame = &mut table.entries[frame_number];
                frame.associated_page_id = x;
                frame[0] = x as u8;
            });
            table
        }

        #[test]
        fn new() {
            let ft = make_standard_table();
            assert_eq!(ft.entries.len(), TEST_TABLE_SIZE);
            assert_eq!(ft.table_size, TEST_TABLE_SIZE);
            assert_eq!(ft.frame_size, TEST_FRAME_SIZE);
        }

        #[test]
        fn allocate() {
            // arrange

            // act

            // assert
            todo!();
        }

        #[test]
        fn reaper() {
            // arrange

            // act

            // assert
            todo!();
        }
    }

    #[cfg(test)]
    mod tlb_tests {

        use super::*;
        const SIZE_TEST: usize = 3;

        #[test]
        fn build() {
            let tlb = TLB::build(SIZE_TEST);
            assert_eq!(tlb.map.len(), 0);
            assert_eq!(tlb.table_size, SIZE_TEST);
        }

        #[test]
        fn find_and_replace() {
            let mut tlb = TLB::build(SIZE_TEST);
            let min = 0;
            let max = 5;

            (min..max).for_each(|x| {
                assert!(tlb.find(x).is_none());
                tlb.cache_element(x, x);
                assert!(tlb.find(x).is_some());
            });

            assert!(tlb.find(max).is_none());
        }
    }
}

// TODO: ensure tests are in place that ensure the functionality of the TLB replacement and frame
// replacement algorithms.
//
// TODO: need to write test that ensures the alignment of the TLB with the page table.
