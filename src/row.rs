use super::*;
use std::str::CharIndices;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Row {
    line: Line,
    spans: Spans,
}

impl Row {
    pub fn new(string: String, style: Style) -> Self {
        let line = Line::new(string);
        let width = line.width();
        let spans = Spans::new(vec![Span { width, style }]);

        Self { line, spans }
    }

    pub fn cells(&self) -> Cells {
        Cells::new(self)
    }

    pub fn push(&mut self, str: &str, style: Style) {
        let width = self.line.push(str);
        self.spans.push(Span { width, style });
    }

    pub fn paint(&mut self, column: u16, str: &str, style: Style) {
        let width = self.line.paint(column, str);
        self.spans.paint(column, Span { width, style });
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cell<'a> {
    str: &'a str,
    width: u16,
    style: Style,
}

#[derive(Clone, Debug)]
pub struct Cells<'a> {
    line: line::cell::Cells<'a>,
    spans: spans::Iter<'a>,
    span: Span,
}

impl<'a> Cells<'a> {
    pub fn new(row: &'a Row) -> Self {
        Self {
            line: row.line.cells(),
            spans: row.spans.iter(),
            span: default(),
        }
    }
}

impl<'a> Iterator for Cells<'a> {
    type Item = Cell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.line.next()?;

        if self.span.width == 0 {
            let span = self.spans.next().copied();

            // We know `Spans` covers the entire `Line` in a `Row`
            debug_assert!(span.is_some());
            let Some(span) = span else { return None; };

            self.span = span;
        }

        // If everything is right, this is OK
        debug_assert!(cell.width <= self.span.width);
        self.span.width -= cell.width;

        Some(Cell {
            str: cell.str,
            width: cell.width,
            style: self.span.style,
        })
    }
}
