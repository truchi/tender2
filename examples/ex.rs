#![allow(unused)]

use tender::{
    line::Line,
    row::Row,
    style::{Color, Style},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn main() {
    let str = include_str!("input.txt");
    let mut count = 0;

    for str in str.lines() {
        for (i, _) in str.char_indices() {
            let mut line = Line::new(0);
            line.push(&str[..i]);
            line.push(&str[i..]);

            assert!(line.str == str);
            assert!(line.width == width(str));
            count += 1;
        }
    }

    println!("{count}");

    fn width(str: &str) -> u16 {
        str.graphemes(true)
            .map(|grapheme| grapheme.width().max(2) as u16)
            .sum()
    }
}

fn main2() {
    let red = Style {
        foreground: Color { r: 255, g: 0, b: 0 },
        ..Default::default()
    };
    let green = Style {
        foreground: Color { r: 0, g: 255, b: 0 },
        ..Default::default()
    };
    let blue = Style {
        foreground: Color { r: 0, g: 0, b: 255 },
        ..Default::default()
    };

    let mut row = Row::new(3, red);
    dbg!(collect(row.cells()));
    row.paint(0, "🦀", green);
    dbg!(collect(row.cells()));
    row.paint(1, "🦀", blue);
    dbg!(collect(row.cells()));

    let mut row = Row::new(4, red);
    dbg!(collect(row.cells()));
    row.paint(0, "👩", green);
    dbg!(collect(row.cells()));
    row.paint(2, "‍🔬", blue);
    dbg!(collect(row.cells())); // WRONG

    let mut row = Row::new(4, red);
    dbg!(collect(row.cells()));
    row.paint(0, "👩", green);
    dbg!(collect(row.cells()));
    row.paint(2, "🔬", blue);
    dbg!(collect(row.cells()));
    row.paint(2, "‍", blue);
    dbg!(collect(row.cells())); // PANIC
}

fn collect<I: IntoIterator>(i: I) -> Vec<I::Item> {
    i.into_iter().collect()
}
