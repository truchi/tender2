pub use cells::*;

use unicode_segmentation::Graphemes;

/// Unicode helpers.
pub trait Unicode {
    type Graphemes<'a>
    where
        Self: 'a;
    type Cells<'a>
    where
        Self: 'a;

    /// Returns the displayed width in columns.
    fn width(&self) -> usize;

    /// Returns an iterator over the grapheme clusters.
    fn graphemes(&self) -> Self::Graphemes<'_>;

    /// Returns an iterator over the cells.
    fn cells(&self) -> Self::Cells<'_>;
}

impl Unicode for str {
    type Graphemes<'a> = Graphemes<'a>;
    type Cells<'a> = Cells<'a>;

    fn width(&self) -> usize {
        unicode_width::UnicodeWidthStr::width(self)
    }

    fn graphemes(&self) -> Self::Graphemes<'_> {
        unicode_segmentation::UnicodeSegmentation::graphemes(self, true)
    }

    fn cells(&self) -> Self::Cells<'_> {
        Cells::new(self)
    }
}

pub mod cells {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub struct Cell<'a> {
        pub index: usize,
        pub column: usize,
        pub width: usize,
        pub str: &'a str,
    }

    #[derive(Clone, Debug)]
    pub struct Cells<'a> {
        graphemes: Graphemes<'a>,
        index: usize,
        column: usize,
    }

    impl<'a> Cells<'a> {
        pub fn new(str: &'a str) -> Self {
            Self {
                graphemes: str.graphemes(),
                index: 0,
                column: 0,
            }
        }
    }

    impl<'a> Iterator for Cells<'a> {
        type Item = Cell<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            let str = self.graphemes.next()?;
            let index = self.index;
            let column = self.column;
            let width = str.width();

            // `unicode_width` has trouble with some clusters (e.g. woman scientist emoji)
            let width = width.min(2);

            self.index += str.len();
            self.column += width;

            if width == 0 {
                self.next()
            } else {
                Some(Cell {
                    index,
                    column,
                    width,
                    str,
                })
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test() {
            const NUL: &str = "\0";
            const A: &str = "a̐";
            const O: &str = "ö̲";
            const E: &str = "é";
            const CRAB: &str = "🦀";
            const CRLF: &str = "\r\n";

            let cell = |index, column, width, str| Cell {
                index,
                column,
                width,
                str,
            };

            let str = [NUL, A, NUL, O, CRAB, E, CRLF].concat();
            let cells = Cells::new(&str).collect::<Vec<_>>();

            assert_eq!(
                cells,
                [
                    cell(1, 0, 1, A),
                    cell(2 + A.len(), 1, 1, O),
                    cell(2 + A.len() + O.len(), 2, 2, CRAB),
                    cell(2 + A.len() + O.len() + CRAB.len(), 4, 1, E),
                ]
            );
        }
    }
}

pub mod graphemes {
    use super::*;
    use crate::ZWNJ;

    pub struct Graphemes<'a> {
        str: &'a str,
    }

    impl<'a> Graphemes<'a> {
        pub fn new(str: &'a str) -> Self {
            Self { str }
        }
    }

    impl<'a> Iterator for Graphemes<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<Self::Item> {
            use unicode_segmentation as us;

            let str = self.str;
            let mut chars = self.str.chars();

            let first = chars.next()?;
            let second = chars.next();

            let mut grapheme = |len| {
                let (grapheme, str) = self.str.split_at(len);
                self.str = str;
                Some(grapheme)
            };

            if first.is_ascii() {
                if let Some(second) = second {
                    if second.is_ascii() {
                        grapheme(1)
                    } else {
                        grapheme(us::UnicodeSegmentation::graphemes(str, true).next()?.len())
                    }
                } else {
                    grapheme(1)
                }
            } else {
                grapheme(us::UnicodeSegmentation::graphemes(str, true).next()?.len())
            }
        }
    }
}
