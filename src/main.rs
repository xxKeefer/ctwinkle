mod cli;
// Wired into the run loop in step 3.
#[allow(dead_code)]
mod field;
#[allow(dead_code)]
mod render;

use clap::Parser;

fn main() {
    let _args = cli::Args::parse();
}
