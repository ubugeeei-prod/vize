//! `needs_typescript_stripping`, ported verbatim.
//!
//! The shipped stripper is gated on this scan, so its false positives and
//! false negatives are part of the shipped bytes: text it rejects never
//! reaches the oxc round-trip, and text it accepts takes the round-trip
//! even when the parse turns out to be plain JavaScript.

/// Whether the expression carries syntax the TS lane has to lower.
pub(in crate::emit) fn needs_typescript_stripping(content: &str) -> bool {
    // " as " is a type assertion; `: ` is skipped because object literals
    // spell it too.
    if content.contains(" as ") {
        return true;
    }

    if contains_unquoted_word(content, "satisfies") {
        return true;
    }

    if contains_generic_call(content) {
        return true;
    }

    // Arrow parameter annotations: `(x: Type) => …`, `(a: A, b: B) => …`.
    if content.contains("=>") {
        let bytes = content.as_bytes();
        let mut in_paren = false;
        let mut after_ident = false;
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'(' => {
                    in_paren = true;
                    after_ident = false;
                }
                b')' => {
                    in_paren = false;
                    after_ident = false;
                }
                b':' if in_paren && after_ident => {
                    // `::` is a namespace separator, not an annotation.
                    if i + 1 < bytes.len() && bytes[i + 1] != b':' {
                        return true;
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' | b'0'..=b'9' => {
                    after_ident = true;
                }
                // Whitespace does not reset the identifier state.
                b' ' | b'\t' => {}
                b',' => after_ident = false,
                _ => after_ident = false,
            }
        }
    }

    // A non-null assertion follows an expression (`foo!`, `foo()!`,
    // `foo[0]!`); logical NOT precedes one.
    let bytes = content.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'!' && i > 0 {
            let prev = bytes[i - 1];
            let is_non_null_assertion = prev.is_ascii_alphanumeric()
                || prev == b'_'
                || prev == b'$'
                || prev == b')'
                || prev == b']';
            if is_non_null_assertion {
                return true;
            }
        }
    }

    false
}

fn contains_unquoted_word(content: &str, word: &str) -> bool {
    let mut quote = None;
    let mut prev = '\0';

    for (index, ch) in content.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            _ if content[index..].starts_with(word) => {
                let before = content[..index].chars().next_back();
                let after = content[index + word.len()..].chars().next();
                let before_boundary = before.is_none_or(|ch| !is_ident_char(ch));
                let after_boundary = after.is_none_or(|ch| !is_ident_char(ch));
                if before_boundary && after_boundary {
                    return true;
                }
            }
            _ => {}
        }
        prev = ch;
    }

    false
}

fn contains_generic_call(content: &str) -> bool {
    let mut quote = None;
    let mut prev = '\0';

    for (index, ch) in content.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '<' if previous_non_whitespace(content, index).is_some_and(is_ident_char) => {
                if let Some(close) = find_matching_angle(content, index) {
                    let after = content[close + 1..].trim_start();
                    if after.starts_with('(') {
                        return true;
                    }
                }
            }
            _ => {}
        }
        prev = ch;
    }

    false
}

fn previous_non_whitespace(content: &str, index: usize) -> Option<char> {
    content[..index]
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
}

fn find_matching_angle(content: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';

    for (relative, ch) in content[open_index..].char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_index + relative);
                }
            }
            _ => {}
        }
        prev = ch;
    }

    None
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

#[cfg(test)]
mod tests {
    use super::needs_typescript_stripping;

    #[test]
    fn detects_the_shipped_typescript_shapes() {
        assert!(needs_typescript_stripping("foo as string"));
        assert!(needs_typescript_stripping("payload satisfies Payload"));
        assert!(needs_typescript_stripping("useStore<RootState>()"));
        assert!(needs_typescript_stripping("(x: number) => x + 1"));
        assert!(needs_typescript_stripping("foo!.bar"));
        assert!(!needs_typescript_stripping("foo.bar"));
        assert!(!needs_typescript_stripping("(x) => x + 1"));
        assert!(!needs_typescript_stripping("useStore()"));
        assert!(!needs_typescript_stripping("!foo"));
        assert!(!needs_typescript_stripping(
            "'payload satisfies Payload'.includes(query)"
        ));
    }
}
