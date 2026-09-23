use super::{apply_scoped_css, extract_css_vars, scope_selector, transform_deep, transform_global};

#[test]
fn test_scope_simple_selector() {
    let result = scope_selector(".foo", "[data-v-123]");
    assert_eq!(result, ".foo[data-v-123]");
}

#[test]
fn test_scope_functional_pseudo_class_keeps_inner_intact() {
    // #971: a colon inside `:not(...)` / `:is(...)` / `:where(...)` /
    // `:has(...)` must not be where the scope attribute lands. The
    // scope attaches to the compound selector instead.
    let result = scope_selector(".x:not(:checked)", "[data-v-123]");
    assert_eq!(result, ".x[data-v-123]:not(:checked)");

    let result = scope_selector(".btn:is(:hover, :focus)", "[data-v-123]");
    assert_eq!(result, ".btn[data-v-123]:is(:hover, :focus)");
}

#[test]
fn test_scope_v_deep_combinator_form() {
    // Legacy `::v-deep` combinator form normalizes to `:deep(...)`.
    let result = scope_selector(".foo ::v-deep .bar", "[data-v-123]");
    assert_eq!(result, ".foo[data-v-123] .bar");
}

#[test]
fn test_scope_v_deep_function_form() {
    // Legacy `::v-deep(.x)` function form normalizes to `:deep(.x)`.
    let result = scope_selector(".foo ::v-deep(.bar)", "[data-v-123]");
    assert_eq!(result, ".foo[data-v-123] .bar");
}

#[test]
fn test_scope_legacy_deep_combinators() {
    // Both `>>>` and `/deep/` normalize to `:deep(...)` wrapping the
    // remainder.
    let result = scope_selector(".foo >>> .bar", "[data-v-123]");
    assert_eq!(result, ".foo[data-v-123] .bar");

    let result = scope_selector(".foo /deep/ .bar", "[data-v-123]");
    assert_eq!(result, ".foo[data-v-123] .bar");
}

#[test]
fn test_scope_v_slotted_function_form() {
    let result = scope_selector("::v-slotted(.bar)", "[data-v-123]");
    assert_eq!(result, ".bar[data-v-123]-s");
}

#[test]
fn test_scope_descendant_selector() {
    let result = scope_selector(".foo .bar", "[data-v-123]");
    assert_eq!(result, ".foo .bar[data-v-123]");
}

#[test]
fn test_scope_multiple_selectors() {
    let result = scope_selector(".foo, .bar", "[data-v-123]");
    assert_eq!(result, ".foo[data-v-123], .bar[data-v-123]");
}

#[test]
fn test_transform_deep() {
    let result = transform_deep(":deep(.child)", "[data-v-123]");
    assert_eq!(result, "[data-v-123] .child");
}

#[test]
fn test_transform_deep_after_child_combinator() {
    let result = transform_deep(".sponsors__item > :deep(.sponsor)", "[data-v-123]");
    assert_eq!(result, ".sponsors__item[data-v-123] > .sponsor");
}

#[test]
fn test_apply_scoped_css_deep_after_child_combinator() {
    let css = ".sponsors__item > :deep(.sponsor) { width: 100%; }";
    let result = apply_scoped_css(css, "data-v-123");
    assert_eq!(
        result,
        ".sponsors__item[data-v-123] > .sponsor{ width: 100%; }"
    );
}

#[test]
fn test_apply_scoped_css_preserves_deep_comment_before_selector() {
    let css = "/* override :deep(p) from the parent */\n.foo { color: red; }";
    let result = apply_scoped_css(css, "data-v-123");

    assert_eq!(
        result,
        "/* override :deep(p) from the parent */\n.foo[data-v-123]{ color: red; }"
    );
}

#[test]
fn test_apply_scoped_css_preserves_deep_comment_inside_at_rule() {
    let css = "@media (min-width: 1px) { /* A <span>, not a <p>; ignore :deep(p). */\n.conferences__venue { display: block; } }";
    let result = apply_scoped_css(css, "data-v-abc");

    assert_eq!(
        result,
        "@media (min-width: 1px){ /* A <span>, not a <p>; ignore :deep(p). */\n.conferences__venue[data-v-abc]{ display: block; }}"
    );
}

#[test]
fn test_apply_scoped_css_stray_closing_brace_keeps_scoping() {
    let css = "}.foo { color: red; }.bar { color: blue; }";
    let result = apply_scoped_css(css, "data-v-123");

    assert_eq!(
        result,
        "}.foo[data-v-123]{ color: red; }.bar[data-v-123]{ color: blue; }"
    );
}

#[test]
fn test_transform_global() {
    let result = transform_global(":global(.foo)");
    assert_eq!(result, ".foo");
}

#[test]
fn test_extract_css_vars() {
    let css = ".foo { color: v-bind(color); background: v-bind('bgColor'); }";
    let vars = extract_css_vars(css);
    assert_eq!(vars, vec!["color", "bgColor"]);
}

#[test]
fn test_extract_css_vars_with_quoted_parentheses() {
    let css = r#"
.header {
  background-color: color(from v-bind("parentBg ?? 'var(--bg)'") srgb r g b / 0.85);
}
.textCountGraph {
  background-image: conic-gradient(
var(--countColor) 0% v-bind("Math.min(100, textCountPercentage) + '%'"),
rgba(0, 0, 0, .2) v-bind("Math.min(100, textCountPercentage) + '%'") 100%
  );
}
"#;
    let vars = extract_css_vars(css);
    assert_eq!(
        vars,
        vec![
            "parentBg ?? 'var(--bg)'",
            "Math.min(100, textCountPercentage) + '%'",
            "Math.min(100, textCountPercentage) + '%'",
        ]
    );
}

#[test]
fn test_scope_media_query() {
    let css = "@media (max-width: 768px) { .foo { color: red; } }";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_scope_media_query_with_comment() {
    let css = "/* Mobile responsive */\n@media (max-width: 768px) {\n  .glyph-playground {\n    grid-template-columns: 1fr;\n  }\n}";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_scope_keyframes() {
    let css =
        "@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_scope_webkit_keyframes() {
    let css = "@-webkit-keyframes fade { 0% { opacity: 0; } 100% { opacity: 1; } }";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_nested_css_media_passthrough() {
    // CSS nesting: @media (--mobile) inside a selector should pass through
    let css = "#pages-store {\n  display: grid;\n  row-gap: 1.5rem;\n  @media (--mobile) {\n    row-gap: 1rem;\n  }\n  h1 {\n    padding: 7.5rem 0;\n    @media (--mobile) {\n      padding: 2.5rem 0.75rem;\n    }\n  }\n}";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_root_level_media_with_custom_query() {
    // Root-level @media with custom media query
    let css = ".foo { color: red; }\n@media (--mobile) { .foo { font-size: 12px; } }";
    let result = apply_scoped_css(css, "data-v-abc");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_apply_scoped_css_at_import() {
    let css = "@import \"~/assets/styles/custom-media-query.css\";\n\nfooter { width: 100%; }";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_apply_scoped_css_at_import_with_nested_css() {
    let css = "@import \"custom.css\";\n\nfooter {\n  width: 100%;\n  @media (--mobile) {\n    padding: 1rem;\n  }\n}";
    let result = apply_scoped_css(css, "data-v-abc");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_scope_keyframes_inside_media() {
    let css = "@media (prefers-reduced-motion: no-preference) { @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } .foo { color: red; } }";
    let result = apply_scoped_css(css, "data-v-123");
    insta::assert_snapshot!(result.as_str());
}
