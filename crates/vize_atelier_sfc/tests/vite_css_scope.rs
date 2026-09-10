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

#[test]
fn vite_pipeline_rewrites_legacy_deep_combinators() {
    let output = scope_css_for_pipeline(
        ".panel >>> .inner ::v-deep(.leaf) { padding: 0; }.panel ::v-deep(.other) /deep/ ::v-deep(.child) { margin: 0; }[data-label=\">>>\"] .plain { color: red; }.panel[data-token=\"::v-deep\"] { color: blue; }.panel/* >>> */ .after-comment { color: purple; }.literal\\>\\>\\> .plain { color: green; }",
        "data-v-x",
    );

    assert_eq!(
        output.as_str(),
        ".panel[data-v-x] .inner .leaf{padding: 0;}.panel[data-v-x] .other .child{margin: 0;}[data-label=\">>>\"] .plain[data-v-x]{color: red;}.panel[data-token=\"::v-deep\"][data-v-x]{color: blue;}.panel/* >>> */ .after-comment[data-v-x]{color: purple;}.literal\\>\\>\\> .plain[data-v-x]{color: green;}"
    );
}
