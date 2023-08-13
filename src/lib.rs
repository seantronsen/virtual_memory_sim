#![allow(dead_code, unused_imports)]
mod address;
mod backing_store;
mod stattrack;
mod table;
mod validator;

use address::AddressReader;
use backing_store::BackingStore;
use table::VirtualMemory;
use validator::ValidationReader;

const FILENAME_BSTORE: &str = "BACKING_STORE.bin";
const FILENAME_VALIDATION: &str = "correct.txt";
const FILENAME_ADDRESS: &str = "addresses.txt";
const SIZE_TABLE: usize = 256;
const SIZE_TLB: usize = 16;
const SIZE_FRAME: u64 = 256;
const MASK_PAGE: u32 = 0x0000FF00;
const MASK_OFFSET: u32 = 0x000000FF;

/// A structure which contains the core elements required to run a simulation.
pub struct Simulation {
    virtual_memory: VirtualMemory,
    address_reader: AddressReader,
    validation_reader: ValidationReader,
}

impl Simulation {
    pub fn build(tlb_size: usize, num_pages: usize, num_frames: usize, frame_size: u64) -> Self {
        Self {
            address_reader: AddressReader::new(),
            validation_reader: ValidationReader::new(),
            virtual_memory: VirtualMemory::build(tlb_size, num_pages, num_frames, frame_size),
        }
    }
}

// function header for use later
fn prepare_simulation() -> Simulation {
    Simulation::build(SIZE_TLB, SIZE_TABLE, SIZE_TABLE, SIZE_FRAME)
}

fn run_simulation(simulation: Simulation) {
    let Simulation {
        address_reader,
        validation_reader,
        mut virtual_memory,
    } = simulation;

    let mut stattracker = stattrack::StatTracker::new();

    for (virtual_address, validation_entry) in address_reader.zip(validation_reader) {
        let access_result = virtual_memory
            .access(virtual_address, &mut stattracker)
            .unwrap();
        match access_result == validation_entry {
            true => stattracker.correct_memory_accesses += 1,
            false => {
                println!("expected: {:?}", validation_entry);
                println!("received: {:?}", access_result);
            }
        }
    }
    println!("{}", stattracker);
}

pub fn runner() {
    run_simulation(prepare_simulation())
}
