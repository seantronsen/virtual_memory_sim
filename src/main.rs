use clap::Parser;
use virtual_memory_sim::config::Config;
use virtual_memory_sim::runner;

fn init_msg() {
    println!("virtual memory simulation");
}

fn main() {
    init_msg();
    let config = Config::parse();
    config.display();
    config.validate();
    println!();
    runner(config);
}
