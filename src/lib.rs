#![allow(unused)]

pub mod row;
pub mod spans;
pub mod style;

use row::*;
use spans::*;
use style::*;

fn default<T: Default>() -> T {
    T::default()
}
