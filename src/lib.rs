#![allow(dead_code, unused_imports)]
pub mod address;
pub mod config;
pub mod storage;
pub mod tracker;
pub mod validator;
pub mod virtual_memory;

use address::AddressReader;
use config::Config;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::{process, thread, time::Duration};
use validator::ValidationReader;
use virtual_memory::VirtualMemory;

const MASK_PAGE: u32 = 0x0000FF00;
const MASK_OFFSET: u32 = 0x000000FF;

/// A structure containing the core elements required to run a simulation.
pub struct Simulation {
    virtual_memory: VirtualMemory,
    address_reader: AddressReader,
    validation_reader: ValidationReader,
}

impl Simulation {
    pub fn build(config: &Config) -> Self {
        Self {
            address_reader: AddressReader::new(&config.file_address),
            validation_reader: ValidationReader::new(&config.file_validation),
            virtual_memory: VirtualMemory::build(
                config.size_tlb as usize,
                config.size_table as usize,
                config.size_frame as u64,
                &config.file_storage,
            ),
        }
    }
}

fn run_simulation(config: Config) {
    let Simulation {
        address_reader,
        validation_reader,
        mut virtual_memory,
    } = Simulation::build(&config);

    let num_records = AddressReader::new(&config.file_address).count() as u64;
    let pb = ProgressBar::new(num_records);
    pb.set_style(ProgressStyle::with_template("running simulation: {spinner}").unwrap());
    for (i, (virtual_address, validation_entry)) in
        address_reader.zip(validation_reader).enumerate()
    {
        let access_result = virtual_memory.access(virtual_address).unwrap();
        match access_result == validation_entry {
            true => virtual_memory.tracker.correct_memory_accesses += 1,
            false => {
                println!("failure occurred on record: {i:05}");
                println!("--------------------------------");
                println!("expected: {validation_entry:?}");
                println!("received: {access_result:?}");
            }
        }
        pb.inc(1);
        thread::sleep(Duration::from_micros(250));
    }
    println!("{}", virtual_memory.tracker);
    let tracker = &virtual_memory.tracker;
    if tracker.attempted_memory_accesses != tracker.correct_memory_accesses {
        process::exit(2)
    }
}

pub fn runner(config: Config) {
    run_simulation(config)
}
