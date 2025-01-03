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
        let first_note_oct_part: i16 = octave_part(*first_note).into();

        let mut collected = vec![];
        for first_string in 0..self.strings() {
            let base = self.notes[first_string] + shift;
            for possible in base..base+FRET_RANGE {
                if same_note(possible, *first_note) {
                    let this_oct: i16 = octave_part(possible).into();
                    let note_shift: i16 = this_oct - first_note_oct_part;

                    let shifted_chord: Vec<_> = chord.iter().map(|&n| (n as i16 + note_shift) as u8).collect();
                    let found = self.find_chord_from_string(&shifted_chord[1..], first_string + 1, shift);
                    collected.extend(found.extend_all((first_string, possible - base)));
                }
            }
        }
        collected
    }

    fn find_chord_from_string(&self, shifted_chord: &[u8], start_string: usize, shift: u8) -> Vec<FoundChord> {
        if start_string == self.strings() {
            return vec![FoundChord{hold: Vec::with_capacity(self.strings())}]
        }

        let mut collected = vec![];
        for first_string in start_string..self.strings() {
            let base = self.notes[first_string] + shift;
            for possible in base..base+FRET_RANGE {
                if possible == shifted_chord[0] {
                    let found = self.find_chord_from_string(&shifted_chord[1..], start_string + 1, shift);
                    collected.extend(found.extend_all((first_string, possible - base)));
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

#[inline]
fn same_note(n1: u8, n2: u8) -> bool {
    n1 % 12 == n2 % 12 
}

fn octave_part(n: u8) -> u8 {
    (n / 12) * 12
}

fn note_to_pitch(n: &Note) -> u8 {
    // SAFETY: Pitch and Octave are `repr(u8)`
    let note_pitch: u8 = unsafe { *<*const _>::from(&n.pitch()).cast::<u8>() };
    let octave: u8 = unsafe { *<*const _>::from(&n.octave()).cast::<u8>() };
    // No overflow: max is `15*12 + 11 = 191 < 255``
    note_pitch + octave * 12
}

fn main() {
    println!("Hello, world!");
    // Parse a chord from a string, and inspect the scale.
    // let chord = Chord::parse("Cm7b5").unwrap().chord();
    println!("{:?}", Chord::parse("Bm7b5").unwrap().chord());

    let tune_string = "E4 A4 D5 G5 B5 E6";
    let tune_vec: Vec<_> = tune_string.split(' ').map(|s| {
        Note::parse(&s[0..1]).unwrap().with_octave(Octave::try_from(s[1..].parse::<u8>().unwrap()).unwrap())
    }).collect();
    let tuning = Tuning::new(&tune_vec);
    println!("Searching for {:?}", Chord::parse("Dm").unwrap().chord());

    println!("{:?}", tuning.find_chord(Chord::parse("Dm").unwrap().chord()));
}

