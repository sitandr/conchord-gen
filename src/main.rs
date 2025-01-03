use klib::core::base::Parsable;
use klib::core::note::*;
use klib::core::chord::*;
use klib::core::octave::HasOctave;
use klib::core::octave::Octave;
use klib::core::pitch::HasPitch;

struct Tuning {
    notes: Vec<u8>
}

const FRET_RANGE: u8 = 5;

impl Tuning {
    fn new(notes: &Vec<Note>) -> Self {
        Self{notes: notes.iter().map(note_to_pitch).collect()}
    }

    #[inline]
    fn strings(&self) -> usize {
        self.notes.len()
    }

    fn find_chord(&self, chord: Vec<Note>) -> Vec<FoundChord> {
        let chord: Vec<u8> = chord.iter().map(note_to_pitch).collect();

        //for shift in 0..11 {}
        self.find_chord_with_shift(chord, 0)
    }

    fn find_chord_with_shift(&self, chord: Vec<u8>, shift: u8) -> Vec<FoundChord> {
        let first_note = chord.first().expect("Empty chord");

        let mut collected = vec![];
        for first_string in 0..self.strings() {
            let base = self.notes[first_string] + shift;
            for possible in base..base+FRET_RANGE {
                if possible % 12 == *first_note {
                    let mut left = chord.clone();
                    left.remove(0);
                    let found = self.find_chord_from_string(&chord, left, first_string + 1, shift);
                    collected.extend(found.extend_all((self.strings() - first_string, possible - base)));
                }
            }
        }
        collected
    }

    fn find_chord_from_string(&self, chord: &[u8], left: Vec<u8>, start_string: usize, shift: u8) -> Vec<FoundChord> {
        if start_string == self.strings() && left.len() == 0 {
            return vec![FoundChord{hold: Vec::with_capacity(self.strings())}]
        }

        let mut collected = vec![];
        for first_string in start_string..self.strings() {
            let base = self.notes[first_string] + shift;
            for possible in base..base+FRET_RANGE {
                let possible_m = possible % 12;
                if chord.contains(&possible_m) {
                    let mut left = left.clone();
                    left.retain(|&x| x != possible_m);
                    let found = self.find_chord_from_string(chord, left, first_string + 1, shift);
                    // println!("{}", start_string);
                    collected.extend(found.extend_all((self.strings() - first_string, possible - base)));
                }
            }
        }

        collected
    }
}

#[derive(Debug)]
struct FoundChord {
    hold: Vec<(usize, u8)>
}

trait ExtendAll<T> {
    fn extend_all(self, item: T) -> Self;
}

impl ExtendAll<(usize, u8)> for Vec<FoundChord> {
    fn extend_all(mut self, item: (usize, u8)) -> Self {
        self.iter_mut().for_each(|c| c.hold.push(item));
        self
    }
} 

fn note_to_pitch(n: &Note) -> u8 {
    // SAFETY: Pitch is `repr(u8)`
    let note_pitch: u8 = unsafe { *<*const _>::from(&n.pitch()).cast::<u8>() };
    note_pitch
}

fn main() {
    println!("Hello, world!");
    // Parse a chord from a string, and inspect the scale.
    // let chord = Chord::parse("Cm7b5").unwrap().chord();
    println!("{:?}", Chord::parse("Bm7b5").unwrap().chord());

    let tune_string = "E A D G B E";
    let tune_vec: Vec<_> = tune_string.split(' ').map(Note::parse).map(|s| s.unwrap()).collect();
    let tuning = Tuning::new(&tune_vec);

    println!("{:?}", tuning.find_chord(Chord::parse("Am").unwrap().chord()));
}

