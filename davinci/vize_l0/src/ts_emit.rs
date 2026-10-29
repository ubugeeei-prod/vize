//! Direct writer for checker virtual TypeScript.
//!
//! [`ts!`](crate::ts) is the declarative writer: the token tree is the
//! TypeScript, so a missing `}` does not compile. [`emit_ts!`](crate::emit_ts)
//! remains the string form. Both reserve the exact static and hole length,
//! then write those bytes. No `format!` on the hot path. `{%expr}` is the
//! `Display` path, whose length is not known up front.
//!
//! ```
//! use vize_l0::{String, emit_ts, ts};
//! let mut ts = String::default();
//! let name = "Slots";
//! emit_ts!(&mut ts, "type ${name} = {{}};\n");
//! assert_eq!(ts, "type $Slots = {};\n");
//! ts! { &mut ts => export type { $Slots as Slots }; }
//! ```

use crate::CompactString;
use core::fmt;

/// Buffer that can accept reserved TypeScript bytes.
pub trait TsBuf {
    fn reserve_ts(&mut self, additional: usize);
    fn push_ts_str(&mut self, text: &str);
}

impl TsBuf for CompactString {
    #[inline]
    fn reserve_ts(&mut self, additional: usize) {
        self.reserve(additional);
    }

    #[inline]
    fn push_ts_str(&mut self, text: &str) {
        self.push_str(text);
    }
}

impl TsBuf for std::string::String {
    #[inline]
    fn reserve_ts(&mut self, additional: usize) {
        self.reserve(additional);
    }

    #[inline]
    fn push_ts_str(&mut self, text: &str) {
        self.push_str(text);
    }
}

/// Byte length of a hole, matching what [`EmitHole::emit_hole`] writes.
pub trait EmitLen {
    fn hole_len(&self) -> usize;
}

/// A hole that can be written without the formatting machinery.
pub trait EmitHole<B: TsBuf>: EmitLen {
    fn emit_hole(&self, buffer: &mut B);
}

macro_rules! emit_str {
    ($($ty:ty),+ $(,)?) => {$(
        impl EmitLen for $ty {
            #[inline]
            fn hole_len(&self) -> usize {
                let text: &str = self.as_ref();
                text.len()
            }
        }

        impl<B: TsBuf> EmitHole<B> for $ty {
            #[inline]
            fn emit_hole(&self, buffer: &mut B) {
                buffer.push_ts_str(self.as_ref());
            }
        }
    )+};
}

emit_str!(str, String, &str, &String, CompactString, &CompactString);

macro_rules! emit_unsigned {
    ($($ty:ty),+ $(,)?) => {$(
        impl EmitLen for $ty {
            #[inline]
            fn hole_len(&self) -> usize {
                u64_len(*self as u64)
            }
        }

        impl<B: TsBuf> EmitHole<B> for $ty {
            #[inline]
            fn emit_hole(&self, buffer: &mut B) {
                push_u64(buffer, *self as u64);
            }
        }
    )+};
}

emit_unsigned!(u8, u16, u32, u64, usize);

impl EmitLen for i32 {
    #[inline]
    fn hole_len(&self) -> usize {
        i64_len(i64::from(*self))
    }
}

impl<B: TsBuf> EmitHole<B> for i32 {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_i64(buffer, i64::from(*self));
    }
}

impl EmitLen for i64 {
    #[inline]
    fn hole_len(&self) -> usize {
        i64_len(*self)
    }
}

impl<B: TsBuf> EmitHole<B> for i64 {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_i64(buffer, *self);
    }
}

impl EmitLen for isize {
    #[inline]
    fn hole_len(&self) -> usize {
        i64_len(*self as i64)
    }
}

impl<B: TsBuf> EmitHole<B> for isize {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_i64(buffer, *self as i64);
    }
}

impl EmitLen for bool {
    #[inline]
    fn hole_len(&self) -> usize {
        if *self { 4 } else { 5 }
    }
}

impl<B: TsBuf> EmitHole<B> for bool {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        buffer.push_ts_str(if *self { "true" } else { "false" });
    }
}

/// A TypeScript string literal. The quotes and escapes are the hole bytes.
pub struct TsString<'a>(pub &'a str);

impl EmitLen for TsString<'_> {
    #[inline]
    fn hole_len(&self) -> usize {
        quoted_len(self.0)
    }
}

impl<B: TsBuf> EmitHole<B> for TsString<'_> {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_quoted(buffer, self.0);
    }
}

/// Byte length of `{expr}`. The length matches [`write_hole`].
#[inline(always)]
pub fn hole_len<T>(value: &T) -> usize
where
    T: EmitLen + ?Sized,
{
    value.hole_len()
}

/// Write `{expr}` into `buffer`.
#[inline(always)]
pub fn write_hole<B, T>(buffer: &mut B, value: &T)
where
    B: TsBuf,
    T: EmitHole<B> + ?Sized,
{
    value.emit_hole(buffer);
}

/// Write `{%expr}` through `Display`.
#[inline]
pub fn write_display<B, T>(buffer: &mut B, value: &T)
where
    B: TsBuf,
    T: fmt::Display + ?Sized,
{
    let _ = fmt::Write::write_fmt(&mut DisplayWriter(buffer), format_args!("{value}"));
}

struct DisplayWriter<'a, B>(&'a mut B);

impl<B: TsBuf> fmt::Write for DisplayWriter<'_, B> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.push_ts_str(text);
        Ok(())
    }
}

fn i64_len(value: i64) -> usize {
    if value < 0 {
        1 + u64_len((value as u64).wrapping_neg())
    } else {
        u64_len(value as u64)
    }
}

fn u64_len(mut value: u64) -> usize {
    if value == 0 {
        return 1;
    }
    let mut len = 0;
    while value > 0 {
        len += 1;
        value /= 10;
    }
    len
}

fn push_i64<B: TsBuf>(buffer: &mut B, value: i64) {
    if value < 0 {
        buffer.push_ts_str("-");
        push_u64(buffer, (value as u64).wrapping_neg());
    } else {
        push_u64(buffer, value as u64);
    }
}

fn push_u64<B: TsBuf>(buffer: &mut B, mut value: u64) {
    if value == 0 {
        buffer.push_ts_str("0");
        return;
    }
    let mut digits = [0u8; 20];
    let mut cursor = digits.len();
    for digit in digits.iter_mut().rev() {
        cursor -= 1;
        *digit = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    let Some(encoded) = digits.get(cursor..) else {
        buffer.push_ts_str("0");
        return;
    };
    match core::str::from_utf8(encoded) {
        Ok(text) => buffer.push_ts_str(text),
        Err(_) => buffer.push_ts_str("0"),
    }
}

fn quoted_len(value: &str) -> usize {
    let mut len = 2;
    for ch in value.chars() {
        len += escaped_unit_len(ch);
    }
    len
}

fn push_quoted<B: TsBuf>(buffer: &mut B, value: &str) {
    buffer.push_ts_str("\"");
    for ch in value.chars() {
        push_escaped(buffer, ch);
    }
    buffer.push_ts_str("\"");
}

fn escaped_unit_len(ch: char) -> usize {
    match ch {
        '"' | '\\' | '\n' | '\r' | '\t' | '\u{08}' | '\u{0c}' => 2,
        '\u{2028}' | '\u{2029}' => 6,
        ch if (ch as u32) < 0x20 => 6,
        ch => ch.len_utf8(),
    }
}

fn push_escaped<B: TsBuf>(buffer: &mut B, ch: char) {
    match ch {
        '"' => buffer.push_ts_str("\\\""),
        '\\' => buffer.push_ts_str("\\\\"),
        '\n' => buffer.push_ts_str("\\n"),
        '\r' => buffer.push_ts_str("\\r"),
        '\t' => buffer.push_ts_str("\\t"),
        '\u{08}' => buffer.push_ts_str("\\b"),
        '\u{0c}' => buffer.push_ts_str("\\f"),
        '\u{2028}' => buffer.push_ts_str("\\u2028"),
        '\u{2029}' => buffer.push_ts_str("\\u2029"),
        ch if (ch as u32) < 0x20 => {
            let byte = ch as u8;
            let mut encoded = [b'\\', b'u', b'0', b'0', 0, 0];
            encoded[4] = hex_digit(byte >> 4);
            encoded[5] = hex_digit(byte & 0x0f);
            if let Ok(text) = core::str::from_utf8(&encoded) {
                buffer.push_ts_str(text);
            }
        }
        ch => {
            let mut encoded = [0u8; 4];
            let text = ch.encode_utf8(&mut encoded);
            buffer.push_ts_str(text);
        }
    }
}

fn hex_digit(value: u8) -> u8 {
    char::from_digit(u32::from(value), 16)
        .map(|digit| digit as u8)
        .unwrap_or(b'0')
}

#[cfg(test)]
mod tests {
    use super::TsString;
    use crate::{String, emit_ts, ts};

    #[test]
    fn static_text_braces_and_dollars_stay_literal() {
        let mut ts = String::default();
        let name = "Slots";
        emit_ts!(&mut ts, "type ${name} = {{}};\n");
        assert_eq!(ts, "type $Slots = {};\n");
        emit_ts!(&mut ts, "type ${name} = 1;\n");
        assert_eq!(ts, "type $Slots = {};\ntype $Slots = 1;\n");
    }

    #[test]
    fn integers_skip_the_formatter() {
        let mut ts = String::default();
        let index = 0usize;
        let signed = -12i32;
        emit_ts!(&mut ts, "n{index}_{signed}_{true}_{i64::MIN}_{u64::MAX}");
        assert_eq!(ts, "n0_-12_true_-9223372036854775808_18446744073709551615");
    }

    #[test]
    fn ts_tokens_print_a_slot_alias() {
        let mut ts = String::default();
        let declaration = "<T>";
        let arguments = "<T>";
        ts! { &mut ts =>
            type $Slots #declaration = Awaited<ReturnType<typeof __setup #arguments>>["__vize_typed_slots"];
        }
        assert_eq!(
            ts,
            "type $Slots<T> = Awaited<ReturnType<typeof __setup<T>>>[\"__vize_typed_slots\"];\n"
        );
        ts! { &mut ts => export type { $Slots as Slots }; }
        assert_eq!(
            &ts["type $Slots<T> = Awaited<ReturnType<typeof __setup<T>>>[\"__vize_typed_slots\"];\n".len()..],
            "export type { $Slots as Slots };\n"
        );
        let mut empty = String::default();
        ts! { &mut empty => type $Slots = {}; }
        assert_eq!(empty, "type $Slots = {};\n");
        let mut block = String::default();
        ts! { &mut block =>
            export type Emits = {
                #("  ") #TsString("click"): any[];
            };
        }
        assert_eq!(block, "export type Emits = {\n  \"click\": any[];\n};\n");
        let mut indexed = String::default();
        ts! { &mut indexed =>
            type $Slots = Awaited<ReturnType<typeof __setup>> #("['__vize_typed_slots']");
        }
        assert_eq!(
            indexed,
            "type $Slots = Awaited<ReturnType<typeof __setup>>['__vize_typed_slots'];\n"
        );
    }

    #[test]
    fn display_hole_is_the_slow_path() {
        let mut ts = String::default();
        let value = "kept";
        emit_ts!(&mut ts, "/* {%value} */");
        assert_eq!(ts, "/* kept */");
    }

    #[test]
    fn hole_lengths_match_the_written_bytes() {
        let mut buffer = std::string::String::new();
        let samples: &[(&str, &str)] = &[
            ("plain", "\"plain\""),
            ("say \"hi\"", "\"say \\\"hi\\\"\""),
            ("a\\b", "\"a\\\\b\""),
            ("line\n", "\"line\\n\""),
        ];
        for (raw, expected) in samples {
            buffer.clear();
            let literal = TsString(raw);
            assert_eq!(literal.hole_len(), expected.len(), "{raw:?}");
            literal.emit_hole(&mut buffer);
            assert_eq!(buffer, *expected);
        }
        assert_eq!(i64_len(i64::MIN), "-9223372036854775808".len());
        assert_eq!(u64_len(u64::MAX), "18446744073709551615".len());
    }
}
