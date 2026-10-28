//! Direct writer for checker virtual TypeScript.
//!
//! [`emit_ts!`](crate::emit_ts) splits a template at compile time. Static
//! text is one `push_str` run after a single `reserve`. `{expr}` writes a
//! string or an integer without `format!`. `{%expr}` is the `Display` path.
//! `{{` and `}}` emit literal braces.
//!
//! ```
//! use vize_l0::{String, emit_ts};
//! let mut ts = String::default();
//! let name = "Slots";
//! emit_ts!(&mut ts, "type __Vize{name} = {{}};\n");
//! assert_eq!(ts, "type __VizeSlots = {};\n");
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

/// A hole that can be written without the formatting machinery.
pub trait EmitHole<B: TsBuf> {
    fn emit_hole(&self, buffer: &mut B);
}

macro_rules! emit_str {
    ($($ty:ty),+ $(,)?) => {$(
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
        impl<B: TsBuf> EmitHole<B> for $ty {
            #[inline]
            fn emit_hole(&self, buffer: &mut B) {
                push_u64(buffer, *self as u64);
            }
        }
    )+};
}

emit_unsigned!(u8, u16, u32, u64, usize);

impl<B: TsBuf> EmitHole<B> for i32 {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_i64(buffer, i64::from(*self));
    }
}

impl<B: TsBuf> EmitHole<B> for i64 {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_i64(buffer, *self);
    }
}

impl<B: TsBuf> EmitHole<B> for isize {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        push_i64(buffer, *self as i64);
    }
}

impl<B: TsBuf> EmitHole<B> for bool {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        buffer.push_ts_str(if *self { "true" } else { "false" });
    }
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

#[cfg(test)]
mod tests {
    use crate::{String, emit_ts, ts};

    #[test]
    fn static_text_braces_and_dollars_stay_literal() {
        let mut ts = String::default();
        let name = "Slots";
        emit_ts!(&mut ts, "type __Vize{name} = {{}};\n");
        assert_eq!(ts, "type __VizeSlots = {};\n");
        emit_ts!(&mut ts, "type $$Hoge{name} = 1;\n");
        assert_eq!(ts, "type __VizeSlots = {};\ntype $$HogeSlots = 1;\n");
    }

    #[test]
    fn integers_skip_the_formatter() {
        let mut ts = String::default();
        let index = 0usize;
        let signed = -12i32;
        emit_ts!(
            &mut ts,
            "n{index}_{signed}_{true}_{i64::MIN}_{u64::MAX}"
        );
        assert_eq!(
            ts,
            "n0_-12_true_-9223372036854775808_18446744073709551615"
        );
    }

    #[test]
    fn ts_tokens_print_a_slot_alias() {
        let mut ts = String::default();
        let declaration = "<T>";
        let arguments = "<T>";
        ts! { &mut ts =>
            type __VizeSlots #declaration = Awaited<ReturnType<typeof __setup #arguments>>["__vize_typed_slots"];
        }
        assert_eq!(
            ts,
            "type __VizeSlots<T> = Awaited<ReturnType<typeof __setup<T>>>[\"__vize_typed_slots\"];\n"
        );
        ts! { &mut ts => export type { __VizeSlots as Slots }; }
        assert_eq!(
            &ts["type __VizeSlots<T> = Awaited<ReturnType<typeof __setup<T>>>[\"__vize_typed_slots\"];\n".len()..],
            "export type { __VizeSlots as Slots };\n"
        );
        let mut empty = String::default();
        ts! { &mut empty => type __VizeSlots = {}; }
        assert_eq!(empty, "type __VizeSlots = {};\n");
    }

    #[test]
    fn display_hole_is_the_slow_path() {
        let mut ts = String::default();
        let value = "kept";
        emit_ts!(&mut ts, "/* {%value} */");
        assert_eq!(ts, "/* kept */");
    }
}
