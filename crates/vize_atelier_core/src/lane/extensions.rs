//! Specialized transform entry points layered over the shared transform lane.

use vize_carton::{Bump, String};
use vize_croquis::Croquis;

use super::{JsxTransformCompat, transform_inner};
use crate::{CompilerError, RootNode, TransformOptions};

/// Transform the root AST node with an explicit scope ID for hoisted VNodes.
#[doc(hidden)]
pub fn transform_with_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        false,
        hoisted_scope_id,
        JsxTransformCompat::default(),
    )
}

/// Transform with template syntax quirks and an explicit hoisted VNode scope ID.
#[doc(hidden)]
pub fn transform_with_template_syntax_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        true,
        hoisted_scope_id,
        JsxTransformCompat::default(),
    )
}

/// Transform with Vue parser quirks and an explicit hoisted VNode scope ID.
#[doc(hidden)]
#[deprecated(note = "use transform_with_template_syntax_quirks_and_hoisted_scope_id instead")]
pub fn transform_with_vue_parser_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_with_template_syntax_quirks_and_hoisted_scope_id(
        allocator,
        root,
        options,
        analysis,
        hoisted_scope_id,
    )
}

/// Transform with Babel JSX's static plain-element v-model argument extension.
#[doc(hidden)]
pub fn transform_with_plain_element_model_argument<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        false,
        None,
        JsxTransformCompat {
            allow_static_v_model_arg_on_element: true,
            ..Default::default()
        },
    )
}

/// Transform JSX with Babel-specific element and v-model classification.
#[doc(hidden)]
pub fn transform_with_jsx_compatibility<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    allow_static_v_model_arg_on_element: bool,
    custom_element_spans: &[(u32, u32)],
) -> std::vec::Vec<CompilerError> {
    let mut jsx_compat = JsxTransformCompat {
        allow_static_v_model_arg_on_element,
        ..Default::default()
    };
    jsx_compat
        .custom_element_spans
        .extend(custom_element_spans.iter().copied());
    transform_inner(allocator, root, options, analysis, false, None, jsx_compat)
}
