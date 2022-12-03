use super::*;
use std::ops::Range;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Scan {
    /// Bytes offset of the [`Segment`].
    pub i: usize,
    /// Width offset of the [`Segment`].
    pub x: u16,
    /// Index of the [`Segment`].
    pub index: usize,
    /// The [`Segment`].
    pub segment: Segment,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Segment {
    /// Length.
    pub len: usize,
    /// Width.
    pub width: u16,
    /// Style.
    pub style: Style,
}

impl Segment {
    pub fn scan(segments: &Vec<Self>, x: u16, width: u16) -> Result<Range<Scan>, ()> {
        // Accumulate segments index and width
        let scans =
            segments
                .iter()
                .copied()
                .enumerate()
                .scan((0, 0), |(i, x), (index, segment)| {
                    let scan = Scan {
                        i: *i,
                        x: *x,
                        index,
                        segment,
                    };
                    *i += segment.len;
                    *x += segment.width;
                    Some(scan)
                });

        // Skip segments ending before `x`
        let skipped = scans.skip_while(|scan| scan.x + scan.segment.width <= x);
        let start = skipped.clone().next().ok_or(())?;

        // Take segments ending before `x + width`
        let end = skipped
            .skip_while(|scan| scan.x + scan.segment.width < x + width)
            .next()
            .ok_or(())?;

        Ok(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn scan() {
        let seg = |width| Segment {
            len: 0,
            width,
            style: default(),
        };

        // -- --- ---- --
        let segments = vec![seg(2), seg(3), seg(4), seg(2)];

        let scan = |x, width| Segment::scan(&segments, x, width);
        let indexes = |range: Range<Scan>| range.start.index..range.end.index;

        // -
        // -- --- ---- --
        assert_eq!(indexes(scan(0, 1).unwrap()), 0..0);
        // --
        // -- --- ---- --
        assert_eq!(indexes(scan(0, 2).unwrap()), 0..0);
        //  -
        // -- --- ---- --
        assert_eq!(indexes(scan(1, 1).unwrap()), 0..0);
        //  - -
        // -- --- ---- --
        assert_eq!(indexes(scan(1, 2).unwrap()), 0..1);
        //  - ---
        // -- --- ---- --
        assert_eq!(indexes(scan(1, 4).unwrap()), 0..1);
        //  - --- -
        // -- --- ---- --
        assert_eq!(indexes(scan(1, 5).unwrap()), 0..2);
        //    ---
        // -- --- ---- --
        assert_eq!(indexes(scan(2, 3).unwrap()), 1..1);
        //     -- --
        // -- --- ---- --
        assert_eq!(indexes(scan(3, 4).unwrap()), 1..2);
        //         --
        // -- --- ---- --
        assert_eq!(indexes(scan(6, 2).unwrap()), 2..2);
        //             -
        // -- --- ---- --
        assert_eq!(indexes(scan(9, 1).unwrap()), 3..3);
        //             --
        // -- --- ---- --
        assert_eq!(indexes(scan(9, 2).unwrap()), 3..3);
        //              -
        // -- --- ---- --
        assert_eq!(indexes(scan(10, 1).unwrap()), 3..3);
        //              --
        // -- --- ---- --
        assert!(scan(10, 2).is_err());
        //               --
        // -- --- ---- --
        assert!(scan(11, 2).is_err());
    }
}
