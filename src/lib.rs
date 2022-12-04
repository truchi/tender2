#![allow(unused)]

pub mod line;
pub mod row;
pub mod spans;
pub mod style;

use line::*;
use row::*;
use spans::*;
use style::*;

fn default<T: Default>() -> T {
    T::default()
}
