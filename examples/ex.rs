use tender::{
    row::Row,
    style::{Color, Style},
};

fn main() {
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
