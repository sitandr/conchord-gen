use std::fmt::Display;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;

use klib::core::base::Parsable;
use klib::core::interval::Interval;
use klib::core::note::Note;
use klib::core::chord::{Chord, HasChord, HasRoot, HasSlash};
use klib::core::note::NoteRecreator;
use klib::core::octave::HasOctave;
use klib::core::octave::Octave;
use klib::core::pitch::HasPitch;
use ordered_float::NotNan;

pub struct Tuning {
    notes: Vec<u8>
}

const FRET_RANGE: u8 = 5;

impl Tuning {
    fn new(notes: &[Note]) -> Self {
        Self{notes: notes.iter().map(note_to_full_pitch).collect()}
    }

    pub fn from_str(s: &str) -> Self {
        let tune_vec: Vec<_> = s.split(' ').map(|s| Note::parse(&s[..s.len()-1]).expect("Bad tuning note").with_octave(s[s.len()-1..].parse::<u8>().unwrap().try_into().unwrap())).collect();
        Tuning::new(&tune_vec)
    }

    #[inline]
    fn strings(&self) -> usize {
        self.notes.len()
    }

    fn find_chord(&self, chord: &[u8], optional_fifth: u8) -> Vec<FoundChord> {
        (1..11).flat_map(|s| self.find_chord_with_shift(chord, s, optional_fifth).into_iter()).collect()
    }
 
    fn find_chord_with_shift(&self, chord: &[u8], shift: u8, optional_fifth: u8) -> Vec<FoundChord> {
        let mut found = self.find_chord_from_string(chord, &chord, 0, shift, optional_fifth);
        found = found.into_iter().filter(|c| c.find_bas_note(self) == chord[0]).collect();
        found
    }

    
    /// `optional_fifth`: from 0 to 12, if specified, this note could be omitted; if not possible there, pass u8::MAX
    fn find_chord_from_string(&self, chord: &[u8], left: &[u8], start_string: usize, shift: u8, optional_fifth: u8) -> Vec<FoundChord> {
        if start_string == self.strings() {
            if left.is_empty() {
                return vec![FoundChord{hold: Vec::with_capacity(self.strings()), no_fifth: false}]
            }
            else if left.len() == 1 && left[0] == optional_fifth {
                return vec![FoundChord{hold: Vec::with_capacity(self.strings()), no_fifth: true}]
            }
        }

        let mut collected = vec![];
        for first_string in start_string..self.strings() {
            let base = self.notes[first_string] + shift;
            // all in range + open
            for possible in (base..base+FRET_RANGE).chain(iter::once(self.notes[first_string])) {
                let possible_m = possible % 12;
                if chord.contains(&possible_m) {
                    let mut left = left.to_owned();
                    // remove found notes from `left`
                    left.retain(|&x| x != possible_m);
                    let found = self.find_chord_from_string(chord, &left, first_string + 1, shift, optional_fifth);
                    collected.extend(found.extend_all((first_string, possible + shift - base)));
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
            assert_eq!(v[holded.0], None, "Duplicated strings: {self:?}?");
            v[holded.0] = Some(holded.1);
        }

        FormattedChord{v, no_fifth: self.no_fifth}
    }

    fn find_bas_note(&self, t: &Tuning) -> u8 {
        let b = self.hold.iter().map(|&(string, h)| t.notes[string] + h).min().expect("Chord is empty") % 12;
        // eprintln!("{}, {:?}", self.format(6), t.notes);
        b
    }

    #[allow(dead_code)]
    fn to_string(&self, strings: usize) -> String {
        self.format(strings).to_string()
    }   
}

impl FormattedChord {
    #[allow(clippy::cast_precision_loss)]
    fn evaluate_complexity(&self) -> Option<NotNan<f32>> {
        let deaf = self.v.iter().filter(|e| e.is_none()).count();
        let upper_deaf = self.v.iter().take_while(|e| e.is_none()).count();
        let under_deaf = deaf - upper_deaf;

        let quality = upper_deaf as f32 * 0.3 + under_deaf as f32 * 1.2;
        
        let open = self.v.iter().filter(|&&e| e == Some(0)).count();
        let max_open = self.v.iter().enumerate().rev().find(|&(_, &e)| e == Some(0)).map_or(0, |e| e.0 + 1);

        let min_b = self.v.iter().filter(|e| e.is_some() && e.unwrap() > 0).min().unwrap_or(&Some(0)).unwrap();
        let first_min = if min_b > 0 {self.v.iter().enumerate().find(|&(_, &e)| e.is_some() && e == Some(min_b)).unwrap().0} else {0};
        let max_h = self.v.iter().filter(|e| e.is_some() && e.unwrap() > 0).max().unwrap_or(&Some(0)).unwrap();
        let mut amp_fine = f32::from((max_h - min_b).saturating_sub(2)) * 0.23;
        
        let mut holded = self.v.len() - open - deaf;

        let holded_at_barre = self.v.iter().skip(max_open).filter(|e| e.is_some() && e.unwrap() == min_b).count();
        let barre = if holded >= 4 && holded_at_barre >= 2 && min_b > 0 && (max_open <= first_min) {min_b} else {0};

        if barre > 0 {
            holded -= holded_at_barre;
            // holded = self.v.iter().take(min_open).filter(|e| e.is_some() && e.unwrap() > 0).count();
            // + 1 as we don't need to check opened string
            // holded += self.v.iter().skip(min_open + 1).filter(|e| e.is_some() && e.unwrap() > 0 && (e.unwrap() - barre > 0)).count()  
        };

        if barre > 0 {
            amp_fine *= 1.5;
        }
        
        let barre_fine = match barre {
            1 => 0.5,
            n if n > 1 => 0.3 + 0.03 * f32::from(barre),
            _ => 0.0
        };
        let open_up_barre_fine = if barre > 0 && max_open > 0 {0.2 + max_open as f32 * 0.02} else {0.0};
        let distance_fine = f32::from(max_h).powf(1.5) * 0.05;

        let hold_fine = if barre == 0 {(holded.saturating_sub(1) as f32).powf(1.5) * 0.2} else {(holded as f32).powf(1.5) * 0.15};

        if barre == 0 && holded > 4 || barre > 0 && holded > 3 {
            return None
        }

        if self.v.iter().all(|&s| if let Some(s) = s {s >= 12} else {true}) {
           return None;
        }

        // based on hash to make deduplication work
        #[allow(clippy::cast_precision_loss)]
        let rand_part = (calculate_hash(&self.v) as f32) / u64::MAX as f32 * 1e-5;

        Some(NotNan::new(quality + barre_fine + open_up_barre_fine + hold_fine + amp_fine + distance_fine + rand_part).unwrap())
    }
}

impl Display for FormattedChord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String =
            if self.v.iter().all(|s| if let &Some(t) = s {t < 10} else {true}) {
                self.v.iter().map(|t| if let Some(t) = t {t.to_string()} else {"x".to_string()}).collect()
            } else {
                self.v.iter().map(|t| if let Some(t) = t {t.to_string()} else {"x".to_string()}).intersperse(", ".to_string()).collect()
            };
        write!(f, "{s}")
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

#[inline]
fn note_to_pitch(n: &Note) -> u8 {
    // SAFETY: Pitch is `repr(u8)`
    let note_pitch: u8 = unsafe { *<*const _>::from(&n.pitch()).cast::<u8>() };
    note_pitch
}

#[inline]
fn note_to_full_pitch(n: &Note) -> u8 {
    // SAFETY: Octave is `repr(u8)`
    let octave_pitch: u8 = unsafe { *<*const _>::from(&n.octave()).cast::<u8>() };
    note_to_pitch(n) + octave_pitch * 12
}

fn check_matches_shift(chord: &FoundChord, shift: u8) -> bool {
    let min_note = chord.hold.iter().filter_map(|(_, n)| if *n == 0 {None} else {Some(*n)} ).min().unwrap_or(0);
    shift == min_note
}

type RankedChord = (Option<NotNan<f32>>, FormattedChord);

pub fn build_chord_rank(tuning: &Tuning, name: &str, shift: u8) -> Result<Vec<RankedChord>, String> {
    let chord = Chord::parse(name).ok().ok_or(format!("{name} is not a correct chord"))?;
    let root = chord.root();
    let delta = root != chord.slash();
    let notes = chord.chord();
    let mut fifth_note = u8::MAX;
    if notes.len() - usize::from(delta) > 3 {
        fifth_note = chord.chord().iter().find(|&&n| n - root == Interval::PerfectFifth).map_or(u8::MAX, note_to_pitch);
    }
    let notes: Vec<u8> = notes.iter().map(note_to_pitch).collect();

    let chords = if shift == u8::MAX {
        tuning.find_chord(&notes, fifth_note)
    } else {
        // searching from 0 is in fact searching from 1
        let find_shift = if shift == 0 {
            1
        } else {shift};
        tuning.find_chord_with_shift(&notes, find_shift, fifth_note).into_iter().filter(|c| check_matches_shift(c, shift)).collect()
    };
    let mut chords: Vec<_> = chords.into_iter()
        .map(|c| c.format(tuning.strings()))
        .map(|c| (c.evaluate_complexity(), c))
        .filter(|c| c.0.is_some()).collect();

    chords.sort_by_key(|s| s.0);
    Ok(chords)
}

pub fn search_chord(tuning: &Tuning, name: &str, shift: u8) -> Result<Vec<String>, String> {
    let mut strings: Vec<_> = build_chord_rank(tuning, name, shift)?.into_iter().map(|e| e.1.to_string() + if e.1.no_fifth {"?"} else {""}).collect();
    strings.dedup();
    Ok(strings)
}
