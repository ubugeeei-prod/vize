use super::*;

#[test]
fn scopes_basic_selectors() {
    assert_eq!(
        scope_css_for_pipeline(".foo, .bar:hover { color: red; }", "data-v-x").as_str(),
        ".foo[data-v-x],.bar[data-v-x]:hover{color: red;}"
    );
}

#[test]
fn unwraps_deep_inside_scoped_selector() {
    assert_eq!(
        scope_css_for_pipeline(
            ".parent :deep(.child:nth-child(2)) { color: red; }",
            "data-v-x"
        )
        .as_str(),
        ".parent[data-v-x] .child:nth-child(2){color: red;}"
    );
}

#[test]
fn unwraps_legacy_v_deep_inside_scoped_selector() {
    assert_eq!(
        scope_css_for_pipeline(
            ".parent > ::v-deep(.child:nth-child(2)) { color: red; }",
            "data-v-x"
        )
        .as_str(),
        ".parent[data-v-x] > .child:nth-child(2){color: red;}"
    );
}

#[test]
fn unwraps_preprocessor_special_selectors() {
    let css = "[data-v-x] .parent > ::v-deep(.child), [data-v-x] :slotted(.slot), [data-v-x] .foo:global(.bar) {}";

    assert_eq!(
        unwrap_deep_selectors(css).as_str(),
        "[data-v-x] .parent > .child, [data-v-x] .slot, [data-v-x] .foo.bar {}"
    );
}

#[test]
fn recurses_media_rules() {
    assert_eq!(
        scope_css_for_pipeline(
            "@media (min-width: 1px) { .foo { color: red; } }",
            "data-v-x"
        )
        .as_str(),
        "@media (min-width: 1px) { .foo[data-v-x]{color: red;}}"
    );
}

#[test]
fn scopes_nested_css_like_vue_pipeline() {
    assert_eq!(
        scope_css_for_pipeline(
            "#pages-store { row-gap: 1.5rem; @media (--mobile) { row-gap: 1rem; } h1 { margin: 0; } :deep(.divider) { border: 0; } }",
            "data-v-x"
        )
        .as_str(),
        "#pages-store[data-v-x]{row-gap: 1.5rem;} @media (--mobile) {#pages-store[data-v-x]{row-gap: 1rem;}} #pages-store h1[data-v-x]{margin: 0;} #pages-store[data-v-x] .divider{border: 0;}"
    );
}

#[test]
fn scopes_nested_css_under_media_rules() {
    assert_eq!(
        scope_css_for_pipeline(
            "@media (--mobile) { .foo { color: red; :deep(.bar) { color: blue; } } }",
            "data-v-x"
        )
        .as_str(),
        "@media (--mobile) { .foo[data-v-x]{color: red;} .foo[data-v-x] .bar{color: blue;}}"
    );
}

#[test]
fn preserves_comments_before_media_rules() {
    assert_eq!(
        scope_css_for_pipeline(
            "/* desktop */\n@media (min-width: 1px) { .foo { color: red; } }",
            "data-v-x"
        )
        .as_str(),
        "/* desktop */\n@media (min-width: 1px) { .foo[data-v-x]{color: red;}}"
    );
}

#[test]
fn preserves_comments_before_selectors() {
    assert_eq!(
        scope_css_for_pipeline("/* card */\n.foo { color: red; }", "data-v-x").as_str(),
        "/* card */\n.foo[data-v-x]{color: red;}"
    );
}

#[test]
fn scopes_escaped_utility_selectors() {
    assert_eq!(
        scope_css_for_pipeline(
            ".hover\\:text-\\[\\#00DC82\\]:hover { color: red; }.text-\\[80px\\] { font-size: 80px; }",
            "data-v-x"
        )
        .as_str(),
        ".hover\\:text-\\[\\#00DC82\\][data-v-x]:hover{color: red;}.text-\\[80px\\][data-v-x]{font-size: 80px;}"
    );
}
