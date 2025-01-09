#![feature(iter_intersperse)]

mod search;
use search::{search_chord, Tuning};
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
fn get_chords(tuning: &[u8], name: &[u8], shift: &[u8]) -> Result<Vec<u8>, String> {
    let tuning = std::str::from_utf8(tuning).unwrap();
    let name = std::str::from_utf8(name).unwrap();
    let shift = shift[0] as u8;
    let strings = search_chord(&Tuning::from_str(tuning), &name, shift)?;

    Ok(strings.join(";").as_bytes().to_vec())
}
