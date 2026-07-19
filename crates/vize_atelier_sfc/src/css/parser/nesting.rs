//! Pre-parse bracket nesting guard for the LightningCSS integration.
//!
//! Malformed style sources with deeply nested unclosed function tokens make
//! LightningCSS backtrack exponentially: every declaration whose typed value
//! parse fails is re-parsed as a raw token stream, so each extra nesting
//! level roughly doubles the work. The css_parse fuzz reproducer (#3105)
//! nested brackets 192 deep and needed seconds of CPU (well past the fuzz
//! timeout under instrumentation) only to report a parse error, while depth
//! 32 of the same shape stays in the microsecond range. Real stylesheets
//! never approach this depth, so parsing rejects such sources up front with
//! a regular parse error instead of timing out.

/// Maximum `(`/`[`/`{` nesting depth accepted before invoking LightningCSS.
pub(crate) const MAX_CSS_NESTING_DEPTH: usize = 32;

/// Error reported when [`css_nesting_exceeds_max_depth`] rejects a source.
pub(crate) const NESTING_DEPTH_ERROR: &str =
    "CSS parse error: bracket nesting exceeds the supported depth of 32";

/// Returns whether block nesting is deeper than [`MAX_CSS_NESTING_DEPTH`].
///
/// The scan follows CSS tokenizer rules so only brackets that open real
/// blocks are counted: strings (with the bad-string newline rule), comments,
/// escape sequences, and `url()`/bad-url tokens cannot contain blocks, and
/// brackets inside them are ignored. `url(` spelled with escapes (`\75rl(`)
/// is decoded so a quote inside a bad-url can never hide later brackets from
/// the count.
pub(crate) fn css_nesting_exceeds_max_depth(css: &str) -> bool {
    let bytes = css.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => i = skip_string(bytes, i + 1, bytes[i]),
            b'/' if bytes.get(i + 1) == Some(&b'*') => i = skip_comment(bytes, i + 2),
            b'(' | b'[' | b'{' => {
                depth += 1;
                if depth > MAX_CSS_NESTING_DEPTH {
                    return true;
                }
                i += 1;
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'\\' | 0x80.. => {
                let (next, is_url) = scan_ident(bytes, i);
                i = next;
                if is_url && bytes.get(i) == Some(&b'(') && !url_argument_is_string(bytes, i + 1) {
                    // A url-token (or bad-url) is atomic: it never opens a
                    // block and always ends at the first unescaped `)`.
                    i = skip_url(bytes, i + 1);
                }
            }
            _ => i += 1,
        }
    }
    false
}

/// Scans one ident-ish token run and reports whether it decodes to `url`.
///
/// Number and dimension prefixes are consumed by the same run (`0url` is a
/// dimension, not the `url` ident), which keeps the decision independent of
/// scan position without look-behind.
fn scan_ident(bytes: &[u8], start: usize) -> (usize, bool) {
    let mut i = start;
    let mut decoded = [0u8; 3];
    let mut len = 0usize;
    while i < bytes.len() {
        let ch = match bytes[i] {
            b @ (b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-') => {
                i += 1;
                b.to_ascii_lowercase()
            }
            0x80.. => {
                i += 1;
                0
            }
            b'\\' => {
                let (next, ch) = scan_escape(bytes, i + 1);
                if next == i + 1 {
                    // `\` before a newline or EOF is a lone delim token.
                    i = next;
                    break;
                }
                i = next;
                ch.to_ascii_lowercase()
            }
            _ => break,
        };
        if len < 3 {
            decoded[len] = ch;
        }
        len += 1;
    }
    (i, len == 3 && decoded == *b"url")
}

/// Consumes one escape sequence (the `\` already consumed) and returns the
/// decoded byte for ASCII code points, `0` otherwise.
fn scan_escape(bytes: &[u8], mut i: usize) -> (usize, u8) {
    match bytes.get(i) {
        None | Some(b'\n' | b'\r' | b'\x0C') => (i, 0),
        Some(b) if b.is_ascii_hexdigit() => {
            let mut value = 0u32;
            let mut digits = 0;
            while digits < 6
                && let Some(b) = bytes.get(i)
                && b.is_ascii_hexdigit()
            {
                value = value * 16 + (*b as char).to_digit(16).unwrap_or(0);
                i += 1;
                digits += 1;
            }
            // One optional whitespace terminates a hex escape.
            if matches!(bytes.get(i), Some(b' ' | b'\t' | b'\n' | b'\r' | b'\x0C')) {
                i += 1;
            }
            (i, u8::try_from(value).unwrap_or(0))
        }
        Some(&b) => (i + 1, b),
    }
}

/// Returns whether `url(` is followed by a quoted argument, which makes it a
/// regular function token instead of a url-token.
fn url_argument_is_string(bytes: &[u8], mut i: usize) -> bool {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\n' | b'\r' | b'\x0C')) {
        i += 1;
    }
    matches!(bytes.get(i), Some(b'"' | b'\''))
}

fn skip_string(bytes: &[u8], mut i: usize, quote: u8) -> usize {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = i.saturating_add(2),
            // Bad-string: an unescaped newline ends the token.
            b'\n' | b'\r' | b'\x0C' => return i,
            b if b == quote => return i + 1,
            _ => i += 1,
        }
    }
    i
}

fn skip_comment(bytes: &[u8], mut i: usize) -> usize {
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

/// Consumes url-token or bad-url content up to and including the closing `)`.
fn skip_url(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = i.saturating_add(2),
            b')' => return i + 1,
            _ => i += 1,
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::{MAX_CSS_NESTING_DEPTH, css_nesting_exceeds_max_depth};

    #[test]
    fn accepts_the_documented_boundary_and_rejects_one_past_it() {
        let allowed = "(".repeat(MAX_CSS_NESTING_DEPTH);
        let rejected = "(".repeat(MAX_CSS_NESTING_DEPTH + 1);
        assert!(!css_nesting_exceeds_max_depth(&allowed));
        assert!(css_nesting_exceeds_max_depth(&rejected));
    }

    #[test]
    fn sequential_blocks_do_not_accumulate_depth() {
        let css = ".a { --x: f(1); }".repeat(MAX_CSS_NESTING_DEPTH);
        assert!(!css_nesting_exceeds_max_depth(&css));
    }

    #[test]
    fn brackets_inside_strings_and_comments_do_not_count() {
        let brackets = "(".repeat(MAX_CSS_NESTING_DEPTH + 1);
        assert!(!css_nesting_exceeds_max_depth(&format!(
            ".a {{ content: \"{brackets}\"; }}"
        )));
        assert!(!css_nesting_exceeds_max_depth(&format!(
            "/* {brackets} */ .a {{ color: red; }}"
        )));
    }

    #[test]
    fn a_bad_string_newline_reenables_counting() {
        let css = format!("\" \n{}", "(".repeat(MAX_CSS_NESTING_DEPTH + 1));
        assert!(css_nesting_exceeds_max_depth(&css));
    }

    #[test]
    fn url_tokens_are_atomic() {
        let brackets = "(".repeat(MAX_CSS_NESTING_DEPTH + 1);
        // url-token and bad-url content never opens blocks.
        assert!(!css_nesting_exceeds_max_depth(&format!("url({brackets}")));
        assert!(!css_nesting_exceeds_max_depth(&format!(
            "\\75rl({brackets}"
        )));
        // A quote inside a bad-url must not hide brackets after the `)`.
        assert!(css_nesting_exceeds_max_depth(&format!(
            ".a {{ x: url(a\") ; y: {brackets}"
        )));
        // `url(` with a quoted argument is a plain function token.
        assert!(!css_nesting_exceeds_max_depth(
            ".a { background: url(\"a.png\"); }"
        ));
        // A dimension unit spelled `url` is not a url-token.
        assert!(css_nesting_exceeds_max_depth(&format!("0url({brackets}")));
    }

    #[test]
    fn escaped_brackets_do_not_count() {
        let css = "\\(".repeat(MAX_CSS_NESTING_DEPTH + 1);
        assert!(!css_nesting_exceeds_max_depth(&css));
    }
}
