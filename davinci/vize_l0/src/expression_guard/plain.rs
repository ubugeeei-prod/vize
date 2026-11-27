//! A short-text certificate for the expression safety guard.
//!
//! Template expressions are mostly short member paths, calls, object
//! literals, comparisons and template strings. For those the complete scan
//! always admits the text, yet it runs two lexers with their stacks. This
//! single pass proves the same verdict for a bounded, simpler byte language
//! and leaves every other text to the complete scan.
//!
//! Why an admitted text passes every check of the complete scan:
//! - No `<`, `/`, `@` or `\` appears in code, so its type-angle, decorator and
//!   malformed-escape depths stay 0, no comment or regex starts, and the
//!   type-angle and arrow-default budgets, which only `<` feeds, stay 0.
//! - Strings hold no `\` or line break and template text holds no `\`, so
//!   both scanners split strings, template text and `${…}` exactly as this
//!   pass does. Their delimiter stack is therefore this frame stack: the text
//!   is balanced and every template ends, as checked here.
//! - Every frame costs an opener and a closer, so 63 bytes nest at most 31
//!   deep, the depth limit.
//! - A prefix run counts `! ~ + -` (at most 31 here) and prefix keywords (4 or
//!   more bytes each, so at most 15). Right-associative operators number at
//!   most 63 of the 256 allowed, and no numeric token reaches 4096 bytes.

use super::MAX_EXPRESSION_NESTING_DEPTH;

/// Longest text [`is_short_plain_expression`] admits. Each nesting level needs
/// at least an opener and a closer, so 63 bytes stay within depth 31; a run
/// of 32 prefix keywords or 257 right-associative operators cannot fit.
pub(super) const MAX_PLAIN_EXPRESSION_BYTES: usize = 2 * MAX_EXPRESSION_NESTING_DEPTH + 1;

const PAREN: u64 = 0;
const BRACKET: u64 = 1;
const BRACE: u64 = 2;
const INTERPOLATION: u64 = 3;

/// Open frames, two bits each, innermost in the low bits.
struct Frames {
    kinds: u64,
    depth: u32,
}

impl Frames {
    #[inline(always)]
    fn push(&mut self, kind: u64) {
        self.kinds = (self.kinds << 2) | kind;
        self.depth += 1;
    }

    #[inline(always)]
    fn pop(&mut self, kind: u64) -> bool {
        if self.depth == 0 || self.kinds & 3 != kind {
            return false;
        }
        self.kinds >>= 2;
        self.depth -= 1;
        true
    }

    #[inline(always)]
    fn top(&self) -> Option<u64> {
        (self.depth > 0).then_some(self.kinds & 3)
    }
}

/// Bytes that need no lexer state: identifier characters, digits, spaces and
/// the operators that neither open a frame nor start a prefix run.
const PLAIN: u8 = 1;
/// `! ~ + -`, which may start a prefix-operator run.
const PREFIX: u8 = 2;
/// Quotes, backticks and frame delimiters.
const STRUCTURAL: u8 = 3;

#[expect(
    clippy::indexing_slicing,
    reason = "the const loop index stays below the table length"
)]
static CLASSES: [u8; 256] = {
    let mut classes = [0u8; 256];
    let mut byte = 0usize;
    while byte < 256 {
        classes[byte] = match byte as u8 {
            b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'_'
            | b'$'
            | b' '
            | b'.'
            | b','
            | b':'
            | b'?'
            | b'='
            | b'*'
            | b'%'
            | b'^'
            | b'&'
            | b'|'
            | b'>' => PLAIN,
            b'!' | b'~' | b'+' | b'-' => PREFIX,
            b'"' | b'\'' | b'`' | b'(' | b')' | b'[' | b']' | b'{' | b'}' => STRUCTURAL,
            _ => 0,
        };
        byte += 1;
    }
    classes
};

#[inline(always)]
fn class(byte: u8) -> u8 {
    CLASSES.get(usize::from(byte)).copied().unwrap_or(0)
}

/// Whether every check of `expression_is_safe_to_parse` passes on sight.
///
/// The code bytes are ASCII identifier characters, digits, spaces, balanced
/// `()` / `[]` / `{}`, and the operators `. , : ? = ! ~ + - * % ^ & | >`.
/// Strings hold no `\` or line break; template text holds no `\`. With no
/// `<`, `/`, `@` or `\` in code there is no comment, regex, type-angle or
/// decorator depth and no arrow-default speculation, so the complete scan's
/// depth is the frame depth checked here. At most 31 prefix-operator bytes
/// bound every prefix run, and the length bound covers the rest.
#[inline]
pub(super) fn is_short_plain_expression(content: &str) -> bool {
    let bytes = content.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_PLAIN_EXPRESSION_BYTES {
        return false;
    }
    // Member paths and simple operators need no lexer state at all.
    let Some(start) = bytes.iter().position(|&byte| class(byte) != PLAIN) else {
        return true;
    };
    structured_tail_is_plain(bytes, start)
}

#[inline(never)]
fn structured_tail_is_plain(bytes: &[u8], mut i: usize) -> bool {
    let mut frames = Frames { kinds: 0, depth: 0 };
    let mut prefix_operators = 0usize;
    while let Some(&byte) = bytes.get(i) {
        match class(byte) {
            PLAIN => i += 1,
            PREFIX => {
                prefix_operators += 1;
                i += 1;
            }
            STRUCTURAL => match byte {
                b'"' | b'\'' => match string_end(bytes, i + 1, byte) {
                    Some(next) => i = next,
                    None => return false,
                },
                b'`' => match template_text_end(bytes, i + 1) {
                    Some((next, interpolation)) => {
                        if interpolation {
                            frames.push(INTERPOLATION);
                        }
                        i = next;
                    }
                    None => return false,
                },
                b'(' => {
                    frames.push(PAREN);
                    i += 1;
                }
                b'[' => {
                    frames.push(BRACKET);
                    i += 1;
                }
                b'{' => {
                    frames.push(BRACE);
                    i += 1;
                }
                b')' | b']' => {
                    if !frames.pop(if byte == b')' { PAREN } else { BRACKET }) {
                        return false;
                    }
                    i += 1;
                }
                _ if frames.top() == Some(INTERPOLATION) => {
                    frames.pop(INTERPOLATION);
                    match template_text_end(bytes, i + 1) {
                        Some((next, interpolation)) => {
                            if interpolation {
                                frames.push(INTERPOLATION);
                            }
                            i = next;
                        }
                        None => return false,
                    }
                }
                _ => {
                    if !frames.pop(BRACE) {
                        return false;
                    }
                    i += 1;
                }
            },
            _ => return false,
        }
    }
    frames.depth == 0 && prefix_operators <= MAX_EXPRESSION_NESTING_DEPTH
}

/// The byte after the closing `quote`, for a string with no escape or line
/// break; `None` otherwise.
#[inline(always)]
fn string_end(bytes: &[u8], mut i: usize, quote: u8) -> Option<usize> {
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'\\' | b'\n' | b'\r' => return None,
            _ if byte == quote => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// The byte after the closing backtick (`false`) or an opening `${` (`true`),
/// for template text with no escape; `None` when the text does not end.
#[inline(always)]
fn template_text_end(bytes: &[u8], mut i: usize) -> Option<(usize, bool)> {
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'\\' => return None,
            b'`' => return Some((i + 1, false)),
            b'$' if bytes.get(i + 1) == Some(&b'{') => return Some((i + 2, true)),
            _ => i += 1,
        }
    }
    None
}
