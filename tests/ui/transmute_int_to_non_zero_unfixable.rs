//@no-rustfix
#![warn(clippy::transmute_int_to_non_zero)]

use std::num::NonZero;

macro_rules! to_non_zero {
    ($x:expr) => {
        unsafe { std::mem::transmute($x) }
        //~^ transmute_int_to_non_zero
    };
}

fn in_macro_body(x: u32) -> NonZero<u32> {
    // Still linted, but a suggestion would also change every other expansion of the macro
    to_non_zero!(x)
}

fn main() {}
