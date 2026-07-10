pub(super) struct LineIndex {
    starts: Vec<usize>,
    len: usize,
}

impl LineIndex {
    pub(super) fn new(content: &str) -> Self {
        let mut starts = vec![0];
        for (index, byte) in content.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }

        Self {
            starts,
            len: content.len(),
        }
    }

    /// Convert an LSP (line, character) — where character is in UTF-16 code
    /// units — back to a byte offset into `content`. (#965)
    pub(super) fn line_col_to_offset(&self, content: &str, line: u32, col: u32) -> Option<u32> {
        let line = usize::try_from(line).ok()?;
        let start = *self.starts.get(line)?;
        let end = self.line_end(line);
        let mut current_col = 0u32;
        let mut offset = start;

        if col == 0 {
            return u32::try_from(offset).ok();
        }

        for ch in content[start..end].chars() {
            offset += ch.len_utf8();
            current_col += ch.len_utf16() as u32;
            if current_col >= col {
                return u32::try_from(offset).ok();
            }
        }

        if current_col == col {
            u32::try_from(offset).ok()
        } else {
            None
        }
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
        let end = self.line_end(line);
        let mut col = 0u32;
        let mut cursor = start;
        for ch in content[start..end].chars() {
            if cursor >= offset {
                break;
            }
            col += ch.len_utf16() as u32;
            cursor += ch.len_utf8();
        }
        Some((u32::try_from(line).ok()?, col))
    }

    fn line_end(&self, line: usize) -> usize {
        self.starts
            .get(line + 1)
            .map(|next_start| next_start.saturating_sub(1))
            .unwrap_or(self.len)
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
    }
}
