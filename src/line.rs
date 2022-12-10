use super::*;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, Graphemes, UnicodeSegmentation};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Line {
    pub str: String,
    pub width: u16,
}

impl Line {
    pub fn new(width: u16) -> Self {
        let mut str = String::with_capacity(width as usize);
        for _ in 0..width {
            str.push(' ');
        }

        Self { str, width }
    }

    pub fn cells(&self) -> cell::Cells {
        cell::Cells::new(&self.str)
    }

    /// Adds `str` to the ['Line'].
    pub fn push(&mut self, str: &str) {
        fn width(str: &str) -> u16 {
            str.graphemes(true)
                .map(|grapheme| grapheme.width().max(2) as u16)
                .sum()
        }

        // Easy
        if str.is_empty() {
            return;
        }
        // Peasy
        else if self.str.len() == 0 {
            self.str.push_str(str);
            self.width = width(str);
            return;
        }

        // Push the new part
        let at = self.str.len();
        self.str.push_str(str);

        // We want to know if we are adding to a grapheme
        let mut cursor = GraphemeCursor::new(at, self.str.len(), true);
        let is_boundary = match cursor.is_boundary(&self.str, 0) {
            Ok(is_boundary) => is_boundary,
            _ => unreachable!(),
        };

        let str = if !is_boundary {
            // We are adding to a grapheme: we want to know its range
            let start = match cursor.prev_boundary(&self.str, 0) {
                Ok(Some(start)) => start,
                _ => unreachable!(),
            };
            let end = match cursor.next_boundary(&self.str, 0) {
                Ok(Some(start)) => start,
                _ => unreachable!(),
            };

            // Adjust the width for the overlapping grapheme
            self.width -= width(&self.str[start..at]);
            self.width += width(&self.str[start..end]);

            // Give the new full graphemes
            &self.str[end..]
        } else {
            str
        };

        // Add the width
        self.width += width(str);
    }

    // TODO handle graphemes correctly on start/end
    pub fn paint(&mut self, column: u16, str: &str) -> u16 {
        debug_assert!(!str.contains('\n'));

        // Nothing to do when `column` is outside the `Line`
        if column >= self.width {
            return 0;
        }

        // Find the width of `str`
        let (str, width) = {
            // Crop `str` to the available width
            let last = cell::Cells::new(str)
                .take_while(|cell| cell.column + cell.width <= self.width - column)
                .last();

            // There is nothing to paint
            let Some(last) = last else { return 0; };

            (
                // Crop after the last cell
                &str[..last.index + last.str.len()],
                // Total width
                last.column + last.width,
            )
        };

        // Find the index of `column` in the `Line`
        let (start, wide_start) = {
            // Find the cell at `column`
            let cell = cell::Cells::new(&self.str)
                .skip_while(|cell| cell.column + cell.width <= column)
                .next();

            // We already `column` is inside the `Line`
            debug_assert!(cell.is_some());
            let Some(cell) = cell else { return 0; };

            (
                // We migth start on a wide cell
                // Include it anyway and remember the wide start
                cell.index,
                if cell.column != column {
                    debug_assert!(cell.column + 1 == column);
                    debug_assert!(cell.width == 2);
                    true
                } else {
                    false
                },
            )
        };

        // Find the index of `column + width` in the `Line`
        let (end, wide_end) = {
            // On a wide start, we are actually starting a space before `column`
            let width = if wide_start { width + 1 } else { width };

            // Find the cell at `column + width`
            let cell = cell::Cells::new(&self.str[start..])
                .skip_while(|cell| cell.column + cell.width < width)
                .next();

            // We already know `column + width` is inside the `Line`
            debug_assert!(cell.is_some());
            let Some(cell) = cell else { return 0; };

            (
                // We migth end on a wide cell
                // Include it anyway and remember the wide end
                start + cell.index + cell.str.len(),
                if cell.column + cell.width != width {
                    debug_assert!(cell.column + 1 == width);
                    debug_assert!(cell.width == 2);
                    true
                } else {
                    false
                },
            )
        };

        // Replace `str` in the `Line`
        // NOTE we can do better in terms of performance (copy once) but it would be unsafe...

        let mut start = start;
        let mut end = end;

        if wide_start {
            self.str.insert(start, ' ');
            start += 1;
            end += 1;
        }

        self.str.replace_range(start..end, str);

        if wide_end {
            self.str.insert(start + str.len(), ' ');
        }

        width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn s(str: &str) -> String {
        str.into()
    }

    #[test_case("abc🦀d🦀f", 0, "!!!" => (3, s("!!!🦀d🦀f")); "Paint at 0")]
    #[test_case("abc🦀d🦀f", 1, "!!!" => (3, s("a!!! d🦀f")); "Paint at 1")]
    #[test_case("abc🦀d🦀f", 2, "!!!" => (3, s("ab!!!d🦀f")); "Paint at 2")]
    #[test_case("abc🦀d🦀f", 3, "!!!" => (3, s("abc!!!🦀f")); "Paint at 3")]
    #[test_case("abc🦀d🦀f", 4, "!!!" => (3, s("abc !!! f")); "Paint at 4")]
    #[test_case("abc🦀d🦀f", 5, "!!!" => (3, s("abc🦀!!!f")); "Paint at 5")]
    #[test_case("abc🦀d🦀f", 6, "!!!" => (3, s("abc🦀d!!!")); "Paint at 6")]
    #[test_case("abc🦀d🦀f", 7, "!!!" => (2, s("abc🦀d !!")); "Paint at 7")]
    #[test_case("abc🦀d🦀f", 8, "!!!" => (1, s("abc🦀d🦀!")); "Paint at 8")]
    #[test_case("abc🦀d🦀f", 9, "!!!" => (0, s("abc🦀d🦀f")); "Paint at 9")]
    fn paint(initial: &str, column: u16, str: &str) -> (u16, String) {
        let width = initial.width();
        let mut line = Line::new(width as u16);

        line.paint(0, initial);
        assert_eq!(line.str, initial);

        let w = line.paint(column, str);
        assert_eq!(line.str.width(), width);

        (w, line.str)
    }
}

pub mod cell {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub struct Cell<'a> {
        pub index: usize,
        pub column: u16,
        pub width: u16,
        pub str: &'a str,
    }

    #[derive(Clone, Debug)]
    pub struct Cells<'a> {
        graphemes: Graphemes<'a>,
        index: usize,
        column: u16,
    }

    impl<'a> Cells<'a> {
        pub fn new(str: &'a str) -> Self {
            Self {
                graphemes: str.graphemes(true),
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
            let width = str.width() as u16;

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

            let cell = |index, column, width, str| cell::Cell {
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
