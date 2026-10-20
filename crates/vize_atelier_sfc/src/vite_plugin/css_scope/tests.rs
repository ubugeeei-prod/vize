use super::*;

#[test]
fn scopes_basic_selectors() {
    assert_eq!(
        scope_css_for_pipeline(".foo, .bar:hover { color: red; }", "data-v-x").as_str(),
        ".foo[data-v-x],.bar[data-v-x]:hover{color: red;}"
    );
}

#[test]
fn keeps_global_selectors_unscoped_in_mixed_style_blocks() {
    assert_eq!(
        scope_css_for_pipeline(
            ":global(.probe) { color: red; }.local { color: blue; }",
            "data-v-x"
        )
        .as_str(),
        ".probe{color: red;}.local[data-v-x]{color: blue;}"
    );
    assert_eq!(
        scope_css_for_pipeline(
            ".host:global(.probe):hover, ::v-global(.badge:is(.primary, .secondary)) { color: red; }",
            "data-v-x"
        )
        .as_str(),
        ".probe,.badge:is(.primary, .secondary){color: red;}"
    );
}

#[test]
fn keeps_global_selectors_unscoped_inside_media_and_nested_rules() {
    assert_eq!(
        scope_css_for_pipeline(
            "@media (min-width: 1px) { .host { :global(.probe) { color: red; } .local { color: blue; } } }",
            "data-v-x"
        )
        .as_str(),
        "@media (min-width: 1px) {  .probe{color: red;} .host .local[data-v-x]{color: blue;}}"
    );
}

#[test]
fn global_argument_replaces_the_entire_selector() {
    assert_eq!(
        scope_css_for_pipeline(
            ".wrap :global(.msg), :global(.badge) .wrap, .a > ::v-global(.b) .c { color: red; }",
            "data-v-x"
        )
        .as_str(),
        ".msg,.badge,.b{color: red;}"
    );
}

#[test]
fn scopes_rules_inside_document_and_scope_at_rules() {
    assert_eq!(
        scope_css_for_pipeline(
            "@scope (.card) { .child { color: red; } }@document url-prefix() { .doc { color: blue; } }@-moz-document url-prefix() { .moz { color: green; } }",
            "data-v-x"
        )
        .as_str(),
        "@scope (.card) { .child[data-v-x]{color: red;}}@document url-prefix() { .doc[data-v-x]{color: blue;}}@-moz-document url-prefix() { .moz[data-v-x]{color: green;}}"
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
fn preserves_nested_where_parent_specificity() {
    assert_eq!(
        scope_css_for_pipeline(
            ":where(.card) { color: red; & > .title { font-weight: bold; } & > :deep(.icon) { display: block; } }",
            "data-v-x"
        )
        .as_str(),
        ":where(.card) {&[data-v-x]{color: red;} & > .title[data-v-x]{ font-weight: bold; } &[data-v-x] > .icon{ display: block; }}"
    );
}

#[test]
fn scopes_keyframes_when_unrelated_rules_use_nesting() {
    assert_eq!(
        scope_css_for_pipeline(
            ".card{animation:spin 1s linear infinite;animation-name:spin;-webkit-animation-name:spin;& :deep(.title){color:red;}}@keyframes spin{to{opacity:0;}}@-webkit-keyframes spin{to{opacity:0;}}",
            "data-v-b84fad59"
        )
        .as_str(),
        ".card[data-v-b84fad59]{animation:spin-b84fad59 1s linear infinite;animation-name:spin-b84fad59;-webkit-animation-name:spin-b84fad59;}.card[data-v-b84fad59] .title{color:red;}@keyframes spin-b84fad59{to{opacity:0;}}@-webkit-keyframes spin-b84fad59{to{opacity:0;}}"
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
