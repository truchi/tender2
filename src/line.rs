use super::*;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, Graphemes, UnicodeSegmentation};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn width(str: &str) -> u16 {
    str.graphemes(true)
        .map(|grapheme| grapheme.width().min(2) as u16)
        .sum()
}

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Line {
    string: String,
    width: u16,
}

impl Line {
    pub fn new(string: String) -> Self {
        let width = width(&string);

        Self { width, string }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn cells(&self) -> cell::Cells {
        cell::Cells::new(&self.string)
    }

    /// Adds `str` to the [`Line`] and returns the actual added width.
    pub fn push(&mut self, str: &str) -> u16 {
        // Easy
        if str.is_empty() {
            return 0;
        }
        // Peasy
        else if self.string.len() == 0 {
            let width = width(str);

            self.string.push_str(str);
            self.width = width;

            return width;
        }

        // Push the new part
        let at = self.string.len();
        self.string.push_str(str);

        // We may be joining to a grapheme
        // Eg: `"blah👩\u{200D}".push("🔬blah")` will compose a woman scientist
        let mut cursor = GraphemeCursor::new(at, self.string.len(), true);
        let is_boundary = match cursor.is_boundary(&self.string, 0) {
            Ok(is_boundary) => is_boundary,
            // We give the whole string to the cursor
            _ => unreachable!(),
        };

        // The str without the overlapping grapheme, if any
        let str = if !is_boundary {
            // We are joining to a grapheme
            let start = match cursor.prev_boundary(&self.string, 0) {
                Ok(Some(start)) => start,
                // We give the whole string to the cursor
                _ => unreachable!(),
            };
            let end = match cursor.next_boundary(&self.string, 0) {
                Ok(Some(start)) => start,
                // We give the whole string to the cursor
                _ => unreachable!(),
            };

            // Adjust the width for the overlapping grapheme
            // Due to the "woman scientist issue" in unicode_width
            // we cannot simply `width += string[at..end]`...
            self.width -= width(&self.string[start..at]);
            self.width += width(&self.string[start..end]);

            // Give the new full graphemes
            &self.string[end..]
        } else {
            str
        };

        // Add and return the width
        let width = width(str);
        self.width += width;

        width
    }

    // TODO ZWNJ on start/end
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
            let cell = cell::Cells::new(&self.string)
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
            let cell = cell::Cells::new(&self.string[start..])
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
        {
            // We include zero-width non-joiners (`\u{200C}`) around the painted columns
            // to prevent graphemes to join before/after (this is a feature!)
            // TODO do not include ZWNJ when already there or at the beginning/end of the line
            let before = if wide_start { " \u{200C}" } else { "\u{200C}" };
            let after = if wide_end { "\u{200C} " } else { "\u{200C}" };

            // NOTE we can do better in terms of performance (copy once) but it would be unsafe...
            self.string.insert_str(start, before);
            self.string
                .replace_range(start + before.len()..end + before.len(), str);
            self.string
                .insert_str(start + before.len() + str.len(), after);
        }

        width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    macro_rules! f { ($($tt:tt)*) => { format!($($tt)*) }; }

    #[test_case(""            , 0; "empty")]
    #[test_case(" "           , 1; "space")]
    #[test_case("a"           , 1; "a")]
    #[test_case("🦀"          , 2; "crab")]
    #[test_case("👩\u{200D}🔬", 2; "woman scientist")]
    fn new(str: &str, width: u16) {
        let line = Line::new(str.to_string());

        assert_eq!(line.string, str);
        assert_eq!(line.width, width);
    }

    #[test_case("abc🦀👩\u{200D}🔬def", 10; "Test 1")]
    fn push(string: &str, width: u16) {
        for (i, _) in string.char_indices() {
            let mut line = Line::new(string[..i].to_string());
            line.push(&string[i..]);

            assert_eq!(line.string, string);
            assert_eq!(line.width, width);
        }
    }

    #[test_case("abc🦀d🦀f", 0, "!!!" => (3, f!("{ZWNJ}!!!{ZWNJ}🦀d🦀f")); "Paint at 0")]
    #[test_case("abc🦀d🦀f", 1, "!!!" => (3, f!("a{ZWNJ}!!!{ZWNJ} d🦀f")); "Paint at 1")]
    #[test_case("abc🦀d🦀f", 2, "!!!" => (3, f!("ab{ZWNJ}!!!{ZWNJ}d🦀f")); "Paint at 2")]
    #[test_case("abc🦀d🦀f", 3, "!!!" => (3, f!("abc{ZWNJ}!!!{ZWNJ}🦀f")); "Paint at 3")]
    #[test_case("abc🦀d🦀f", 4, "!!!" => (3, f!("abc {ZWNJ}!!!{ZWNJ} f")); "Paint at 4")]
    #[test_case("abc🦀d🦀f", 5, "!!!" => (3, f!("abc🦀{ZWNJ}!!!{ZWNJ}f")); "Paint at 5")]
    #[test_case("abc🦀d🦀f", 6, "!!!" => (3, f!("abc🦀d{ZWNJ}!!!{ZWNJ}")); "Paint at 6")]
    #[test_case("abc🦀d🦀f", 7, "!!!" => (2, f!("abc🦀d {ZWNJ}!!{ZWNJ}")); "Paint at 7")]
    #[test_case("abc🦀d🦀f", 8, "!!!" => (1, f!("abc🦀d🦀{ZWNJ}!{ZWNJ}")); "Paint at 8")]
    #[test_case("abc🦀d🦀f", 9, "!!!" => (0, f!("abc🦀d🦀f")); "Paint at 9")]
    fn paint(initial: &str, column: u16, str: &str) -> (u16, String) {
        let mut line = Line::new(initial.into());
        let width = line.width;

        let w = line.paint(column, str);
        assert_eq!(line.width, width);

        (w, line.string)
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
