//! Patch-flag comment emission (`8 /* PROPS */`, `16 /* FULL_PROPS */`, …).

use vize_carton::ToCompactString;

use super::EmitCx;

const PATCH_NAMES: [(i32, &str); 6] = [
    (1, "TEXT"),
    (2, "CLASS"),
    (4, "STYLE"),
    (8, "PROPS"),
    (16, "FULL_PROPS"),
    (32, "NEED_HYDRATION"),
];

pub(super) fn emit_patch_flag(cx: &mut EmitCx<'_>, flag: i32) {
    cx.buf.push(", ");
    cx.buf.push(flag.to_compact_string().as_str());
    cx.buf.push(" /* ");
    let mut first = true;
    for (bit, name) in PATCH_NAMES {
        if flag & bit == 0 {
            continue;
        }
        if !first {
            cx.buf.push(", ");
        }
        first = false;
        cx.buf.push(name);
    }
    if first {
        cx.buf.push("UNKNOWN");
    }
    cx.buf.push(" */");
}
