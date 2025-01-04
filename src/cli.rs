#![feature(iter_intersperse)]
mod search;

use search::{Tuning, search_chord};

fn main() -> Result<(), String> {
    // TODO: use clap
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        println!("Write a chord name");
    }
    
    let tune_string = if args.len() == 2 {"E A D G B E"} else {&args[2]};

    let strings = search_chord(&Tuning::from_str(tune_string), &args[1], u8::MAX)?;

    for s in strings {
        println!("{s}");
    }

    Ok(())
}