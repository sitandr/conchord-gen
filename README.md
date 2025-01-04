# Conchord generator

This is tool to generate chord diagrams, with help of [kord](https://github.com/twitchax/kord/tree/main?tab=readme-ov-file). It is used for [conchord]() Typst package, see this repo for good visualizations.

## Using cli

Just input the chord and get the fret numbers. For complex chords, `?` marks the variations with skipped perfect fifth (not "true" chord, but it's okay to use it instead).

```bash
$ cargo run --bin cli --quiet -- "Cmaj7/E"
032000
022010
032400?
032003
xx2410?
032403
02x010
022013
032x00?
022x10?
03x400?
03x000
x75557
0324x0?
032x03
02x410?
0x2410?
03x410?
032410?
022410?
x, 7, 10, 9, 8, 7
x79557?
x79558?
0324x3
022x13
02xx10?
03xx00?
03x4x0?
x7x557?
0xx410?
0x2413
x799x8?
12, 10, 10, x, 12, 12
x, 7, 10, 9, x, 7?
x7x458?
x79x58?
12, x, 9, 9, 8, 8
12, x, 9, 9, 13, 12?
12, x, 10, x, 12, 12?
12, x, 9, 9, x, 8?
12, x, x, 9, 12, 8?
12, x, 9, x, 13, 12?
12, 14, 10, x, x, 12?
```