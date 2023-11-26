# virtual_memory_sim

A virtual memory simulation created for educational purposes using the Rust
programming language and a few packages from `crates.io`. The program was
created to simulate standard operating system behavior where a logical
address space is provided in lieu of the physical resources available to a
system. Like the real version, demand paging is used to keep only necessary
data in memory and infrequently referenced elements risk being paged out.

While this project was created to aid in my learning of some operating system
concepts, it can also be used to test virtual memory page victimization
algorithms with minor modifications.

## Requirements

- Cargo version $\approx$ 1.73.0

## Getting Started

Getting started with the project is trivial thanks to the suite of tools
provided by Rust's `cargo`. After cloning the repository, the simulation
can be compiled and run by issuing the command

```bash

cargo run

```

within the project directory.

**Example**

```text

virtual memory simulation
simulation configuration values:
Config {
    file_storage: "BACKING_STORE.bin",
    file_validation: "correct.txt",
    file_address: "addresses.txt",
    size_table: 64,
    size_tlb: 16,
    size_frame: 256,
    delay_us: 250,
}

running simulation: ⠒
Stats Tracked
---------------------------------
page_hits:                00000183
tlb_hits:                 00000063
tlb_flushes:              00000170
attempted_memory_acceses: 00001000
correct_memory_accesses:  00001000


tlb hit ratio:            0.063000
page hit ratio:           0.183000

```

> Note: The cache hit ratios listed above are poor due to the sequence of
> memory accesses employed (`addresses.txt`) was generated randomly. Better
> results can be found in practice by using data with better temporal locality
> (e.g. a program).

### CLI Options

To see all available options which modify
runtime behavior, issue the command

```bash

cargo run -- --help

```

to have them written to `stdout`. Note that any of the options displayed can
also be specified as environment variables (consult `config.rs` for details).

**Example**

```text

virtual memory simulation
Usage: virtual_memory_sim [OPTIONS]

Options:
      --file-storage <FILE_STORAGE>        [default: BACKING_STORE.bin]
      --file-validation <FILE_VALIDATION>  [default: correct.txt]
      --file-address <FILE_ADDRESS>        [default: addresses.txt]
      --size-table <SIZE_TABLE>            [default: 64]
      --size-tlb <SIZE_TLB>                [default: 16]
      --size-frame <SIZE_FRAME>            [default: 256]
      --delay-us <DELAY_US>                [default: 250]
  -h, --help                               Print help
  -V, --version                            Print version

```
