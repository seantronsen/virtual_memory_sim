#![allow(dead_code, unused_imports)]
mod address;
mod backing_store;
mod table;
mod validator;

use address::AddressReader;
use backing_store::BackingStore;
use table::VirtualMemory;

const FILENAME_BSTORE: &str = "BACKING_STORE.bin";
const FILENAME_VALIDATION: &str = "correct.txt";
const FILENAME_ADDRESS: &str = "addresses.txt";
const SIZE_TABLE: usize = 256;
const SIZE_FRAME: u64 = 256;
const MASK_PAGE: u32 = 0x0000FF00;
const MASK_OFFSET: u32 = 0x000000FF;

/// A structure which contains the core elements required to run a simulation.
pub struct Simulation {
    virtual_memory: VirtualMemory,
    address_reader: AddressReader,
}

impl Simulation {
    pub fn build(num_pages: usize, num_frames: usize, frame_size: u64) -> Self {
        Self {
            address_reader: AddressReader::new(),
            virtual_memory: VirtualMemory::build(num_pages, num_frames, frame_size),
        }
    }
}

// function header for use later
fn prepare_simulation() -> Simulation {
    Simulation::build(SIZE_TABLE, SIZE_TABLE, SIZE_FRAME)
}

fn run_simulation(simulation: Simulation) {
    let Simulation {
        address_reader,
        mut virtual_memory,
    } = simulation;

    for virtual_address in address_reader {
        let byte = virtual_memory.access(virtual_address).unwrap();
        println!("byte value: {}", byte);
    }
}

pub fn runner() {
    run_simulation(prepare_simulation())
}
