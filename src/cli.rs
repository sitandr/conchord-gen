#![feature(iter_intersperse)]
mod search;
use search::*;

fn main() -> Result<(), String> {
    // TODO: use clap
    let tune_string = "E A D G B E";

    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        println!("Write a chord name")
    }
    let strings = search_chord(Tuning::from_str(&tune_string), &args[1])?;

    for s in strings {
        println!("{}", s)
    }

    Ok(())
}