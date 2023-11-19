use clap::Parser;
use std::env;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(long, default_value_t =  env_or_default_str("SIM_FILE_STORAGE", "BACKING_STORE.bin"))]
    file_storage: String,
    #[arg(long, default_value_t =  env_or_default_str("SIM_FILE_VALIDATION", "correct.txt"))]
    file_validation: String,

    #[arg(long, default_value_t =  env_or_default_str("SIM_FILE_ADDRESS", "addresses.txt"))]
    file_address: String,

    #[arg(long, default_value_t = env_or_default_u32("SIM_SIZE_TABLE", 64))]
    size_table: u32,

    #[arg(long, default_value_t =  env_or_default_u32("SIM_SIZE_TLB", 16))]
    size_tlb: u32,

    #[arg(long, default_value_t = env_or_default_u32("SIM_SIZE_FRAME", 256))]
    size_frame: u32,
}

impl Config {
    fn validate(&self) {
        if self.size_tlb == 0 || self.size_tlb > self.size_table {
            eprintln!("'size_tlb' must be a non-zero value less than 'size_table'");
            process::exit(1);
        } else if f64::from(self.size_frame).log2().fract() != 0.0 {
            eprintln!("'size_frame' must be a non-zero power of 2 integer value");
            process::exit(1);
        }
    }
}

fn env_or_default_str(varname: &str, default: &str) -> String {
    match env::var(varname) {
        Ok(val) => val,
        _ => String::from(default),
    }
}

fn env_or_default_u32(varname: &str, default: u32) -> u32 {
    match env::var(varname) {
        Ok(val) => val
            .parse()
            .expect(&format!("expected unsigned int for env var: '{}'", varname)),
        _ => default,
    }
}
