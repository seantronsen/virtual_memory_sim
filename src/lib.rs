#![allow(dead_code, unused_imports)]
mod address;
mod backing_store;
mod table;

const BACKING_STORE_FILENAME: &str = "BACKING_STORE.bin";
const ADDRESS_FILE: &str = "addresses.txt";
const TABLE_SIZE: usize = 256;
const FRAME_SIZE: usize = 256;
const MASK_PAGE: u32 = 0x0000FF00;
const MASK_OFFSET: u32 = 0x000000FF;

enum Error {}
type Result<T> = std::result::Result<T, Error>;

/// A structure which contains the core elements required to run a simulation.
pub struct Simulation {
    pub page_table: table::PageTable,
    frame_table: table::FrameTable,
    address_reader: address::AddressReader,
    backing_store: backing_store::BackingStore,
}

impl Simulation {
    pub fn build(num_pages: usize, num_frames: usize, frame_size: usize) -> Self {
        let page_table = table::PageTable::build(num_pages);
        let frame_table = table::FrameTable::build(num_frames, frame_size);

        Self {
            page_table,
            frame_table,
            address_reader: address::AddressReader::new(),
            backing_store: backing_store::BackingStore::build(),
        }
    }
}

fn prepare_simulation() -> Simulation {
    // function header for use later
    Simulation::build(TABLE_SIZE, TABLE_SIZE, FRAME_SIZE)
}

fn run_simulation(simulation: Simulation) {
    let Simulation {
        mut page_table,
        mut frame_table,
        mut address_reader,
        mut backing_store,
    } = simulation;

    for _ in 0..5 {
        let logical_address = address_reader.next().unwrap();
        let page = &mut page_table[logical_address.number_page as usize];

        // only allows for valid page valid frame or the opposite
        let byte_value = match page.valid {
            true => {
                let frame = &frame_table[page.frame_index as usize];
                frame.buffer[logical_address.number_offset as usize]
            }
            false => {
                let new_frame_index = frame_table.obtain_available_index().unwrap();
                let frame = &mut frame_table[new_frame_index];
                backing_store
                    .read_frame(logical_address.number_page as u64, frame)
                    .unwrap();
                frame.valid = true;
                page.valid = true;
                frame.buffer[logical_address.number_offset as usize]
            }
        };
        // println!("byte value: {:#04x}", byte_value);
        println!("byte value: {}", byte_value);
    }

    // do some stuff
}

pub fn runner() {
    run_simulation(prepare_simulation())
}
