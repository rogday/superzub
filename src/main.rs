#![feature(const_fn)]

use std::collections::{hash_map::Entry, HashMap};

// This program solves a variation of 15-puzzle game.
//
// Let's assume we have a 3 by 3 field like this:
//
// +---+---+---+
// | a | b | c |
// | d | e |   |
// | g | h | i |
// +---+---+---+
//
// Note that element on (x: 2, y: 1) represents a blank tile.
// This will be converted to an unsigned 32 bit integer by placing nine 3-bit tile indices [0, 7], occupying 27 bits.
// 5 bits left will be used as a position(P) of a blank tile, so that it can be found quickly.
// The field above will be converted to in binary: PPPPPIII.HHHGGG00.0EEEDDDC.CCBBBAAA,
// where dots delimit bytes and 3 bits owned by the blank tile are filled with zeros.

#[derive(Debug)]
enum SolveError {
    InputSizeMismatch,
    OutputSizeMismatch,

    AlphabetMismatch,

    Unsolvable,
}

#[derive(Debug)]
struct Trace {
    // According to Wiki, the longest optimal solution is 80 moves long.
    trace:   Vec<u32>,
    mapping: HashMap<u32, char>,
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.trace.iter().try_for_each(|&field| {
            let blank = get_blank_pos(field);
            for i in 0..9 {
                if i != blank {
                    let id = get_tile(field, i);
                    write!(f, "{} ", self.mapping[&id])?;
                } else {
                    write!(f, "  ")?;
                }
                if i % 3 == 2 {
                    writeln!(f)?;
                }
            }
            writeln!(f)
        })
    }
}

#[inline(always)]
const fn get_blank_pos(field: u32) -> u32 {
    field >> 27
}

#[inline(always)]
const fn get_tile(field: u32, i: u32) -> u32 {
    let mask = get_mask(i);
    (field & mask) >> (i * 3)
}

#[inline(always)]
const fn to_pos(x: u32) -> u32 {
    x << 27
}

#[inline(always)]
const fn wrap_around(x: i32) -> u32 {
    ((32 + x) % 32) as u32
}

const fn get_mask(x: u32) -> u32 {
    0b111 << (x * 3)
}

fn make_move(mut field: u32, in_bounds: fn(u32) -> bool, delta_pos: i32) -> u32 {
    //extract position from field
    let mut blank_pos = get_blank_pos(field);

    if !in_bounds(blank_pos) {
        return field;
    }

    // calculate new position of blank tile
    blank_pos = (blank_pos as i32 + delta_pos) as u32;

    //FIXME: examples
    //turn position of a number into mask on that number
    let mask = get_mask(blank_pos);

    //extract the digit from place in which we will move blank tile
    let masked = field & mask;

    // every cell is represented using 3 bits, this converts change of
    // position into amount of bits that need to be shifted
    let shift = delta_pos * 3;

    // move digit to old blank space
    // negative shifts work by wrapping around so that masked << -1 becomes masked >> 1
    let digit_new = masked.rotate_right(wrap_around(shift));

    //clean up garbage from blank space for digit_new to be placed in
    field &= !mask;

    // apply digit move
    field |= digit_new;

    // apply position change
    field += to_pos(delta_pos as u32);

    field
}

fn up(field: u32) -> u32 {
    make_move(field, |pos| pos >= 3, -3)
}

fn down(field: u32) -> u32 {
    make_move(field, |pos| pos <= 5, 3)
}

fn left(field: u32) -> u32 {
    make_move(field, |pos| pos % 3 != 0, -1)
}

fn right(field: u32) -> u32 {
    make_move(field, |pos| pos % 3 != 2, 1)
}

fn make_mapping(input: &str) -> HashMap<char, u32> {
    let mut mapping = HashMap::with_capacity(9);
    let mut key = 0;

    for c in input.chars().filter(|&c| c != ' ') {
        if let Entry::Vacant(entry) = mapping.entry(c) {
            entry.insert(key);
            key += 1;
        }
    }

    mapping
}

trait BoolExt<E> {
    fn ok_or(&self, error: E) -> Result<(), E>;
}

impl<E> BoolExt<E> for bool {
    fn ok_or(&self, error: E) -> Result<(), E> {
        if *self {
            Ok(())
        } else {
            Err(error)
        }
    }
}

fn validate_input(input: &str, output: &str) -> Result<(), SolveError> {
    let char_count = |s: &str| s.chars().count();

    (char_count(input) == 9).ok_or(SolveError::InputSizeMismatch)?;
    (char_count(output) == 9).ok_or(SolveError::OutputSizeMismatch)?;
    is_permutation(input, output).ok_or(SolveError::AlphabetMismatch)
}

fn is_permutation(a: &str, b: &str) -> bool {
    let count = |s: &str, ch| s.chars().filter(|&c| c == ch).count();

    a.chars().all(|ch| count(a, ch) == count(b, ch))
}

const fn fact(mut x: usize) -> usize {
    let mut ret = 1;
    while x > 1 {
        ret *= x;
        x -= 1;
    }
    ret
}

const MAX_CAPACITY: usize = fact(9);

fn pack(input: &str, output: &str, mapping: &HashMap<char, u32>) -> (u32, u32) {
    let mut in_cipher = 0;
    let mut out_cipher = 0;

    let pack = |cipher: &mut u32, index, ch| {
        *cipher |= match ch {
            ' ' => to_pos(index as u32),
            _ => mapping[&ch] << (index * 3),
        };
    };

    for (index, (in_char, out_char)) in input.chars().zip(output.chars()).enumerate() {
        pack(&mut in_cipher, index, in_char);
        pack(&mut out_cipher, index, out_char);
    }

    (in_cipher, out_cipher)
}

fn bfs(input: &str, output: &str) -> Result<Trace, SolveError> {
    validate_input(input, output)?;

    let mut mapping = make_mapping(input);
    let (input, output) = pack(input, output, &mapping);
    let mapping = mapping.drain().map(|(k, v)| (v, k)).collect();

    println!("input:  {:#034b}\noutput: {:#034b}\n", input, output);

    let mut arr = HashMap::with_capacity(MAX_CAPACITY);
    let mut current_moves = Vec::with_capacity(MAX_CAPACITY);
    let mut future_moves = Vec::with_capacity(MAX_CAPACITY);

    arr.insert(output, output);
    current_moves.push(output);

    let mut cur = 0;

    while cur != input {
        cur = current_moves.pop().ok_or(SolveError::Unsolvable)?;

        for f in &[up, down, left, right] {
            let value = f(cur);

            arr.entry(value).or_insert_with(|| {
                future_moves.push(value);
                cur
            });
        }

        if current_moves.is_empty() {
            std::mem::swap(&mut current_moves, &mut future_moves);
        }
    }

    let mut trace = vec![cur];

    while cur != arr[&cur] {
        cur = arr[&cur];
        trace.push(cur);
    }

    Ok(Trace { trace, mapping })
}

fn main() {
    match bfs("abcde ghi", " ihgedcba") {
        Ok(trace) => println!("{}", trace),
        Err(err) => eprintln!("{:?}", err),
    }
}
