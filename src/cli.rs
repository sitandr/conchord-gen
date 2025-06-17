#![feature(iter_intersperse)]
mod search;

use search::{search_chord, Tuning};

fn main() -> Result<(), String> {
    // TODO: use clap
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        return Err("Write a chord name".to_string());
    }

    let tune_string = if args.len() == 2 {
        "E1 A1 D2 G2 B2 E3"
    } else {
        &args[2]
    };

    let tuning = Tuning::try_from_str(tune_string)?;
    let strings = search_chord(&tuning, &args[1], u8::MAX, true)?;

    for s in strings {
        println!("{s}");
    }

    Ok(())
}
