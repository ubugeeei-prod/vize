//! Sparse UTF16/byte coordinates using the CSS tokenizer's newline model.

struct WideEnd {
    byte: usize,
    unit: usize,
}

struct Line {
    start: usize,
    end: usize,
    units: usize,
    wide: std::ops::Range<usize>,
}

pub(super) struct Positions<'a> {
    text: &'a str,
    lines: Vec<Line>,
    wide: Vec<WideEnd>,
}

impl<'a> Positions<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        let mut lines = Vec::new();
        let mut wide = Vec::new();
        let (mut start, mut units, mut wide_start) = (0, 0, 0);
        let mut chars = text.char_indices().peekable();
        while let Some((at, ch)) = chars.next() {
            if matches!(ch, '\r' | '\n' | '\u{c}') {
                let mut next_start = at + 1;
                if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                    let _ = chars.next();
                    next_start += 1;
                }
                lines.push(Line {
                    start,
                    end: at,
                    units,
                    wide: wide_start..wide.len(),
                });
                start = next_start;
                units = 0;
                wide_start = wide.len();
            } else {
                units += ch.len_utf16();
                if !ch.is_ascii() {
                    wide.push(WideEnd {
                        byte: at + ch.len_utf8() - start,
                        unit: units,
                    });
                }
            }
        }
        lines.push(Line {
            start,
            end: text.len(),
            units,
            wide: wide_start..wide.len(),
        });
        Self { text, lines, wide }
    }

    pub(super) fn byte(&self, line: u32, column: u32) -> Option<usize> {
        let line = self.lines.get(line as usize)?;
        let column = column as usize;
        if column > line.units {
            return None;
        }
        let wide = self.wide.get(line.wide.clone())?;
        let end = wide.partition_point(|point| point.unit <= column);
        let correction = end
            .checked_sub(1)
            .and_then(|at| wide.get(at))
            .map_or(0, |point| point.byte - point.unit);
        let at = line.start + column + correction;
        (at <= line.end && self.text.is_char_boundary(at)).then_some(at)
    }

    pub(super) fn location(&self, byte: usize) -> Option<(u32, u32)> {
        if byte > self.text.len() || !self.text.is_char_boundary(byte) {
            return None;
        }
        let index = self
            .lines
            .partition_point(|line| line.start <= byte)
            .checked_sub(1)?;
        let line = self.lines.get(index)?;
        if byte > line.end {
            return None;
        }
        let within = byte - line.start;
        let wide = self.wide.get(line.wide.clone())?;
        let end = wide.partition_point(|point| point.byte <= within);
        let correction = end
            .checked_sub(1)
            .and_then(|at| wide.get(at))
            .map_or(0, |point| point.byte - point.unit);
        Some((
            u32::try_from(index).ok()?,
            u32::try_from(within - correction).ok()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Positions;

    #[test]
    fn unicode_css_newlines_and_surrogate_interiors_have_exact_independent_coordinates() {
        let source = "a雪😀z\r\nq😀\rr雪\u{c}s";
        let positions = Positions::new(source);
        for (line, column, byte) in [
            (0, 0, 0),
            (0, 1, 1),
            (0, 2, 4),
            (0, 4, 8),
            (0, 5, 9),
            (1, 0, 11),
            (1, 1, 12),
            (1, 3, 16),
            (2, 0, 17),
            (2, 1, 18),
            (2, 2, 21),
            (3, 0, 22),
            (3, 1, 23),
        ] {
            assert_eq!(positions.byte(line, column), Some(byte));
            assert_eq!(positions.location(byte), Some((line, column)));
        }
        assert_eq!(positions.byte(0, 3), None);
        assert_eq!(positions.byte(1, 2), None);
        assert_eq!(positions.byte(0, 6), None);
        assert_eq!(positions.byte(4, 0), None);
        assert_eq!(positions.location(10), None);
        assert_eq!(positions.location(5), None);
        assert_eq!(positions.location(24), None);
    }
}
