//! Two reviewed #7889 corrections to the immutable pre-retirement witness.
//! Authority: first_newline_parser_witness.json and the exact whole AST files.

use vize_l0::{config::VueVersion, hash::StableHasher128};
use vize_relief::options::{ParseMode, ParserOptions, WhitespaceStrategy};

const BEFORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/_fixtures/first_newline_parser_before.txt"
));
const AFTER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/_fixtures/first_newline_parser_after.txt"
));
const BEFORE_DIGEST: [u8; 16] = [
    177, 145, 132, 48, 217, 143, 93, 166, 252, 74, 92, 143, 86, 128, 114, 203,
];
const AFTER_DIGEST: [u8; 16] = [
    74, 137, 159, 212, 99, 214, 35, 180, 208, 148, 114, 27, 206, 253, 63, 2,
];
pub(super) const APPROVED_AGGREGATE: [u8; 16] = [
    85, 144, 88, 208, 172, 88, 157, 157, 0, 9, 189, 50, 214, 112, 133, 47,
];

pub(super) fn corrected_digest(
    index: usize,
    fields: &[&[u8]; 3],
    context: &str,
    options: &ParserOptions,
    historical: Option<&[u8]>,
) -> Option<([u8; 16], u8)> {
    let (required_context, bit) = match index {
        1103 => ("document=false; rcdata_textarea_title prefix@18", 1),
        3251 => ("document=true; rcdata_textarea_title prefix@18", 2),
        _ => return None,
    };
    let mut before = StableHasher128::new();
    for bytes in [fields[0], BEFORE.as_bytes(), fields[2]] {
        before.update(&(bytes.len() as u64).to_le_bytes());
        before.update(bytes);
    }
    (context == required_context
        && fields[0] == b"<textarea>{{ x }} "
        && fields[1] == AFTER.as_bytes()
        && historical == Some(BEFORE_DIGEST.as_slice())
        && before.digest() == BEFORE_DIGEST
        && default_fixture_options(options))
    .then_some((AFTER_DIGEST, bit))
}

fn default_fixture_options(options: &ParserOptions) -> bool {
    // Exhaustive fields keep the approval narrow if ParserOptions grows.
    let ParserOptions {
        mode,
        whitespace,
        delimiters,
        is_pre_tag,
        is_native_tag,
        is_custom_element,
        custom_renderer,
        is_void_tag,
        get_namespace,
        on_error,
        on_warn,
        comments,
        experimental_in_tag_comments,
        dialect,
    } = options;
    let defaults = ParserOptions::default();
    *mode == ParseMode::Base
        && *whitespace == WhitespaceStrategy::Condense
        && delimiters == &defaults.delimiters
        && core::ptr::fn_addr_eq(*is_pre_tag, defaults.is_pre_tag)
        && is_native_tag.is_none()
        && is_custom_element.is_none()
        && !*custom_renderer
        && core::ptr::fn_addr_eq(*is_void_tag, defaults.is_void_tag)
        && core::ptr::fn_addr_eq(*get_namespace, defaults.get_namespace)
        && on_error.is_none()
        && on_warn.is_none()
        && *comments
        && !*experimental_in_tag_comments
        && *dialect == VueVersion::V3
}
