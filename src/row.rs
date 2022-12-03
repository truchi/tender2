use std::str::CharIndices;

use super::*;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Row {
    text: String,
    spans: Spans,
}

impl Row {}
