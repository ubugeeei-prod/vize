//! Code/message-level suppression rules shared by the LSP and CLI diagnostic
//! paths. These decide whether a TypeScript diagnostic is reportable at all,
//! independent of where it maps back to in the original source.

use std::path::Path;

use super::OriginalPosition;

pub(crate) fn should_skip_diagnostic(code: Option<u32>, message: &str) -> bool {
    match code {
        // TS2666: virtual-TS generation injects helper bindings that can trip
        // this code outside the user's source — suppress to match vue-tsc.
        Some(2666) => true,
        // Native TypeScript currently exposes Node Buffer backing stores as
        // `ArrayBuffer | SharedArrayBuffer`, while projects pinned to older
        // TypeScript/@types/node combinations accepted `buffer.slice(...)` as
        // `ArrayBuffer`. Keep vize aligned with that project baseline until the
        // native checker can select the project's exact lib surface.
        Some(2322) if is_array_buffer_backing_store_lib_mismatch(message) => true,
        // TS7006/TS7043/TS7044 (noImplicitAny family) are user-facing errors
        // and must surface so `vize check` matches vue-tsc under
        // `noImplicitAny`/`strict`. They were previously suppressed (#966).
        _ => false,
    }
}

fn is_array_buffer_backing_store_lib_mismatch(message: &str) -> bool {
    message
        .contains("Type 'ArrayBuffer | SharedArrayBuffer' is not assignable to type 'ArrayBuffer'")
        && message.contains("SharedArrayBuffer")
}

pub(crate) fn should_skip_original_diagnostic(
    code: Option<u32>,
    original: &OriginalPosition,
) -> bool {
    code == Some(6133) && original.block_type.is_none() && is_vue_source(&original.path)
}

fn is_vue_source(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}
