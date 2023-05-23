#![allow(unused)]

use std::time::Instant;
use tender::{
    line::Line,
    row::Row,
    style::{Color, Style},
};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

fn main() {
    let str1 = include_str!("../test.txt");
    let str2 = "\u{200C}ab\u{200C}cde\u{200C}fghijklmno\u{200C}pqrstuvwxyz\u{200C}\n\u{200C}ABCDEFGHIJKLMNOPQ\u{200C}RSTUVWXYZ{[(<>)]},?;.:/!§\u{200C}";

    fn test(str: &str, mul: usize) {
        let mut string = String::from(str);

        for _ in 0..mul {
            string.push_str(str);
        }

        let now = Instant::now();
        let tender_graphemes = tender::unicode::graphemes::Graphemes::new(&string);
        let tender_count = tender_graphemes.count();
        dbg!(now.elapsed());

        let now = Instant::now();
        let unicode_graphemes = string.graphemes(true);
        let unicode_count = unicode_graphemes.count();
        dbg!(now.elapsed());

        assert!(tender_count == unicode_count);
    }

    test(str1, 100);
    test(str2, 1_000_000);

    let a = collect(str1.graphemes(true));
    let b = collect(tender::unicode::graphemes::Graphemes::new(str1));
    assert!(a == b);
    let a = collect(str2.graphemes(true));
    let b = collect(tender::unicode::graphemes::Graphemes::new(str2));
    assert!(a == b);
}

fn main2() {
    let str = include_str!("../test.txt");
    let str = [str, "👩\u{200D}🔬"].concat();
    dbg!(collect(
        str.graphemes(true)
            .filter(|g| g.chars().count() > 2)
            .filter(|g| {
                let mut chars = collect(g.chars());
                chars.pop();
                chars
                    .into_iter()
                    .collect::<String>()
                    .graphemes(true)
                    .count()
                    == 1
            })
    ));
    // dbg!(collect(s.chars()));

    // Splitting grapheme don't make more graphemes!

    let red = "\u{1b}[0;31m";
    let green = "\u{1b}[0;32m";
    let blue = "\u{1b}[0;34m";
    let s = "म\u{947}\u{902}";
    println!("{s}");
    println!("{red}म{green}\u{947}{blue}\u{902}");
}

fn main3() {
    let str = include_str!("../test.txt");

    // let mut count = 0;
    // for str in str.lines() {
    //     for (i, _) in str.char_indices() {
    //         let mut line = Line::new(0);
    //         line.push(&str[..i]);
    //         line.push(&str[i..]);

    //         assert!(line.str == str);
    //         assert!(line.width == width(str));
    //         count += 1;
    //     }
    // }
    // println!("{count}");

    let now = Instant::now();
    let mut width = 0;
    for _ in 0..100 {
        for str in str.lines() {
            width += str
                .graphemes(true)
                .map(|grapheme| grapheme.width().max(2))
                .sum::<usize>();
        }
    }
    println!("graphemes: {:?}", now.elapsed());

    let now = Instant::now();
    let mut width = 0;
    for _ in 0..100 {
        for str in str.lines() {
            width += str.width();
        }
    }
    println!("width: {:?}", now.elapsed());

    fn width(str: &str) -> u16 {
        str.graphemes(true)
            .map(|grapheme| grapheme.width().max(2) as u16)
            .sum()
    }
}

// fn main2() {
//     let red = Style {
//         foreground: Color { r: 255, g: 0, b: 0 },
//         ..Default::default()
//     };
//     let green = Style {
//         foreground: Color { r: 0, g: 255, b: 0 },
//         ..Default::default()
//     };
//     let blue = Style {
//         foreground: Color { r: 0, g: 0, b: 255 },
//         ..Default::default()
//     };

//     let mut row = Row::new(3, red);
//     dbg!(collect(row.cells()));
//     row.paint(0, "🦀", green);
//     dbg!(collect(row.cells()));
//     row.paint(1, "🦀", blue);
//     dbg!(collect(row.cells()));

//     let mut row = Row::new(4, red);
//     dbg!(collect(row.cells()));
//     row.paint(0, "👩", green);
//     dbg!(collect(row.cells()));
//     row.paint(2, "\u{200D}🔬", blue);
//     dbg!(collect(row.cells())); // WRONG

//     let mut row = Row::new(4, red);
//     dbg!(collect(row.cells()));
//     row.paint(0, "👩", green);
//     dbg!(collect(row.cells()));
//     row.paint(2, "🔬", blue);
//     dbg!(collect(row.cells()));
//     row.paint(2, "\u{200D}", blue);
//     dbg!(collect(row.cells())); // PANIC
// }

fn collect<I: IntoIterator>(i: I) -> Vec<I::Item> {
    i.into_iter().collect()
}
