use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
    pub style: UnderlineStyle,
    pub color: Color,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub underline: Option<Underline>,
}
