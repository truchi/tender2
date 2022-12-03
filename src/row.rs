use std::str::CharIndices;

use super::*;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Row {
    text: String,
    segments: Vec<Segment>,
}

impl Row {
    pub fn draw(&mut self, x: u16, width: u16, str: &str, style: Style) -> Result<(), ()> {
        debug_assert!(!str.contains('\n'));
        debug_assert!(str.width() == width as usize);
        debug_assert!(width != 0);

        // TODO left-combine first zero-width chars

        let range = Segment::scan(&self.segments, x, width)?;
        let (start, end) = (range.start, range.end);

        if start.index == end.index {
            debug_assert_eq!(start, end);
            let scan = start;

            debug_assert!(scan.segment.width >= width);

            //      -
            // ... --- ...

            //     ---
            // ... --- ...
            if scan.segment.width == width {
                self.text.replace_range(scan.i..scan.i + str.len(), str);
                self.segments[scan.index] = Segment {
                    len: str.len(),
                    width,
                    style,
                };
            }
            //     -
            // ... --- ...
            else if scan.x == x {
                let [lead, overlap] =
                    leading_columns(&self.text[scan.i..scan.i + scan.segment.len], width);
                let len = lead.0.len() + overlap.0.len();
                let extra = lead.1 + overlap.1 - width;

                self.text.replace_range(scan.i..scan.i + len, str);
                for _ in 0..extra {
                    self.text.insert(scan.i + str.len(), ' ');
                }

                let left = Segment {
                    len: str.len(),
                    width,
                    style,
                };
                let right = Segment {
                    len: scan.segment.len + extra as usize - len,
                    width: scan.segment.width - left.width,
                    style: scan.segment.style,
                };

                self.segments.insert(scan.index, left);
                self.segments[scan.index + 1] = right;
            }
            //       -
            // ... --- ...
            else if scan.x + scan.segment.width == x + width {
            }
        } else {
            //     --- ... --
            // ... --- ... --- ...
        }

        Ok(())
    }
}

fn leading_columns(str: &str, width: u16) -> [(&str, u16); 2] {
    let mut x = 0;
    let mut it = str
        .char_indices()
        .map(|(i, char)| {
            let w = char.width().unwrap_or_default() as u16;
            let ixw = (i, x, w);
            x += w;
            ixw
        })
        .skip_while(|(_, x, w)| *x + *w <= width);

    match it.next() {
        None => [(str, x), ("", 0)],
        Some((i, x, w)) => {
            debug_assert!(w != 0);
            debug_assert!(x <= width);

            let first = (&str[..i], x);

            // --
            // ----
            if x == width {
                [first, ("", 0)]
            }
            // --
            // -──-
            else {
                let j = it
                    .skip_while(|(_, _, w)| *w == 0)
                    .next()
                    .map(|(i, _, _)| i)
                    .unwrap_or(str.len());

                debug_assert!(i != j);

                [first, (&str[i..j], w)]
            }
        }
    }
}

fn trailing_columns(str: &str, width: u16) -> [(&str, u16); 2] {
    let mut x = 0;
    let mut it = str
        .char_indices()
        .rev()
        .map(|(i, char)| {
            let w = char.width().unwrap_or_default() as u16;
            x += w;
            let ixw = (char, i, x, w);
            ixw
        })
        .map(|fuck| dbg!(fuck))
        .skip_while(|(_, _, x, w)| *x < width);

    match dbg!(it.next()) {
        None => [(str, x), ("", 0)],
        Some((c, i, x, w)) => {
            dbg!((c, i, x, w));
            debug_assert!(w != 0);
            // debug_assert!(x <= width); // ??

            let start = (&str[i..], x);

            // --
            // ----
            if x == width {
                [start, ("", 0)]
            }
            // --
            // -──-
            else {
                dbg!("ici");
                let j = it
                    .skip_while(|(_, _, _, w)| *w == 0)
                    .next()
                    .map(|a| dbg!(a))
                    .map(|(_, i, _, _)| i)
                    .unwrap_or(str.len());

                // let j = str.len() - j;

                debug_assert!(i != j);

                [start, (&str[j..i], w)]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const RED: Color = Color { r: 255, g: 0, b: 0 };
    const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    fn from_spans<I>(spans: I) -> Row
    where
        I: IntoIterator<Item = (Color, &'static str)>,
    {
        let mut row = Row::default();

        for (foreground, str) in spans {
            row.text.push_str(str);
            row.segments.push(Segment {
                len: str.len(),
                width: str.width() as u16,
                style: Style {
                    foreground,
                    ..default()
                },
            });
        }

        row
    }

    fn draw_in(row: &Row, x: u16, foreground: Color, str: &str) -> Result<Row, ()> {
        let mut row = row.clone();
        row.draw(
            x,
            str.width() as u16,
            str,
            Style {
                foreground,
                ..default()
            },
        )?;

        Ok(row)
    }

    #[test]
    fn draw() {
        let row = from_spans([(RED, "hello")]);
        assert_eq!(
            draw_in(&row, 0, GREEN, "HELLO").unwrap(),
            from_spans([(GREEN, "HELLO")])
        );

        let row = from_spans([(RED, "abcde")]);
        assert_eq!(
            draw_in(&row, 0, GREEN, "AB").unwrap(),
            from_spans([(GREEN, "AB"), (RED, "cde")])
        );

        let row = from_spans([(RED, "a🦀c")]);
        assert_eq!(
            draw_in(&row, 0, GREEN, "AB").unwrap(),
            from_spans([(GREEN, "AB"), (RED, " c")])
        );

        // let row = from_spans([(RED, "I<3🦀")]);
        // assert_eq!(
        //     draw_in(&row, 0, GREEN, "I<3!").unwrap(),
        //     from_spans([(GREEN, "I<3!"), (RED, " ")])
        // );

        // let row = from_spans([(RED, "hello")]);
        // assert_eq!(
        //     draw_in(&row, 0, GREEN, "helloworld").unwrap(),
        //     from_spans([(GREEN, "helloworld")])
        // );

        // let row = from_spans([(RED, "hello"), (GREEN, "world")]);
        // assert_eq!(
        //     draw_in(&row, 0, BLUE, "HELLOWORLD").unwrap(),
        //     from_spans([(BLUE, "HELLOWORLD")])
        // );
    }

    // #[test]
    // fn columns() {
    //     //         12 1
    //     let str = "a🦀b";
    //     // assert_eq!(super::leading_columns(str, 1), [("a", 1), ("", 0)]);
    //     // assert_eq!(super::leading_columns(str, 2), [("a", 1), ("🦀", 2)]);
    //     // assert_eq!(super::leading_columns(str, 3), [("a🦀", 3), ("", 0)]);
    //     // assert_eq!(super::leading_columns(str, 4), [("a🦀b", 4), ("", 0)]);
    //     // assert_eq!(super::leading_columns(str, 5), [("a🦀b", 4), ("", 0)]);

    //     //         1  2   1
    //     let str = "a\0🦀\0b\0";
    //     // assert_eq!(super::leading_columns(str, 1), [("a\0", 1), ("", 0)]);
    //     // assert_eq!(super::leading_columns(str, 2), [("a\0", 1), ("🦀\0", 2)]);
    //     // assert_eq!(super::leading_columns(str, 3), [("a\0🦀\0", 3), ("", 0)]);
    //     // assert_eq!(super::leading_columns(str, 4), [("a\0🦀\0b\0", 4), ("", 0)]);
    //     // assert_eq!(super::leading_columns(str, 5), [("a\0🦀\0b\0", 4), ("", 0)]);

    //     let str = "ab🦀c";
    //     // assert_eq!(trailing_columns(str, 1), [("c", 1), ("", 0)]);
    //     assert_eq!(trailing_columns(str, 2), [("c", 1), ("🦀", 2)]);
    //     // assert_eq!(trailing_columns(str, 3), [("🦀b", 3), ("", 0)]);
    //     // assert_eq!(trailing_columns(str, 5), [("a🦀b", 4), ("", 0)]);
    //     // assert_eq!(trailing_columns(str, 4), [("a🦀b", 4), ("", 0)]);

    //     let str = "a\0b\0🦀\0c\0";
    //     // assert_eq!(trailing_columns(str, 1), [("c\0", 1), ("", 0)]);
    //     // assert_eq!(trailing_columns(str, 2), [("b\0", 1), ("🦀\0", 2)]);
    //     // assert_eq!(trailing_columns(str, 3), [("🦀\0b\0", 3), ("", 0)]);
    //     // assert_eq!(trailing_columns(str, 5), [("a\0🦀\0b\0", 4), ("", 0)]);
    //     // assert_eq!(trailing_columns(str, 4), [("a\0🦀\0b\0", 4), ("", 0)]);
    // }

    #[test]
    fn columns2() {
        let str = "a🦀b";
        let columns = Columns::new(str);
        assert_eq!(
            columns.collect::<Vec<_>>(),
            vec![("a", 1), ("🦀", 2), ("b", 1)]
        );

        let str = "a\0🦀\0b\0";
        let columns = Columns::new(str);
        assert_eq!(
            columns.collect::<Vec<_>>(),
            vec![("a\0", 1), ("🦀\0", 2), ("b\0", 1)]
        );
    }
}

struct Columns<'a> {
    str: &'a str,
}

impl<'a> Columns<'a> {
    fn new(str: &'a str) -> Self {
        debug_assert!(!str.contains('\n'));
        debug_assert!(
            str.chars()
                .next()
                .map(|char| char.width().unwrap_or_default())
                != Some(0)
        );

        Self { str }
    }
}

impl<'a> Iterator for Columns<'a> {
    type Item = (&'a str, u16);

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.str.char_indices();
        let width = chars.next()?.1.width().unwrap_or_default() as u16;
        debug_assert!(width != 0);

        let (item, str) = match chars
            .skip_while(|(i, c)| c.width().unwrap_or_default() == 0)
            .next()
        {
            Some((i, _)) => self.str.split_at(i),
            _ => (self.str, ""),
        };

        self.str = str;
        Some((item, width))
    }
}

impl<'a> DoubleEndedIterator for Columns<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut chars = self.str.char_indices().rev();

        let (i, char) = chars
            .skip_while(|(i, c)| c.width().unwrap_or_default() == 0)
            .next()?;

        let (str, _) = self.str.split_at(i + char.len_utf8());
        let (str, item) = str.split_at(i);

        None
    }
}
