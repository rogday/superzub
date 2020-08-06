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
// Note that element on (x: 2, y: 1) represents missing square.
// This will be converted to an unsigned 32 bit integer by placing nine 3-bit tile indices from 0 to +Inf.
// 5 bits left will be used as a position(P) of missing square, so the field above will be converted to:
// in binary: PPPPPIII.HHHGGG__._EEEDDDC.CCBBBAAA,
// where dots delimit bytes and 3 underscored bits is our missing square filled with some garbage values,
// that we're not interested in.

#[derive(Debug)]
enum SolveError {
    _AlphabetMismatch,
    Unsolvable,
}

struct Trace {
    /// According to Wiki, the longest optimal solution is 80 moves long.
    inner: Vec<u32>,

    mapping: HashMap<u32, char>,
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.iter().try_for_each(|&field| {
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

macro_rules! gen_functions {
    ($($fn_name:ident, |$pos: ident| $in_bounds:block, $delta_pos:expr),*) => {
        $(fn $fn_name(mut field: u32) -> u32 {
            //extract position from field
            let mut blank_pos = get_blank_pos(field);

            let $pos = blank_pos;

            if !$in_bounds {
                return field;
            }

            // calculate new position of blank tile
            blank_pos = (blank_pos as i32 + $delta_pos) as u32;

            //FIXME: examples
            //turn position of a number into mask on that number
            let mask = get_mask(blank_pos);

            //extract the digit from place in which we will move blank tile
            let masked = field & mask;

            // every cell is represented using 3 bits, this converts change of
            // position into amount of bits that need to be shifted
            const SHIFT: i32 = $delta_pos * 3;

            // move digit to old blank space
            // negative shifts work by wrapping around so that masked << -1 becomes masked >> 1
            let digit_new = masked.rotate_right(wrap_around(SHIFT));

            //clean up garbage from blank space for digit_new to be placed in
            field &= !mask;

            // apply digit move
            field |= digit_new;

            // apply position change
            field += to_pos($delta_pos as u32);

            field
        })*
    };
}

#[rustfmt::skip]
gen_functions![
    up,    |pos| { pos >= 3 },     -3i32,
    down,  |pos| { pos <= 5 },      3i32,
    left,  |pos| { pos % 3 != 0 }, -1i32,
    right, |pos| { pos % 3 != 2 },  1i32
];

fn make_mapping(input: &str) -> HashMap<char, u32> {
    let mut mapping = HashMap::new();
    let mut key = 0;

    for c in input.chars() {
        if let Entry::Vacant(entry) = mapping.entry(c) {
            entry.insert(key);
            key += 1;
        }
    }

    mapping
}

fn solve(input: &str, output: &str) -> Result<Trace, SolveError> {
    assert!(input.chars().count() == 9, "Input string should be 9 characters long");
    assert!(output.chars().count() == 9, "Output string should be 9 characters long");

    let mapping = make_mapping(input);

    let mut in_cipher: u32 = 0;
    let mut out_cipher: u32 = 0;

    // FIXME: will panic if output uses alphabet > alphabet of input but not otherwise
    // check for space and output = valid permutation of input
    let mutator = |cipher: &mut u32, index: usize, ch: char| {
        *cipher |= match ch {
            ' ' => to_pos(index as u32),
            _ => mapping[&ch] << (index * 3),
        };
    };

    for (index, (in_char, out_char)) in input.chars().zip(output.chars()).enumerate() {
        mutator(&mut in_cipher, index, in_char);
        mutator(&mut out_cipher, index, out_char);
    }

    println!("input:  {:#034b}\noutput: {:#034b}\n", in_cipher, out_cipher);

    let test: HashMap<_, _> = mapping.clone().drain().map(|(k, v)| (v, k)).collect();

    const fn fact(mut x: usize) -> usize {
        let mut ret = 1;
        while x > 1 {
            ret *= x;
            x -= 1;
        }
        ret
    }

    const MAX: usize = fact(9);

    let mut arr = HashMap::with_capacity(MAX);
    let mut current_moves = Vec::with_capacity(MAX);
    let mut future_moves = Vec::with_capacity(MAX);

    arr.insert(out_cipher, out_cipher);
    current_moves.push(out_cipher);

    //FIXME: out_cipher?
    let mut cur: u32 = 0;

    while cur != in_cipher {
        cur = current_moves.pop().ok_or(SolveError::Unsolvable)?;

        for f in &[up, down, left, right] {
            let value = f(cur);

            if let Entry::Vacant(entry) = arr.entry(value) {
                entry.insert(cur);
                future_moves.push(value);
            }
        }

        if current_moves.is_empty() {
            std::mem::swap(&mut current_moves, &mut future_moves);
        }
    }

    let mut trace = Vec::new();
    trace.push(cur);

    while cur != arr[&cur] {
        cur = arr[&cur];
        trace.push(cur);
    }

    //NOTE: field should implement Display, but it needs mapping..
    Ok(Trace { inner: trace, mapping: test })
}

fn main() {
    match solve("12345678 ", " 87654321") {
        Ok(trace) => println!("{}", trace),
        Err(err) => panic!("{:?}", err),
    }
}