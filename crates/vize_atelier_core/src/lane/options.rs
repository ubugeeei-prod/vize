use vize_s0::{FxHashSet, String};

use crate::options::CustomElementMatcher;

#[derive(Default)]
pub(crate) struct JsxTransformCompat {
    pub allow_static_v_model_arg_on_element: bool,
    pub custom_element_spans: FxHashSet<(u32, u32)>,
}

#[derive(Default)]
pub(crate) struct TransformLaneOptions {
    pub template_syntax_quirks: bool,
    pub hoisted_scope_id: Option<String>,
    pub jsx_compat: JsxTransformCompat,
    pub custom_elements: CustomElementMatcher,
}
