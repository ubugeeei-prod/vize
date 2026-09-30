use alloc::vec::Vec;
use vize_l0::Span;

use super::{EmitDocument, SpanLink};

impl EmitDocument {
    /// Append another document with every occurrence of each `(from, to)`
    /// pattern replaced, rebasing its links onto the escaped output. Patterns
    /// are ASCII and tried in order at each byte; a link boundary at an
    /// escaped byte lands before its replacement.
    #[inline(always)]
    pub fn push_escaped(&mut self, other: &EmitDocument, escapes: &[(&str, &str)]) {
        let base = self.text.len();
        let bytes = other.text.as_bytes();
        // (input start, input end, cumulative signed length change)
        let mut growth: Vec<(usize, usize, isize)> = Vec::new();
        let (mut start, mut index) = (0, 0);
        while let Some(rest) = bytes.get(index..).filter(|rest| !rest.is_empty()) {
            let Some((from, to)) = escapes
                .iter()
                .find(|(from, _)| !from.is_empty() && rest.starts_with(from.as_bytes()))
            else {
                index += 1;
                continue;
            };
            self.text
                .push_str(other.text.get(start..index).unwrap_or_default());
            self.text.push_str(to);
            if self.recording && !other.links.is_empty() {
                let total = growth.last().map_or(0, |&(_, _, total)| total);
                growth.push((
                    index,
                    index + from.len(),
                    total + to.len() as isize - from.len() as isize,
                ));
            }
            index += from.len();
            start = index;
        }
        self.text
            .push_str(other.text.get(start..).unwrap_or_default());
        if !self.recording {
            return;
        }
        let out = |offset: u32| {
            let before = growth.partition_point(|&(at, _, _)| at < offset as usize);
            let shifted = match before.checked_sub(1).and_then(|last| growth.get(last)) {
                Some(&(at, end, _)) if (offset as usize) < end => {
                    let previous = before
                        .checked_sub(2)
                        .and_then(|last| growth.get(last))
                        .map_or(0, |&(_, _, total)| total);
                    at.saturating_add_signed(previous)
                }
                Some(&(_, _, total)) => (offset as usize).saturating_add_signed(total),
                None => offset as usize,
            };
            (base + shifted) as u32
        };
        self.links.extend(other.links.iter().map(|link| SpanLink {
            generated: Span::new(out(link.generated.start), out(link.generated.end)),
            ..link.clone()
        }));
    }
}
