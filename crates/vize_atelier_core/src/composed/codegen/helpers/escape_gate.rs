// DO NOT EDIT. Generated from crates/vize_atelier_core/src/codegen/helpers/escape_gate.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! The fast-path gate of `escape_js_string`.
#[cfg(creusot)]
use creusot_std::prelude::*;

/// Returns true if a byte may belong to a character that `escape_js_string`
/// would rewrite (an escape-requiring char) or that `decode_html_entities`
/// might act on (`&`). Used as a cheap fast-path gate: when no byte of the
/// input matches, the string passes through both stages unchanged.
///
/// The control characters the slow path escapes are exactly the Unicode `Cc`
/// category — C0 (U+0000..=U+001F), DEL (U+007F), and C1 (U+0080..=U+009F) —
/// because the catch-all arm uses `char::is_control()`. In UTF-8 those are the
/// bytes `0x00..=0x1F`, `0x7F`, and the two-byte sequences `0xC2 0x80..=0xC2
/// 0x9F`. We flag every `0xC2` lead byte (a conservative superset of the C1
/// range) so a C1 control can never slip through the fast path; this is the
/// subtle case a naive `b < 0x20` check would miss. All other non-ASCII bytes
/// (lead `>= 0xC3` and continuation bytes) encode characters `>= U+00C0`, none
/// of which are control characters, so they are safe to pass through.
#[cfg_attr(creusot, ensures(forall<c: Int> rewritten(c) && utf8_lead(c) == b@ ==> result))]
#[inline]
pub(super) fn byte_may_need_js_escaping(b: u8) -> bool {
    b < 0x20 || b == b'"' || b == b'&' || b == b'\\' || b == 0x7F || b == 0xC2
}

/// A Unicode scalar value: a code point that is not a surrogate.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn scalar(c: Int) -> bool {
    pearlite! { 0 <= c && c <= 0x10FFFF && !(0xD800 <= c && c <= 0xDFFF) }
}

/// A scalar that one of the two slow stages rewrites: `decode_html_entities`
/// acts on `&`, and `escape_js_string` escapes `\`, `"` and every character
/// of the Unicode general category `Cc` (U+0000..=U+001F, U+007F..=U+009F).
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn rewritten(c: Int) -> bool {
    pearlite! {
        scalar(c) && (c == 0x26 || c == 0x5C || c == 0x22 || c <= 0x1F || (0x7F <= c && c <= 0x9F))
    }
}

/// The first byte of the UTF-8 encoding of scalar `c` (Unicode, Table 3-6).
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn utf8_lead(c: Int) -> Int {
    pearlite! {
        if c < 0x80 {
            c
        } else if c < 0x800 {
            0xC0 + c / 0x40
        } else if c < 0x1_0000 {
            0xE0 + c / 0x1000
        } else {
            0xF0 + c / 0x4_0000
        }
    }
}
