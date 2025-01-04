use std::fmt::Display;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::u8;

use klib::core::base::Parsable;
use klib::core::interval::Interval;
use klib::core::known_chord::HasRelativeChord;
use klib::core::known_chord::HasRelativeScale;
use klib::core::note::*;
use klib::core::chord::*;
use klib::core::pitch::HasPitch;
use ordered_float::NotNan;

pub struct Tuning {
    notes: Vec<u8>
}

const FRET_RANGE: u8 = 5;

impl Tuning {
    fn new(notes: &Vec<Note>) -> Self {
        Self{notes: notes.iter().map(note_to_pitch).collect()}
    }

    pub fn from_str(s: &str) -> Self {
        let tune_vec: Vec<_> = s.split(' ').map(Note::parse).map(|s| s.unwrap()).collect();
        Tuning::new(&tune_vec)
    }

    #[inline]
    fn strings(&self) -> usize {
        self.notes.len()
    }

    fn find_chord(&self, chord: Vec<Note>, optional_fifth: u8) -> Vec<FoundChord> {
        let chord: Vec<u8> = chord.iter().map(note_to_pitch).collect();

        (0..11).flat_map(|s| self.find_chord_with_shift(&chord, s, optional_fifth).into_iter()).collect()
    }

    /// optional_fifth: from 0 to 12, if specified, this note could be ommited 
    fn find_chord_with_shift(&self, chord: &[u8], shift: u8, optional_fifth: u8) -> Vec<FoundChord> {
        let first_note = chord.first().expect("Empty chord");

        let mut collected = vec![];
        for first_string in 0..self.strings() {
            let base = self.notes[first_string] + shift;
            for possible in base..base+FRET_RANGE {
                if possible % 12 == *first_note {
                    let mut left = chord.to_vec();
                    left.remove(0);
                    let found = self.find_chord_from_string(&chord, left, first_string + 1, shift, optional_fifth);
                    collected.extend(found.extend_all((first_string, possible - base + shift)));
                }
            }
        }
        collected
    }

    // TODO: Gmaj7 dosn't give 324003

    fn find_chord_from_string(&self, chord: &[u8], left: Vec<u8>, start_string: usize, shift: u8, optional_fifth: u8) -> Vec<FoundChord> {
        if start_string == self.strings() {
            
            if left.len() == 0 {
                return vec![FoundChord{hold: Vec::with_capacity(self.strings()), no_fifth: false}]
            }
            else if left.len() == 1 && left[0] == optional_fifth {
                return vec![FoundChord{hold: Vec::with_capacity(self.strings()), no_fifth: true}]
            }
        }

        let mut collected = vec![];
        for first_string in start_string..self.strings() {
            let base = self.notes[first_string] + shift;
            for possible in base..base+FRET_RANGE {
                let possible_m = possible % 12;
                if chord.contains(&possible_m) {
                    let mut left = left.clone();
                    left.retain(|&x| x != possible_m);
                    let found = self.find_chord_from_string(chord, left, first_string + 1, shift, optional_fifth);
                    // println!("{}", start_string);
                    collected.extend(found.extend_all((first_string, possible - base + shift)));
                }
            }
        }

        collected
    }
}

#[derive(Debug, PartialEq)]
struct FoundChord {
    hold: Vec<(usize, u8)>,
    no_fifth: bool
}

pub struct FormattedChord {
    v: Vec<Option<u8>>,
    no_fifth: bool
}

impl FoundChord {
    fn format(&self, strings: usize) -> FormattedChord {
        let mut v = vec![None; strings];
        for holded in &self.hold {
            assert_eq!(v[holded.0], None, "Dublicated strings: {self:?}?");
            v[holded.0] = Some(holded.1);
        }

        FormattedChord{v, no_fifth: self.no_fifth}
    }

    #[allow(dead_code)]
    fn to_string(&self, strings: usize) -> String {
        self.format(strings).to_string()
    }   
}

impl FormattedChord {
    fn to_string(&self) -> String {
        if self.v.iter().all(|s| if let &Some(t) = s {t < 10} else {true}) {
            self.v.iter().map(|t| if let Some(t) = t {t.to_string()} else {"x".to_string()}).collect()
        } else {
            self.v.iter().map(|t| if let Some(t) = t {t.to_string()} else {"x".to_string()}).intersperse(", ".to_string()).collect()
        }
    }

    fn evaluate_complexity(&self) -> Option<NotNan<f32>> {
        let deaf = self.v.iter().filter(|e| e.is_none()).count();
        let upper_deaf = self.v.iter().take_while(|e| e.is_none()).count();
        let under_deaf = deaf - upper_deaf;

        let quality = upper_deaf as f32 * 0.3 + under_deaf as f32 * 1.2;

        let open = self.v.iter().filter(|&&e| e == Some(0)).count();
        let open_up = self.v.iter().take_while(|&&e| e == Some(0)).count();
        let holded = self.v.len() - open - deaf;

        let min_b = self.v.iter().filter(|e| e.is_some() && e.unwrap() > 0).min().unwrap().unwrap();
        let max_h = self.v.iter().filter(|e| e.is_some() && e.unwrap() > 0).max().unwrap().unwrap();

        let mut amp_fine = (max_h - min_b).saturating_sub(2) as f32 * 0.2;

        let barre = if holded > 3 && min_b > 0 {min_b} else {0};
        let holded = if barre > 0 {self.v.iter().filter(|e| e.is_some() && e.unwrap() > 0 && (e.unwrap() - barre > 0)).count()} else {holded};

        if barre > 0 {
            amp_fine *= 2.0
        }
        
        let barre_fine = if barre > 1 {0.3 + 0.05 * barre as f32} else if barre == 1 {0.5} else {0.0}; 
        let open_up_barre_fine = if barre > 0 && open_up > 0 {0.3} else {0.0};
        let distance_fine = min_b as f32 * 0.08;

        let hold_fine = if barre == 0 {holded.saturating_sub(2) as f32 * 0.1} else {holded.saturating_sub(1) as f32 * 0.2};

        if barre == 0 && holded > 4 || barre > 0 && holded > 3 {
            return None
        }

        if self.v.iter().all(|&s| if let Some(s) = s {s >= 12} else {true}) {
           return None;
        }

        // based on hash to make deduplication work
        let rand_part = (calculate_hash(&self.v) as f32) / u64::MAX as f32 * 1e-5;

        Some(NotNan::new(quality + barre_fine + open_up_barre_fine + hold_fine + amp_fine + distance_fine + rand_part).unwrap())
    }
}

impl Display for FormattedChord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
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

pub fn build_chord_rank(tuning: Tuning, name: &str) -> Result<Vec<(Option<NotNan<f32>>, FormattedChord)>, String> {
    let chord = Chord::parse(name).ok().ok_or(format!("{} is not a correct chord", name))?;
    let root = chord.root();
    let delta = root != chord.slash();
    let notes = chord.chord();
    let mut fifth_note = u8::MAX;
    if notes.len() - delta as usize > 3 {
        fifth_note = chord.chord().into_iter().find(|&n| n - root == Interval::PerfectFifth).as_ref().map(note_to_pitch).unwrap_or(fifth_note);
    }
    
    let chords = tuning.find_chord(notes, fifth_note);
    let mut chords: Vec<_> = chords.into_iter()
        .map(|c| c.format(tuning.strings()))
        .map(|c| (c.evaluate_complexity(), c))
        .filter(|c| c.0.is_some()).collect();

    chords.sort_by_key(|s| s.0);
    Ok(chords)
}

pub fn search_chord(tuning: Tuning, name: &str) -> Result<Vec<String>, String> {
    let mut strings: Vec<_> = build_chord_rank(tuning, name)?.into_iter().map(|e| e.1.to_string() + if e.1.no_fifth {"?"} else {""}).collect();
    strings.dedup();
    Ok(strings)
}
