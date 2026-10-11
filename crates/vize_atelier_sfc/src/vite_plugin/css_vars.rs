use vize_carton::String;

/// Replace CSS v-bind() calls before handing styles to Vite's CSS pipeline.
pub fn transform_css_vars_for_pipeline(css: &str, scope_id: &str) -> String {
    crate::css::transform_css_v_bind(css, Some(scope_id)).0
}
