use vize_carton::String;

/// Replace CSS v-bind() calls before handing styles to Vite's CSS pipeline.
pub fn transform_css_vars_for_pipeline(css: &str, scope_id: &str) -> String {
    transform_css_vars_for_pipeline_with_context(css, scope_id, None)
}

/// Use the stable component scope as the production hash authority.
///
/// The optional filename is retained as the public production-mode marker, but
/// checkout-local path bytes must never affect emitted CSS variable names.
pub fn transform_css_vars_for_pipeline_with_context(
    css: &str,
    scope_id: &str,
    production_filename: Option<&str>,
) -> String {
    let bump = vize_carton::pool::acquire();
    let stable_id = scope_id.strip_prefix("data-v-").unwrap_or(scope_id);
    let (code, _) = crate::css::transform::extract_and_transform_v_bind_with_context(
        &bump,
        css,
        Some(scope_id),
        production_filename.map(|_| stable_id),
    );
    String::from(code)
}

/// Transform a retained original style, publishing the edits used for source maps.
#[cfg(feature = "original-style-bindings")]
pub fn transform_original_style_bindings(
    style: &vize_l1_to_l2::lower::OriginalStyleBindings<'_>,
    scope_id: &str,
    production_filename: Option<&str>,
    record: impl FnMut(vize_l4::targets::css::StyleBindingEdit),
) -> String {
    let stable_id = scope_id.strip_prefix("data-v-").unwrap_or(scope_id);
    vize_l4::targets::css::emit_style_bindings(
        style,
        |expression| {
            if production_filename.is_some() {
                crate::css::prod_scoped_v_bind_name(stable_id, expression)
            } else {
                crate::css::scoped_v_bind_name(scope_id, expression)
            }
        },
        record,
    )
}
