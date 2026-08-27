//! Specialized transform entry points layered over the shared transform lane.

use vize_croquis::Croquis;
use vize_s0::{Allocator, String};

use super::{JsxTransformCompat, TransformLaneOptions, transform_inner};
use crate::{CompilerError, RootNode, TransformOptions, options::CustomElementMatcher};

/// Transform the root AST node with an explicit scope ID for hoisted VNodes.
#[doc(hidden)]
pub fn transform_with_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
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
        TransformLaneOptions {
            hoisted_scope_id,
            ..Default::default()
        },
        None,
    )
}

/// Transform with template syntax quirks and an explicit hoisted VNode scope ID.
#[doc(hidden)]
pub fn transform_with_template_syntax_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
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
        TransformLaneOptions {
            template_syntax_quirks: true,
            hoisted_scope_id,
            ..Default::default()
        },
        None,
    )
}

/// Transform with declarative custom-element patterns and optional hoisted scope ID.
#[doc(hidden)]
pub fn transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    custom_elements: CustomElementMatcher,
    template_syntax_quirks: bool,
    hoisted_scope_id: Option<String>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions {
            template_syntax_quirks,
            hoisted_scope_id,
            custom_elements,
            ..Default::default()
        },
        None,
    )
}

/// Transform with Vue parser quirks and an explicit hoisted VNode scope ID.
#[doc(hidden)]
#[deprecated(note = "use transform_with_template_syntax_quirks_and_hoisted_scope_id instead")]
pub fn transform_with_vue_parser_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
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
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions {
            jsx_compat: JsxTransformCompat {
                allow_static_v_model_arg_on_element: true,
                ..Default::default()
            },
            ..Default::default()
        },
        None,
    )
}

/// Transform against an explicit loc-span source basis.
///
/// JSX roots keep the root element's slice in `RootNode::source` (the source
/// maps embed it), while node spans index into the whole module source; this
/// entry lets those callers supply the module text the spans resolve against.
#[doc(hidden)]
pub fn transform_with_source_text<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    source_text: &'a str,
) -> std::vec::Vec<CompilerError> {
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions::default(),
        Some(source_text),
    )
}

/// Transform JSX with Babel-specific element and v-model classification.
#[doc(hidden)]
pub fn transform_with_jsx_compatibility<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    allow_static_v_model_arg_on_element: bool,
    custom_element_spans: &[(u32, u32)],
    source_text: Option<&'a str>,
) -> std::vec::Vec<CompilerError> {
    let mut jsx_compat = JsxTransformCompat {
        allow_static_v_model_arg_on_element,
        ..Default::default()
    };
    jsx_compat
        .custom_element_spans
        .extend(custom_element_spans.iter().copied());
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions {
            jsx_compat,
            ..Default::default()
        },
        source_text,
    )
}
