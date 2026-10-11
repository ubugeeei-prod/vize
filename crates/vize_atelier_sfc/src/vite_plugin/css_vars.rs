use vize_carton::String;

/// Replace CSS v-bind() calls before handing styles to Vite's CSS pipeline.
pub fn transform_css_vars_for_pipeline(css: &str, scope_id: &str) -> String {
    transform_css_vars_for_pipeline_with_context(css, scope_id, None)
}

/// Use the same production filename authority as the SFC's runtime variables.
pub fn transform_css_vars_for_pipeline_with_context(
    css: &str,
    scope_id: &str,
    production_filename: Option<&str>,
) -> String {
    let bump = vize_carton::pool::acquire();
    let (code, _) = crate::css::transform::extract_and_transform_v_bind_with_context(
        &bump,
        css,
        Some(scope_id),
        production_filename,
    );
    String::from(code)
}
