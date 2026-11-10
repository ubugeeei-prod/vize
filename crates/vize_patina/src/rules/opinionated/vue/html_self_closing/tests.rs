use super::{
    HtmlSelfClosing, HtmlSelfClosingHtmlOptions, HtmlSelfClosingNuxt, HtmlSelfClosingOptions,
    HtmlSelfClosingStyle,
};
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(HtmlSelfClosing::default()));
    Linter::with_registry(registry)
}

fn create_configured_linter(options: HtmlSelfClosingOptions) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(HtmlSelfClosing::new(options)));
    Linter::with_registry(registry)
}

#[test]
fn test_invalid_self_closing_void() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<main><img /></main>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].start, 6);
    assert_eq!(result.diagnostics[0].end, 13);
}

#[test]
fn test_invalid_self_closing_void_in_sfc() {
    let source = r#"<template>
  <view class="card">
    <div><img :src="imgUrl" alt="" /></div>
  </view>
</template>
<script lang="ts">export default {}</script>
"#;
    let result = create_linter().lint_sfc(source, "card.vue");
    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
}

#[test]
fn test_ignores_markup_inside_noscript() {
    let result = create_linter().lint_sfc(
        "<template><main><noscript><br /></noscript></main></template>",
        "home.vue",
    );
    assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
}

#[test]
fn test_valid_void_not_self_closing() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<img>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn plain_html_documents_do_not_receive_vue_self_closing_advice() {
    let linter = create_linter();
    for filename in ["index.html", "INDEX.HTM", "untitled"] {
        let result = linter.lint_standalone_html(
            r#"<!doctype html><html><head><meta charset="UTF-8" /></head><body><div id="app"></div></body></html>"#,
            filename,
        );
        assert_eq!(
            result.warning_count, 0,
            "{filename}: {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_valid_component_self_closing() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_kebab_case_vue_builtins_self_closing() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div><transition-group tag="div" name="fade" /><keep-alive /><TransitionGroup /></div>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_empty_component() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<section><MyComponent></MyComponent></section>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].start, 9);
    assert_eq!(result.diagnostics[0].end, 22);
}

#[test]
fn test_invalid_empty_lowercase_single_word_component() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<main><cell></cell></main>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_flags_empty_direct_template_root() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent></MyComponent>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (0, 13)
    );
}

#[test]
fn test_empty_html_and_whitespace_only_content_use_the_start_tag_range() {
    let linter = create_linter();
    let source = "<main><div> \n </div></main>";
    let result = linter.lint_template(source, "test.vue");
    assert_eq!(result.warning_count, 1);
    let start = source.find("<div>").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(result.diagnostics[0].end, start + 5);
}

#[test]
fn test_empty_slot_is_an_html_element() {
    let linter = create_linter();
    let source = "<main><slot></slot></main>";
    let result = linter.lint_template(source, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].start, 6);
    assert_eq!(result.diagnostics[0].end, 12);
}

#[test]
fn test_valid_component_with_content() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent>content</MyComponent>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_empty_nuxt_child_builtin_uses_the_component_policy() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<nuxt-child id="index"></nuxt-child>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (0, 23)
    );
}

fn create_nuxt_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(HtmlSelfClosingNuxt::default()));
    Linter::with_registry(registry)
}

#[test]
fn test_nuxt_preset_allows_vuetify_tags() {
    let linter = create_nuxt_linter();
    let result = linter.lint_template(
        r#"<v-dialog><v-btn></v-btn><v-icon></v-icon><v-spacer /></v-dialog>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_default_still_flags_empty_vuetify_tags() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<main><v-btn></v-btn></main>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_nuxt_preset_still_flags_other_components() {
    let linter = create_nuxt_linter();
    let result = linter.lint_template(r#"<main><MyComponent></MyComponent></main>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_configured_option_matches_requested_eslint_vue_shape() {
    let linter = create_configured_linter(HtmlSelfClosingOptions {
        html: HtmlSelfClosingHtmlOptions {
            void: HtmlSelfClosingStyle::Any,
            normal: HtmlSelfClosingStyle::Never,
            component: HtmlSelfClosingStyle::Any,
        },
        svg: HtmlSelfClosingStyle::Any,
        math: HtmlSelfClosingStyle::Any,
    });
    let result = linter.lint_template(
        r#"<main><img><div /><MyComponent></MyComponent><svg><path></path></svg><math><mi></mi></math></main>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].message,
        "Element must not use self-closing syntax"
    );
}

#[test]
fn configured_always_recognizes_authored_void_elements_as_empty() {
    let linter = create_configured_linter(HtmlSelfClosingOptions {
        html: HtmlSelfClosingHtmlOptions {
            void: HtmlSelfClosingStyle::Always,
            ..HtmlSelfClosingHtmlOptions::default()
        },
        ..HtmlSelfClosingOptions::default()
    });
    let result = linter.lint_sfc(
        "<template><img src=\"/logo.png\"></template>",
        "LogoView.vue",
    );
    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].message,
        "Void element should be self-closing"
    );
}

#[test]
fn test_configured_never_flags_self_closing_components_svg_and_math() {
    let linter = create_configured_linter(HtmlSelfClosingOptions {
        html: HtmlSelfClosingHtmlOptions {
            component: HtmlSelfClosingStyle::Never,
            ..HtmlSelfClosingHtmlOptions::default()
        },
        svg: HtmlSelfClosingStyle::Never,
        math: HtmlSelfClosingStyle::Never,
    });
    let result = linter.lint_template(
        r#"<main><MyComponent /><svg><path /></svg><math><mi /></math></main>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 3);
}
