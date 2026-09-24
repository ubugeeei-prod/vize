//! Transactional lookahead for JavaScript regex literals in SFC block scans.

pub(in crate::sfc::parse) fn skip_regex_literal(
    bytes: &[u8],
    mut pos: usize,
    len: usize,
    line: &mut usize,
    last_newline: &mut usize,
) -> Option<usize> {
    // Closing-tag search must ignore `</script>` inside JS regex literals without
    // allocating a lexer token stream. This byte scanner only activates in
    // syntactic positions where `/` can start a regex and tracks character
    // classes/escapes well enough to continue the zero-copy SFC block scan.
    debug_assert_eq!(bytes.get(pos), Some(&b'/'));
    pos += 1;
    let mut in_character_class = false;
    // On failure callers resume at the original slash. A speculative newline
    // must not move their location past a closing tag they have yet to visit.
    let mut scanned_line = *line;
    let mut scanned_last_newline = *last_newline;

    while pos < len
        && let Some(&c) = bytes.get(pos)
    {
        if c == b'\n' {
            // An unescaped newline terminates JavaScript regex literals. Stop
            // treating this as regex so normal malformed-block handling wins.
            return None;
        }

        if c == b'\\' {
            if pos + 1 < len && bytes.get(pos + 1) == Some(&b'\n') {
                scanned_line += 1;
                scanned_last_newline = pos + 1;
            }
            pos = (pos + 2).min(len);
            continue;
        }

        if in_character_class {
            if c == b']' {
                in_character_class = false;
            }
            pos += 1;
            continue;
        }

        match c {
            b'[' => {
                in_character_class = true;
                pos += 1;
            }
            b'/' => {
                pos += 1;
                while pos < len
                    && bytes
                        .get(pos)
                        .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    pos += 1;
                }
                *line = scanned_line;
                *last_newline = scanned_last_newline;
                return Some(pos);
            }
            _ => pos += 1,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::skip_regex_literal;

    #[test]
    fn failed_lookahead_does_not_commit_line_state() {
        for bytes in [b"/[a\\\n".as_slice(), b"/[a\\\n\n", b"/[a\\\nb\\\nc"] {
            let (mut line, mut last_newline) = (4, 17);
            assert_eq!(
                skip_regex_literal(bytes, 0, bytes.len(), &mut line, &mut last_newline),
                None
            );
            assert_eq!((line, last_newline), (4, 17));
        }
    }

    #[test]
    fn successful_lookahead_commits_line_state_once() {
        let bytes = b"/a\\\nb/g";
        let (mut line, mut last_newline) = (4, 0);
        assert_eq!(
            skip_regex_literal(bytes, 0, bytes.len(), &mut line, &mut last_newline),
            Some(bytes.len())
        );
        assert_eq!((line, last_newline), (5, 3));
    }

    #[test]
    fn regex_escapes_classes_and_flags_still_hide_closing_tags() {
        let bytes = br"/[</script>\/]/giu";
        let (mut line, mut last_newline) = (4, 0);
        assert_eq!(
            skip_regex_literal(bytes, 0, bytes.len(), &mut line, &mut last_newline),
            Some(bytes.len())
        );
        assert_eq!((line, last_newline), (4, 0));
    }
}
