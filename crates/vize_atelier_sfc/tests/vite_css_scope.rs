use vize_atelier_sfc::vite_plugin::scope_css_for_pipeline;

#[test]
fn vite_pipeline_scopes_parent_before_trailing_universal_selectors() {
    assert_eq!(
        scope_css_for_pipeline(".dialog__action-buttons > * { flex: 1; }", "data-v-x").as_str(),
        ".dialog__action-buttons[data-v-x] > *{flex: 1;}"
    );
    assert_eq!(
        scope_css_for_pipeline(".dialog__action-buttons>*:hover { flex: 1; }", "data-v-x").as_str(),
        ".dialog__action-buttons[data-v-x]>*:hover{flex: 1;}"
    );
}
