mod split;
mod type_text;
mod unsafe_flow;
#[cfg(test)]
mod unsafe_flow_tests;

pub use type_text::{
    TypeTextKind, classify_type_text, is_any_like_type_texts, is_array_like_type_texts,
    is_bigint_like_type_texts, is_error_like_type_texts, is_number_like_type_texts,
    is_promise_like_type_texts, is_string_array_like_type_texts, is_string_like_type_texts,
    is_unknown_like_type_texts, split_top_level_type_text, split_type_text,
};
pub use unsafe_flow::{has_unsafe_any_flow, is_unsafe_assignment, is_unsafe_return};
