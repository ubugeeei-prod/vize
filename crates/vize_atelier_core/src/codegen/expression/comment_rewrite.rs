//! Line-comment to block-comment conversion for single-line render output.
//!
//! Template parsers may normalize newlines in attribute values to spaces, so a
//! `//` comment in an authored expression would eat the rest of the generated
//! line. The converter rewrites `// …` to `/* … */` — but it must lex like
//! JavaScript to know which `/` starts a comment: the old scanner knew only
//! strings, so the `//` inside a regex literal's `\/\/` (or after a regex it
//! mis-terminated at a `/` inside a character class, #3943) was rewritten and
//! the emitted render expression was corrupted. Regex positions are decided
//! with the same fuzz-hardened primitives the expression nesting guard uses.

use crate::steps::expression::nesting::scan::{
    keyword_allows_regex_after, skip_identifier, skip_line_comment, skip_number, skip_quoted,
    skip_regex,
};
use vize_s0::String;

/// Convert `// …` line comments to `/* … */` block comments, copying strings,
/// template literals, block comments, and regex literals through verbatim.
///
/// Template literals are copied to their closing backtick like the string arms
/// (escape-aware, no `${}` recursion): a line comment inside an interpolation
/// survives unrewritten, matching the previous behavior. Copied segments are
/// sliced, not rebuilt byte-by-byte, so non-ASCII content survives intact.
pub(crate) fn convert_line_comments_to_block(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut result = String::with_capacity(content.len());
    // Whether a `/` at the cursor would start a regex literal (operand
    // position), mirroring the nesting scanner's tracking.
    let mut can_start_regex = true;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'\'' | b'"' | b'`' => {
                // `skip_quoted` is a cursor, not a slice bound: an unterminated
                // literal whose last byte is `\` overshoots to len+1 (the
                // nesting guard only ever compares it). Clamp before slicing.
                let end = skip_quoted(bytes, i + 1, b).min(bytes.len());
                result.push_str(&content[i..end]);
                i = end;
                can_start_regex = false;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let comment_start = i + 2;
                // A line comment ends at any ECMAScript line terminator — LF,
                // CR, LS, or PS — not LF alone: stopping only at LF pulled the
                // code after a bare CR (or LS/PS) into the generated block
                // comment. The terminator itself is left for a later iteration,
                // which copies it through unchanged.
                let comment_end = skip_line_comment(bytes, comment_start);
                let comment_text = content[comment_start..comment_end].trim_end();
                result.push_str("/* ");
                // A line comment may legally contain `*/`, which would close the
                // generated block comment early — exposing the rest as live code
                // or producing invalid JavaScript. Neutralize it; comment text
                // carries no semantics.
                result.push_str(&comment_text.replace("*/", "* /"));
                result.push_str(" */");
                i = comment_end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let mut end = i + 2;
                let end = loop {
                    if end + 1 >= bytes.len() {
                        break bytes.len();
                    }
                    if bytes[end] == b'*' && bytes[end + 1] == b'/' {
                        break end + 2;
                    }
                    end += 1;
                };
                result.push_str(&content[i..end]);
                i = end;
            }
            b'/' if can_start_regex => {
                // An unterminated regex is not one (the lexer recovers); fall
                // through and scan the bytes as ordinary source, same as the
                // nesting guard.
                if let Some(end) = skip_regex(bytes, i + 1) {
                    result.push_str(&content[i..end]);
                    i = end;
                    can_start_regex = false;
                } else {
                    result.push('/');
                    i += 1;
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let end = skip_identifier(bytes, i + 1);
                result.push_str(&content[i..end]);
                can_start_regex = keyword_allows_regex_after(&bytes[i..end]);
                i = end;
            }
            b'0'..=b'9' => {
                let end = skip_number(bytes, i + 1);
                result.push_str(&content[i..end]);
                i = end;
                can_start_regex = false;
            }
            b')' | b']' | b'}' => {
                result.push(b as char);
                i += 1;
                can_start_regex = false;
            }
            // `++`/`--`: the fallback arm would see two operator bytes and claim
            // operand position, so the division in `a++ / b // note` was lexed
            // as a regex that swallowed the real comment's first `/`. A `/`
            // after an increment/decrement is always division, as the nesting
            // scanner also tracks.
            b'+' | b'-' if bytes.get(i + 1) == Some(&b) => {
                result.push_str(&content[i..i + 2]);
                i += 2;
                can_start_regex = false;
            }
            _ => {
                // One whole character, not one byte: non-ASCII must not be
                // split into mojibake.
                let ch = content[i..].chars().next().unwrap_or('\u{FFFD}');
                result.push(ch);
                i += ch.len_utf8();
                if !ch.is_ascii_whitespace() {
                    // Operators, `(`/`[`/`{`, commas: an operand may follow.
                    can_start_regex = true;
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::convert_line_comments_to_block;

    #[test]
    fn regex_literals_survive_including_character_classes() {
        // The #3943 reproducer: `/` inside `[^/]` must not terminate the
        // regex, and the `\/\/` inside it must not read as a line comment.
        let source = "url ? url.replace(/https?:\\/\\/[^/]+\\//, '/') : 'No URL'";
        assert_eq!(convert_line_comments_to_block(source), source);
        // Regex directly after `(`, `=`, `return`, and `,`.
        for source in [
            "match(/a[/]b/)",
            "const x = /[/]+/g",
            "return /a\\/b/.test(s)",
            "f(1, /[^/]/)",
        ] {
            assert_eq!(convert_line_comments_to_block(source), source);
        }
    }

    #[test]
    fn real_line_comments_still_convert() {
        assert_eq!(
            convert_line_comments_to_block("count // note"),
            "count /*  note */"
        );
        // Division stays division; the trailing comment still converts.
        assert_eq!(
            convert_line_comments_to_block("a / b // half"),
            "a / b /*  half */"
        );
        // After a closing paren a slash is division, not a regex.
        assert_eq!(
            convert_line_comments_to_block("f(x) / 2 // note"),
            "f(x) / 2 /*  note */"
        );
        // After `++`/`--` a slash is division too; the comment still converts.
        assert_eq!(
            convert_line_comments_to_block("a++ / b // note"),
            "a++ / b /*  note */"
        );
        assert_eq!(
            convert_line_comments_to_block("a-- / b // note"),
            "a-- / b /*  note */"
        );
    }

    #[test]
    fn every_line_terminator_ends_the_comment() {
        // CR, LS (U+2028), and PS (U+2029) end a line comment just like LF, and
        // the terminator is copied through unchanged.
        assert_eq!(
            convert_line_comments_to_block("count // note\r next"),
            "count /*  note */\r next"
        );
        assert_eq!(
            convert_line_comments_to_block("count // note\u{2028}next"),
            "count /*  note */\u{2028}next"
        );
        assert_eq!(
            convert_line_comments_to_block("count // note\u{2029}next"),
            "count /*  note */\u{2029}next"
        );
        assert_eq!(
            convert_line_comments_to_block("count // note\nnext"),
            "count /*  note */\nnext"
        );
    }

    #[test]
    fn block_comment_terminator_inside_a_line_comment_is_neutralized() {
        // `*/` in the comment text must not close the generated block comment
        // and revive the code after it.
        assert_eq!(
            convert_line_comments_to_block("value // */ + sideEffect() /*"),
            "value /*  * / + sideEffect() /* */"
        );
    }

    #[test]
    fn an_unterminated_string_ending_in_a_backslash_does_not_slice_past_the_end() {
        // The template_compile fuzz corpus caught the ported scanner slicing
        // with skip_quoted's cursor, which overshoots to len+1 when the last
        // byte is an escape lead (crash-43ea1164, v0.344.0 release gate).
        for source in ["'\\", "\"unterminated \\", "'//p+\\\\\\\\:\\"] {
            let converted = convert_line_comments_to_block(source);
            assert_eq!(converted, source);
        }
    }

    #[test]
    fn strings_templates_and_block_comments_pass_through() {
        for source in [
            "'https://a' + \"//b\"",
            "`https://tpl`",
            "x /* keep // this */ + y",
        ] {
            assert_eq!(convert_line_comments_to_block(source), source);
        }
        // Non-ASCII outside strings survives (the old byte-wise copy
        // produced mojibake).
        assert_eq!(
            convert_line_comments_to_block("名前 ?? 'https://例.jp' // 補足"),
            "名前 ?? 'https://例.jp' /*  補足 */"
        );
    }
}
