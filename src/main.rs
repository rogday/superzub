#![feature(const_fn)]

use std::collections::{HashMap, VecDeque};

use fool::BoolExt;
use itertools::Itertools;

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
    AlphabetMismatch,
    Unsolvable,
}

#[derive(Debug)]
struct Trace {
    // According to Wiki, the longest optimal solution is 31 moves long.
    trace: Vec<u32>,
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.trace.iter().try_for_each(|&field| {
            let blank = get_blank_pos(field);
            for i in 0..9 {
                if i != blank {
                    write!(f, "{} ", get_tile(field, i))?;
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

#[inline(always)]
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

    //clean up garbage in blank tile for future moves
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

const fn fact(mut x: usize) -> usize {
    let mut ret = 1;
    while x > 1 {
        ret *= x;
        x -= 1;
    }
    ret
}

/// https://www.cs.bham.ac.uk/~mdr/teaching/modules04/java2/TilesSolvability.html
///
/// If N(in NxN puzzle) is odd, then puzzle instance is solvable if number of inversions is even in the input state.
/// An inversion is when a tile precedes another tile with a lower number on it.
///
/// Proof:
///
/// Moving a tile along the row (left or right) doesn’t change the number of inversions,
/// and therefore doesn’t change its polarity.
/// Moving a tile along the column (up or down) can change the number of inversions.
/// The tile moves past an even number of other tiles (N – 1). So move changes number of inversions by (+i - k),
/// so i and k are both odd or even, so the change is even
fn check_solvability(input: &[u32; 9]) -> Result<(), SolveError> {
    let inversions = (0..9)
        .flat_map(|i| std::iter::once(i).cartesian_product(i + 1..9))
        .filter(|&(i, k)| input[k] != 0 && input[i] > input[k])
        .count();

    (inversions % 2 == 0).ok_or(SolveError::Unsolvable)
}

fn validate_input(input: &[u32; 9]) -> Result<(), SolveError> {
    let count = |x: u32| input.iter().filter(|&&y| x == y).count();

    input.iter().all(|&x| x < 9 && count(x) == 1).ok_or(SolveError::AlphabetMismatch)
}

fn pack(input: &[u32; 9]) -> u32 {
    input.iter().enumerate().fold(0, |packed, (index, &tile)| {
        packed | if tile == 0 { to_pos(index as u32) } else { (tile - 1) << (index * 3) }
    })
}

fn solve(input: &[u32; 9]) -> Result<Trace, SolveError> {
    validate_input(input)?;
    check_solvability(input)?;

    // +---+---+---+
    // | 1 | 2 | 3 |
    // | 4 | 5 | 6 |
    // | 7 | 8 |   |
    // +---+---+---+
    const GOAL: u32 = 0b01000000111110101100011010001000;

    let input = pack(input);

    println!("input:  {:#034b}\noutput: {:#034b}\n", input, GOAL);

    Ok(bfs(input, GOAL))
}

fn bfs(input: u32, output: u32) -> Trace {
    const MAX_CAPACITY: usize = fact(9);

    let mut tree = HashMap::with_capacity(MAX_CAPACITY);
    let mut moves = VecDeque::with_capacity(MAX_CAPACITY);

    tree.insert(output, output);
    moves.push_back(output);

    let mut current = 0;

    while current != input {
        current = moves.pop_front().unwrap();

        for f in &[up, down, left, right] {
            let value = f(current);

            tree.entry(value).or_insert_with(|| {
                moves.push_back(value);
                current
            });
        }
    }

    let mut trace = vec![current];

    while current != tree[&current] {
        current = tree[&current];
        trace.push(current);
    }

    Trace { trace }
}

fn main() {
    #[rustfmt::skip]
    let input = &[
        1, 2, 3,
        4, 5, 0,
        6, 7, 8];

    match solve(input) {
        Ok(trace) => println!("{}", trace),
        Err(err) => eprintln!("{:?}", err),
    }
}
