use super::*;

#[derive(Clone, Default, Debug)]
pub struct Canvas {
    rows: Vec<Row>,
}

impl Canvas {
    pub fn new(rows: Vec<Row>) -> Self {
        debug_assert!(rows
            .windows(2)
            .all(|rows| rows[0].width() == rows[1].width()));

        Self { rows }
    }

    /// Returns the width of this [`Canvas`].
    pub fn width(&self) -> u16 {
        self.rows.get(0).map(|row| row.width()).unwrap_or_default()
    }

    pub fn paint(&mut self, line: usize, column: u16, str: &str, style: Style) {
        self.rows
            .get_mut(line)
            .map(|line| line.paint(column, str, style));
    }
}
