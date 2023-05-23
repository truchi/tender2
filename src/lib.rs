#![allow(unused)]

pub mod canvas;
pub mod line;
pub mod row;
pub mod spans;
pub mod style;
pub mod unicode;

use canvas::*;
use line::*;
use row::*;
use spans::*;
use style::*;
use unicode::*;

const ZWNJ: char = '\u{200C}';

fn default<T: Default>() -> T {
    T::default()
}
