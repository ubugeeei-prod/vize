//! Equality helpers for parser options containing function pointers.

use crate::SfcParseOptions;

pub(super) fn parse_options_eq(left: &SfcParseOptions, right: &SfcParseOptions) -> bool {
    left.filename == right.filename
        && left.source_map == right.source_map
        && left.pad == right.pad
        && left.ignore_empty == right.ignore_empty
        && match (&left.template_parse_options, &right.template_parse_options) {
            (None, None) => true,
            (Some(left), Some(right)) => parser_options_eq(left, right),
            _ => false,
        }
}

#[allow(clippy::too_many_lines)]
fn parser_options_eq(
    left: &vize_relief::ParserOptions,
    right: &vize_relief::ParserOptions,
) -> bool {
    left.mode == right.mode
        && left.whitespace == right.whitespace
        && left.delimiters == right.delimiters
        && std::ptr::fn_addr_eq(left.is_pre_tag, right.is_pre_tag)
        && optional_fn_eq(left.is_native_tag, right.is_native_tag)
        && optional_fn_eq(left.is_custom_element, right.is_custom_element)
        && left.custom_renderer == right.custom_renderer
        && std::ptr::fn_addr_eq(left.is_void_tag, right.is_void_tag)
        && std::ptr::fn_addr_eq(left.get_namespace, right.get_namespace)
        && optional_handler_eq(left.on_error, right.on_error)
        && optional_handler_eq(left.on_warn, right.on_warn)
        && left.comments == right.comments
        && left.experimental_in_tag_comments == right.experimental_in_tag_comments
        && left.dialect == right.dialect
}

fn optional_fn_eq(left: Option<fn(&str) -> bool>, right: Option<fn(&str) -> bool>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => std::ptr::fn_addr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn optional_handler_eq(
    left: Option<fn(vize_relief::CompilerError)>,
    right: Option<fn(vize_relief::CompilerError)>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => std::ptr::fn_addr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}
