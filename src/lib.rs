#![allow(unused)]

pub mod row;
pub mod segment;

use row::*;
use segment::*;

fn default<T: Default>() -> T {
    T::default()
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum UnderlineStyle {
    #[default]
    Single,
    Double,
    Curl,
    Dot,
    Dash,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Underline {
    style: UnderlineStyle,
    color: Color,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Style {
    foreground: Color,
    background: Color,
    bold: bool,
    italic: bool,
    strike: bool,
    underline: Option<Underline>,
}
