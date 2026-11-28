// DO NOT EDIT. Generated from crates/vize_atelier_core/src/lane/options.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
use vize_l0::{FxHashSet, Span, String};

use crate::composed::options::CustomElementMatcher;

#[derive(Default)]
pub(crate) struct JsxTransformCompat {
    pub allow_static_v_model_arg_on_element: bool,
    pub custom_element_spans: FxHashSet<(u32, u32)>,
}

#[derive(Default)]
pub(crate) struct TransformLaneOptions<'a> {
    pub template_syntax_quirks: bool,
    pub hoisted_scope_id: Option<String>,
    pub jsx_compat: JsxTransformCompat,
    pub custom_elements: CustomElementMatcher,
    pub frozen_elements: &'a [Span],
}
