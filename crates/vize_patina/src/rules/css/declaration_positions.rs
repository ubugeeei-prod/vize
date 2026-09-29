//! Source spans for declarations in a Lightning CSS style rule.

use lightningcss::rules::style::StyleRule;

pub(super) struct DeclarationPositions<'a> {
    declarations: Vec<(&'a str, usize, usize)>,
    used: Vec<bool>,
}

#[expect(
    clippy::string_slice,
    clippy::indexing_slicing,
    reason = "the scanner checks bounds and slices only at ASCII delimiters"
)]
impl<'a> DeclarationPositions<'a> {
    pub(super) fn new(source: &'a str, rule: &StyleRule<'_>) -> Self {
        let line_start = if rule.loc.line == 0 {
            0
        } else {
            source
                .match_indices('\n')
                .nth(rule.loc.line as usize - 1)
                .map_or(0, |(index, _)| index + 1)
        };
        let column = rule.loc.column.saturating_sub(1) as usize;
        let start = source[line_start..]
            .char_indices()
            .scan(0, |units, (index, ch)| {
                let previous = *units;
                *units += ch.len_utf16();
                Some((index, previous))
            })
            .find(|(_, units)| *units >= column)
            .map_or(source.len(), |(index, _)| line_start + index);
        let bytes = source.as_bytes();
        let mut open = start;
        while open < bytes.len() && bytes[open] != b'{' {
            open += 1;
        }
        let mut declarations = Vec::new();
        let mut segment = open.saturating_add(1);
        let mut depth = 1usize;
        let mut i = segment;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'/' if bytes.get(i + 1) == Some(&b'*') => {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    i = (i + 2).min(bytes.len());
                }
                b'\'' | b'"' => {
                    let quote = bytes[i];
                    i += 1;
                    while i < bytes.len() && bytes[i] != quote {
                        if bytes[i] == b'\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                    i = (i + 1).min(bytes.len());
                }
                b'{' => {
                    depth += 1;
                    segment = i + 1;
                    i += 1;
                }
                b'}' => {
                    if depth == 1 {
                        Self::record(source, segment, i, &mut declarations);
                    }
                    depth -= 1;
                    segment = i + 1;
                    i += 1;
                }
                b';' if depth == 1 => {
                    Self::record(source, segment, i, &mut declarations);
                    segment = i + 1;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        let used = vec![false; declarations.len()];
        Self { declarations, used }
    }

    fn record(source: &'a str, start: usize, end: usize, out: &mut Vec<(&'a str, usize, usize)>) {
        let mut start = start;
        while start < end {
            start += source[start..end].len() - source[start..end].trim_start().len();
            if !source[start..end].starts_with("/*") {
                break;
            }
            let Some(close) = source[start + 2..end].find("*/") else {
                return;
            };
            start += close + 4;
        }
        let Some(colon) = source[start..end].find(':') else {
            return;
        };
        let before = &source[start..start + colon];
        let name = before.trim();
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return;
        }
        let name_start = start + before.len() - before.trim_start().len();
        out.push((name, name_start, name_start + name.len()));
    }

    pub(super) fn take(&mut self, name: &str, offset: usize) -> Option<(u32, u32)> {
        for (index, (candidate, start, end)) in self.declarations.iter().enumerate() {
            if !self.used[index] && candidate.eq_ignore_ascii_case(name) {
                self.used[index] = true;
                return Some(((offset + start) as u32, (offset + end) as u32));
            }
        }
        None
    }
}
