use vize_carton::line_index::LineBreaks;

impl super::DiagnosticMapper<'_> {
    pub(super) fn virtual_offset(
        &mut self,
        file: &super::VirtualFile,
        line: u32,
        column: u32,
    ) -> Option<u32> {
        if let Some(index) = self.virtual_line_indexes.get(&file.virtual_path) {
            return index.line_col_to_offset(&file.content, line, column);
        }
        let index = LineIndex::for_backend(&file.content, self.virtual_line_breaks);
        let offset = index.line_col_to_offset(&file.content, line, column);
        self.virtual_line_indexes
            .insert(file.virtual_path.clone(), index);
        offset
    }
}

pub(super) struct LineIndex {
    starts: vize_carton::SmallVec<[usize; 8]>,
    len: usize,
    backend: bool,
}

impl LineIndex {
    #[cfg(test)]
    pub(super) fn new(content: &str) -> Self {
        let mut starts = vize_carton::smallvec![0];
        for (index, byte) in content.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }

        Self {
            starts,
            len: content.len(),
            backend: false,
        }
    }

    /// Compiler and LSP line positions use distinct Unicode-break conventions.
    /// The same convention is used when reporting authored source positions.
    pub(super) fn for_backend(content: &str, breaks: LineBreaks) -> Self {
        let starts = breaks.line_starts(content).collect();
        Self {
            starts,
            len: content.len(),
            backend: true,
        }
    }

    /// Convert an LSP (line, character) — where character is in UTF-16 code
    /// units — back to a byte offset into `content`. (#965)
    pub(super) fn line_col_to_offset(&self, content: &str, line: u32, col: u32) -> Option<u32> {
        let line = usize::try_from(line).ok()?;
        let start = *self.starts.get(line)?;
        let end = self.line_end(content, line);
        let offset = vize_carton::line_index::utf16_offset(&content[start..end], col)?;
        u32::try_from(start + offset).ok()
    }

    /// Convert a byte offset to LSP (line, character). `character` is in
    /// UTF-16 code units — astral characters (`len_utf16() == 2`) count as
    /// two so the column matches what `vue-tsc` / `@vue/language-tools`
    /// report. (#965)
    pub(super) fn offset_to_line_col(&self, content: &str, offset: u32) -> Option<(u32, u32)> {
        let offset = usize::try_from(offset).ok()?;
        if offset > self.len {
            return None;
        }

        let line = self.starts.partition_point(|start| *start <= offset);
        let line = line.saturating_sub(1);
        let start = *self.starts.get(line)?;
        let end = self.line_end(content, line);
        let mut boundary = offset.min(end);
        while !content.is_char_boundary(boundary) {
            boundary += 1;
        }
        let col = vize_carton::line_index::utf16_len(&content[start..boundary]);
        Some((u32::try_from(line).ok()?, u32::try_from(col).ok()?))
    }

    fn line_end(&self, content: &str, line: usize) -> usize {
        let Some(&next_start) = self.starts.get(line + 1) else {
            return self.len;
        };
        let prefix = &content[..next_start];
        let width = if self.backend && prefix.ends_with("\r\n") {
            2
        } else if self.backend && prefix.ends_with(['\u{2028}', '\u{2029}']) {
            3
        } else {
            1
        };
        next_start - width
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    #[test]
    fn matches_source_map_boundaries() {
        let content = "a\nbeta\n";
        let index = LineIndex::new(content);

        assert_eq!(index.line_col_to_offset(content, 0, 1), Some(1));
        assert_eq!(index.line_col_to_offset(content, 1, 4), Some(6));
        assert_eq!(index.line_col_to_offset(content, 2, 0), Some(7));
        assert_eq!(index.line_col_to_offset(content, 1, 5), None);
        assert_eq!(index.offset_to_line_col(content, 7), Some((2, 0)));

        let content = "é\n";
        let index = LineIndex::new(content);
        assert_eq!(index.offset_to_line_col(content, 1), Some((0, 1)));
    }

    #[test]
    fn counts_astral_chars_as_two_utf16_units() {
        let content = "\u{1F600}x\n";
        let index = LineIndex::new(content);

        assert_eq!(index.offset_to_line_col(content, 5), Some((0, 3)));
        assert_eq!(index.line_col_to_offset(content, 0, 3), Some(5));
        assert_eq!(index.line_col_to_offset(content, 0, 1), None);
    }

    #[test]
    fn native_typescript_line_breaks_preserve_utf16_boundaries() {
        for newline in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
            let content = vize_carton::cstr!("\u{1f600}{newline}x{newline}");
            let index = LineIndex::for_backend(&content, super::LineBreaks::TypeScript);
            let second = 4 + newline.len() as u32;
            assert_eq!(index.line_col_to_offset(&content, 0, 2), Some(4));
            assert_eq!(index.line_col_to_offset(&content, 0, 3), None);
            assert_eq!(index.line_col_to_offset(&content, 1, 0), Some(second));
            assert_eq!(index.line_col_to_offset(&content, 1, 1), Some(second + 1));
            assert_eq!(index.offset_to_line_col(&content, second), Some((1, 0)));
            assert_eq!(
                index.offset_to_line_col(&content, content.len() as u32),
                Some((2, 0))
            );
        }
    }
}
