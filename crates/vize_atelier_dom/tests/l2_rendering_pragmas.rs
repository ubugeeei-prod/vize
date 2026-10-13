//! #8652: native development DOM and SSR must agree on annotation comments.

#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "corpus regressions inspect complete compiler output"
)]

use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_l0::Allocator;

const LEADING: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/dev-comment-hydration/leading.template.txt"
);
const PRAGMAS: &[&str] = &[
    "eslint-disable",
    "eslint-disable-line",
    "eslint-disable-next-line",
    "eslint-enable",
    "oxlint-disable",
    "oxlint-disable-line",
    "oxlint-disable-next-line",
    "oxlint-enable",
];

fn compile(source: &str, source_map: bool) -> (String, String) {
    let allocator = Allocator::new();
    let (errors, result) =
        compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            DomCompilerOptions { comments: true, source_map, ..Default::default() },
            TemplateSyntaxMode::Standard,
            None,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    (
        result.result.preamble.to_string(),
        result.result.code.to_string(),
    )
}

#[test]
fn selected_dom_omits_annotations_with_ordinary_comments_enabled() {
    assert_eq!(compile(LEADING, false), compile(LEADING, true));
    for pragma in PRAGMAS {
        for surrounding in [
            "<!--PRAGMA rule --><div>root</div>",
            "<!--PRAGMA rule -->",
            "<div><!--PRAGMA rule --></div>",
            "<div><!--PRAGMA rule --><i v-if=\"show\">yes</i><b v-else>no</b></div>",
            "<div><i v-for=\"item in items\" :key=\"item\"><!--PRAGMA rule -->{{ item }}</i></div>",
            "<template v-if=\"show\"><!--PRAGMA rule --><i>yes</i><b>more</b></template>",
            "<div>a <!--PRAGMA rule --> b {{ value }}</div>",
            "<slot><!--PRAGMA rule --><span>fallback</span></slot>",
            "<div><!--ordinary--><!--PRAGMA rule --><span>kept</span></div>",
        ] {
            let source = surrounding.replace("PRAGMA", pragma);
            assert_eq!(compile(&source, false), compile(&source, true), "{source}");
        }
        let annotated = format!("<Foo><!--{pragma} rule --><span>slot</span></Foo>");
        assert_eq!(
            compile(&annotated, false),
            compile("<Foo><span>slot</span></Foo>", false)
        );
        let list = "<template v-for=\"item in items\" :key=\"item\"><!--PRAGMA rule --><i>{{ item }}</i><b>more</b></template>";
        assert_eq!(
            compile(&list.replace("PRAGMA", pragma), false),
            compile(&list.replace("<!--PRAGMA rule -->", ""), false)
        );
    }
}

#[test]
fn ordinary_comments_and_similar_words_keep_their_rendered_children() {
    for content in [
        "ordinary",
        "eslint-disableish",
        "oxlint-disablement",
        "note eslint-disable",
        "eslint-disable\u{00a0}rule",
        "oxlint-disable\u{2003}rule",
        "\u{00a0}eslint-disable rule",
    ] {
        let source = format!("<div><!--{content}--><span>kept</span></div>");
        let selected = compile(&source, false);
        assert_eq!(selected, compile(&source, true), "{source}");
        let expected = (
            "const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue\n".to_owned(),
            format!(
                "function render(_ctx, _cache, $props, $setup, $data, $options) {{\n  return (_openBlock(), _createElementBlock(\"div\", null, [\n    _createCommentVNode(\"{content}\"),\n    _createElementVNode(\"span\", null, \"kept\")\n  ]))\n}}"
            ),
        );
        assert_eq!(selected, expected, "{source}");
    }
}
