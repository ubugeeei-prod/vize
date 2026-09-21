//! Provenance through an oxc re-print.
//!
//! Stripping TypeScript re-prints a module from its AST, so no byte run
//! survives. oxc's codegen records where each printed node came from; this
//! turns those (line, UTF-16 column) tokens back into byte offsets so a
//! re-printed module keeps token-level provenance.

use oxc_sourcemap::SourceMap;

use super::runs::Runs;

/// Line starts under oxc's line-break rules (`\n`, `\r\n`, `\r`, U+2028,
/// U+2029), so a (line, column) pair from its codegen resolves back to the
/// same byte.
struct Lines<'a> {
    text: &'a str,
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    /// Lines split at `\n` only, as `SourceMapBuilder` resolves them.
    fn newline_only(text: &'a str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(index, _)| index + 1));
        Self { text, starts }
    }

    fn new(text: &'a str) -> Self {
        let mut starts = vec![0];
        let mut chars = text.char_indices().peekable();
        while let Some((index, ch)) = chars.next() {
            match ch {
                '\r' if chars.peek().is_some_and(|&(_, next)| next == '\n') => {}
                '\n' | '\r' | '\u{2028}' | '\u{2029}' => starts.push(index + ch.len_utf8()),
                _ => {}
            }
        }
        Self { text, starts }
    }

    /// Byte offset of a 0-based line and UTF-16 column, if it names a character
    /// on that line.
    fn offset(&self, line: u32, column: u32) -> Option<usize> {
        let start = *self.starts.get(line as usize)?;
        let end = self
            .starts
            .get(line as usize + 1)
            .copied()
            .unwrap_or(self.text.len());
        let mut units = 0u32;
        for (index, ch) in self.text[start..end].char_indices() {
            if units == column {
                return Some(start + index);
            }
            if units > column {
                return None;
            }
            units += ch.len_utf16() as u32;
        }
        (units == column && end == self.text.len()).then_some(end)
    }
}

/// The provenance of `printed` in `parsed` recorded by an oxc codegen map:
/// one point per printed token.
pub(crate) fn reprint_points(map: &SourceMap<'_>, printed: &str, parsed: &str) -> Runs {
    let printed_lines = Lines::new(printed);
    let parsed_lines = Lines::new(parsed);
    let mut points: Vec<(usize, usize)> = map
        .get_tokens()
        .filter_map(|token| {
            let out = printed_lines.offset(token.get_dst_line(), token.get_dst_col())?;
            let src = parsed_lines.offset(token.get_src_line(), token.get_src_col())?;
            Some((out, src))
        })
        .collect();
    points.sort_unstable();
    points.dedup_by_key(|point| point.0);
    let mut runs = Runs::default();
    for (out, src) in points {
        runs.point(out, src);
    }
    runs
}

/// Carry a module map (Source Map v3 JSON in `.vue` offsets, as the SFC
/// compiler emits it for `pre`) through an oxc re-print to `post` whose token
/// provenance is `reprint`: a re-printed token keeps the mapping `map` has at
/// exactly the byte it was printed from.
pub(crate) fn remap_reprinted(
    map: &serde_json::Value,
    pre: &str,
    post: &str,
    reprint: &Runs,
) -> Option<serde_json::Value> {
    let source = map["sourcesContent"][0].as_str()?;
    let filename = map["sources"][0].as_str()?;
    let json = serde_json::to_string(map).ok()?;
    let parsed = SourceMap::from_json_string(&json).ok()?;
    let (pre_lines, source_lines) = (Lines::newline_only(pre), Lines::newline_only(source));
    let origins: vize_carton::FxHashMap<usize, usize> = parsed
        .get_tokens()
        .filter_map(|token| {
            let generated = pre_lines.offset(token.get_dst_line(), token.get_dst_col())?;
            let origin = source_lines.offset(token.get_src_line(), token.get_src_col())?;
            Some((generated, origin))
        })
        .collect();
    let mut remapped = Runs::default();
    for &(out, at) in reprint.points() {
        if let Some(&origin) = origins.get(&at) {
            remapped.point(out, origin);
        }
    }
    super::module_map_value(post, remapped, filename, source)
}

#[cfg(test)]
mod tests {
    use super::Lines;

    #[test]
    fn lines_follow_oxc_line_breaks_and_utf16_columns() {
        let text = "a\r\nbé\u{1F600}x\ry\u{2028}z";
        let lines = Lines::new(text);
        assert_eq!(lines.offset(0, 0), Some(0));
        assert_eq!(lines.offset(1, 0), Some(3));
        assert_eq!(lines.offset(1, 2), Some(text.find('\u{1F600}').unwrap()));
        assert_eq!(lines.offset(1, 4), Some(text.find('x').unwrap()));
        assert_eq!(lines.offset(1, 3), None, "inside a surrogate pair");
        assert_eq!(lines.offset(2, 0), Some(text.find('y').unwrap()));
        assert_eq!(lines.offset(3, 0), Some(text.find('z').unwrap()));
        assert_eq!(lines.offset(4, 0), None);
    }
}
