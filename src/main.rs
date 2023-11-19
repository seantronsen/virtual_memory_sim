use clap::Parser;
use virtual_memory_sim::config::{self, Config};
use virtual_memory_sim::runner;

fn welcome() {
    println!("virtual memory simulation");
}

fn main() {
    welcome();

    // TODO: parse env vars for potential configuration
    // then output the config values (e.g. table sizes)
    // used to run the simulation.
    let config = Config::parse();
    println!("{:#?}", config);
    // execute runner
    //println!("running simulation");
    //runner();
}
