use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Span {
    pub width: u16,
    pub style: Style,
}

impl Span {}

pub type Iter<'a> = std::slice::Iter<'a, Span>;
pub type IterMut<'a> = std::slice::IterMut<'a, Span>;
pub type Scan<'a> = std::iter::Scan<
    std::iter::Enumerate<Iter<'a>>,
    u16,
    fn(&mut u16, (usize, &'a Span)) -> Option<(usize, u16, &'a Span)>,
>;
pub type ScanMut<'a> = std::iter::Scan<
    std::iter::Enumerate<IterMut<'a>>,
    u16,
    fn(&mut u16, (usize, &'a mut Span)) -> Option<(usize, u16, &'a mut Span)>,
>;

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Spans(Vec<Span>);

impl Spans {
    pub fn width(&self) -> u16 {
        self.iter().map(|span| span.width).sum()
    }

    pub fn iter(&self) -> Iter {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut {
        self.0.iter_mut()
    }

    pub fn scan(&self) -> Scan {
        self.iter().enumerate().scan(0, |column, (index, span)| {
            let c = *column;
            *column += span.width;
            Some((index, c, span))
        })
    }

    pub fn scan_mut(&mut self) -> ScanMut {
        self.iter_mut()
            .enumerate()
            .scan(0, |column, (index, span)| {
                let c = *column;
                *column += span.width;
                Some((index, c, span))
            })
    }

    // pub fn get(&self, column: u16) -> Option<(usize, u16, &Span)> {
    //     self.iter()
    //         .enumerate()
    //         .scan(0, |c, (i, span)| {
    //             let column = *c;
    //             *c += span.width;
    //             Some((i, column, span))
    //         })
    //         .find(|(i, c, span)| column < c + span.width)
    // }

    // pub fn get_mut(&mut self, column: u16) -> Option<(usize, u16, &mut Span)> {
    //     self.iter_mut()
    //         .enumerate()
    //         .scan(0, |c, (i, span)| {
    //             let column = *c;
    //             *c += span.width;
    //             Some((i, column, span))
    //         })
    //         .find(|(i, c, span)| column < c + span.width)
    // }

    // TODO handle same consecutive styles
    pub fn paint(&mut self, column: u16, span: Span) {
        let mut last = (0, 0, &default());
        let mut scan = self
            .scan()
            .inspect(|(i, c, s)| last = (*i, *c, *s))
            .skip_while(|(i, c, s)| c + s.width <= column);

        let start = if let Some(start) = scan.next() {
            start
        } else {
            return;
        };

        let end = if column + span.width <= start.1 + start.2.width {
            start
        } else {
            scan.skip_while(|(i, c, s)| c + s.width < column + span.width)
                .next()
                .unwrap_or(last)
        };

        // Not growing in width!
        let span = {
            let mut span = span;
            span.width = span.width.min(last.1 + last.2.width - column);
            span
        };

        //     ___ ... ___
        // ___ ___ ... ___ ___
        if start.1 == column && end.1 + end.2.width == column + span.width {
            self.0.splice(start.0..=end.0, [span]);
        }
        //     ___ ... __
        // ___ ___ ... ___ ___
        else if start.1 == column {
            self.0.splice(
                start.0..=end.0,
                [
                    span,
                    Span {
                        width: (end.1 + end.2.width) - (column + span.width),
                        style: end.2.style,
                    },
                ],
            );
        }
        //      __ ... ___
        // ___ ___ ... ___ ___
        else if end.1 + end.2.width == column + span.width {
            self.0.splice(
                start.0..=end.0,
                [
                    Span {
                        width: column - start.1,
                        style: start.2.style,
                    },
                    span,
                ],
            );
        }
        //      __ ... __
        // ___ ___ ... ___ ___
        else {
            self.0.splice(
                start.0..=end.0,
                [
                    Span {
                        width: column - start.1,
                        style: start.2.style,
                    },
                    span,
                    Span {
                        width: (end.1 + end.2.width) - (column + span.width),
                        style: end.2.style,
                    },
                ],
            );
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
    const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };

    fn new_span(width: u16, color: Color) -> Span {
        Span {
            width,
            style: Style {
                foreground: color,
                ..default()
            },
        }
    }

    fn new_spans(spans: impl IntoIterator<Item = (u16, Color)>) -> Spans {
        Spans(
            spans
                .into_iter()
                .map(|(width, color)| new_span(width, color))
                .collect(),
        )
    }

    #[test]
    fn scan() {
        let spans = new_spans([(1, RED), (2, GREEN), (3, BLUE)]);
        let scan = spans.scan().collect::<Vec<_>>();

        assert_eq!(
            scan,
            [
                (0, 0, &new_span(1, RED)),
                (1, 1, &new_span(2, GREEN)),
                (2, 3, &new_span(3, BLUE)),
            ]
        );

        let mut spans = new_spans([(1, RED), (2, GREEN), (3, BLUE)]);
        let scan = spans.scan_mut().collect::<Vec<_>>();

        assert_eq!(
            scan,
            [
                (0, 0, &mut new_span(1, RED)),
                (1, 1, &mut new_span(2, GREEN)),
                (2, 3, &mut new_span(3, BLUE)),
            ]
        );
    }

    #[test]
    fn paint_1() {
        // ___
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(0, new_span(3, YELLOW));

        assert_eq!(spans, new_spans([(3, YELLOW), (3, GREEN), (3, BLUE)]));

        //     ___
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(3, new_span(3, YELLOW));

        assert_eq!(spans, new_spans([(3, RED), (3, YELLOW), (3, BLUE)]));

        //         ___
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(6, new_span(3, YELLOW));

        assert_eq!(spans, new_spans([(3, RED), (3, GREEN), (3, YELLOW)]));
    }

    #[test]
    fn paint_2() {
        // __
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(0, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(2, YELLOW), (1, RED), (3, GREEN), (3, BLUE)])
        );

        //     __
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(3, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (2, YELLOW), (1, GREEN), (3, BLUE)])
        );

        //         __
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(6, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (3, GREEN), (2, YELLOW), (1, BLUE)])
        );
    }

    #[test]
    fn paint_3() {
        //  __
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(1, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(1, RED), (2, YELLOW), (3, GREEN), (3, BLUE)])
        );

        //      __
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(4, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (1, GREEN), (2, YELLOW), (3, BLUE)])
        );

        //          __
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(7, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (3, GREEN), (1, BLUE), (2, YELLOW)])
        );
    }

    #[test]
    fn paint_4() {
        //    __
        // ______ ______ ______

        let mut spans = new_spans([(6, RED), (6, GREEN), (6, BLUE)]);
        spans.paint(3, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (2, YELLOW), (1, RED), (6, GREEN), (6, BLUE)])
        );

        //          _
        // ______ ______ ______

        let mut spans = new_spans([(6, RED), (6, GREEN), (6, BLUE)]);
        spans.paint(8, new_span(1, YELLOW));

        assert_eq!(
            spans,
            new_spans([(6, RED), (2, GREEN), (1, YELLOW), (3, GREEN), (6, BLUE)])
        );

        //                __
        // ______ ______ ______

        let mut spans = new_spans([(6, RED), (6, GREEN), (6, BLUE)]);
        spans.paint(13, new_span(2, YELLOW));

        assert_eq!(
            spans,
            new_spans([(6, RED), (6, GREEN), (1, BLUE), (2, YELLOW), (3, BLUE)])
        );
    }

    #[test]
    fn paint_5() {
        //     _ _
        // ___ _ _ ___

        let mut spans = new_spans([(3, RED), (1, GREEN), (1, RED), (3, BLUE)]);
        spans.paint(3, new_span(2, YELLOW));

        assert_eq!(spans, new_spans([(3, RED), (2, YELLOW), (3, BLUE)]));

        //     _ __ _
        // ___ _ __ _ ___

        let mut spans = new_spans([(3, RED), (1, GREEN), (2, BLUE), (1, GREEN), (3, RED)]);
        spans.paint(3, new_span(4, YELLOW));

        assert_eq!(spans, new_spans([(3, RED), (4, YELLOW), (3, RED)]));
    }

    #[test]
    fn paint_6() {
        //     ___ _
        // ___ ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE), (3, RED)]);
        spans.paint(3, new_span(4, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (4, YELLOW), (2, BLUE), (3, RED)])
        );

        //     ___ __ _
        // ___ ___ __ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (2, BLUE), (3, GREEN), (3, RED)]);
        spans.paint(3, new_span(6, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (6, YELLOW), (2, GREEN), (3, RED)])
        );
    }

    #[test]
    fn paint_7() {
        //      __ ___
        // ___ ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE), (3, RED)]);
        spans.paint(4, new_span(5, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (1, GREEN), (5, YELLOW), (3, RED)])
        );

        //      __ __ ___
        // ___ ___ __ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (2, BLUE), (3, GREEN), (3, RED)]);
        spans.paint(4, new_span(7, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (1, GREEN), (7, YELLOW), (3, RED)])
        );
    }

    #[test]
    fn paint_8() {
        //      __ __
        // ___ ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE), (3, RED)]);
        spans.paint(4, new_span(4, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (1, GREEN), (4, YELLOW), (1, BLUE), (3, RED)])
        );

        //      __ __ __
        // ___ ___ __ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (2, BLUE), (3, GREEN), (3, RED)]);
        spans.paint(4, new_span(6, YELLOW));

        assert_eq!(
            spans,
            new_spans([(3, RED), (1, GREEN), (6, YELLOW), (1, GREEN), (3, RED)])
        );
    }

    #[test]
    fn paint_9() {
        //      __ _____
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(4, new_span(7, YELLOW));

        assert_eq!(spans, new_spans([(3, RED), (1, GREEN), (5, YELLOW)]));

        //     ___ _____
        // ___ ___ ___

        let mut spans = new_spans([(3, RED), (3, GREEN), (3, BLUE)]);
        spans.paint(3, new_span(8, YELLOW));

        assert_eq!(spans, new_spans([(3, RED), (6, YELLOW)]));
    }
}
