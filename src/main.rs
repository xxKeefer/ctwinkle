mod app;
mod cli;
mod field;
mod input;
mod palette;
mod render;
mod term;

use clap::Parser;
use crossterm::{event, terminal};
use field::{Field, Mode};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use std::io::{self, BufWriter};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const DENSITY: u8 = 5;

fn main() -> io::Result<()> {
    let args = cli::Args::parse();

    let stop = Arc::new(AtomicBool::new(false));
    for sig in [SIGTERM, SIGHUP, SIGINT] {
        signal_hook::flag::register(sig, Arc::clone(&stop))?;
    }

    let (w, h) = terminal::size()?;
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_nanos() as u64);
    let mut field = Field::new(w, h, DENSITY, seed);
    if args.rain {
        field.set_mode(Mode::Rain);
    }

    let mut guard = term::Guard::new(BufWriter::new(io::stdout()))?;
    terminal::enable_raw_mode()?;
    app::run(guard.out(), &mut field, args.palette(), &stop, |timeout| {
        if event::poll(timeout)? {
            event::read().map(Some)
        } else {
            Ok(None)
        }
    })
}
