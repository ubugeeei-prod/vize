use super::{Document, documentation, hover_documentation, item};

#[test]
fn named_color_hover_keeps_the_whole_packet_with_a_stylesheet_example() {
    let document = Document::css(".demo { color: re|d; }", false);
    let hover = super::super::hover(&document.context(), 0).unwrap();
    let expected = serde_json::json!({
        "contents": {
            "kind": "markdown",
            "value": concat!(
                "**red**\n\n_CSS value for color_\n\n#ff0000\n\n**Example**\n\n",
                "```css\n.example {\n  color: red;\n}\n```\n\n",
                "**Property syntax**\n\n```css\n<color>\n```\n\n**Docs**\n\n",
                "[MDN CSS named colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color)",
                "\n\n**Docs**\n\n",
                "[MDN Reference](https://developer.mozilla.org/docs/Web/CSS/Reference/Properties/color)"
            )
        },
        "range": {
            "start": { "line": 2, "character": 15 },
            "end": { "line": 2, "character": 18 }
        }
    });
    assert_eq!(serde_json::to_value(hover).unwrap(), expected);
}

#[test]
fn real_sfc_property_value_pseudo_and_at_rule_hovers_match_selected_completion_docs() {
    for (needle, offset, label, source) in [
        ("display: flex", 3, "display", "Properties/display"),
        ("display: flex", 11, "flex", "Properties/display"),
        ("color: red", 8, "red", "Properties/color"),
        ("width: auto", 9, "auto", "Properties/width"),
        ("&:hover", 4, ":hover", "Selectors/:hover"),
        ("::before", 5, "::before", "Selectors/::before"),
        ("@media", 4, "@media", "At-rules/@media"),
    ] {
        let document = Document::fixture(needle, offset);
        let hover = super::super::hover(&document.context(), 0).unwrap();
        let docs = hover_documentation(&hover);
        assert!(docs.contains(&["**", label, "**"].concat()), "{docs}");
        assert!(
            docs.contains(
                &[
                    "https://developer.mozilla.org/docs/Web/CSS/Reference/",
                    source
                ]
                .concat()
            )
        );
        assert!(hover.range.is_some());
        assert_eq!(docs, documentation(item(&document.complete(), label)));
        assert!(!document.state.is_lsp_typecheck_enabled());
    }
}

#[test]
fn vue_hover_explains_actual_scoping_and_uses_specific_official_sections() {
    for (needle, offset, example, explanation, section) in [
        (
            "v-bind",
            3,
            "v-bind('theme.color')",
            "both plain and scoped style blocks",
            "v-bind-in-css",
        ),
        (
            ":deep",
            3,
            ".parent :deep(.child)",
            "selector inside :deep() stays unscoped",
            "deep-selectors",
        ),
        (
            ":slotted",
            4,
            ":slotted(div)",
            "belongs to the component that provides it",
            "slotted-selectors",
        ),
        (
            ":global",
            4,
            ":global(.red)",
            "match outside the component",
            "global-selectors",
        ),
    ] {
        let document = Document::fixture(needle, offset);
        let hover = super::super::hover(&document.context(), 0).unwrap();
        let docs = hover_documentation(&hover);
        assert!(docs.contains("**Example**\n\n```css"));
        assert!(docs.contains(example), "{docs}");
        assert!(docs.contains(explanation), "{docs}");
        assert!(docs.contains(&["https://vuejs.org/api/sfc-css-features.html#", section].concat()));
        assert!(!docs.contains("$1"));
        assert!(
            hover.range.is_none(),
            "original Vue feature range behavior stays intact"
        );
    }
}

#[test]
fn v_bind_is_documented_in_a_plain_style_block_and_unknown_css_has_no_hover() {
    let document = Document::marked(
        "<template><div /></template><style>.demo { color: v-bi|nd(theme); }</style>",
        false,
    );
    let hover = super::super::hover(&document.context(), 0).unwrap();
    assert!(hover_documentation(&hover).contains("both plain and scoped style blocks"));
    for css in [
        ".demo { /* co|lor: red; */ }",
        ".demo { content: 'co|lor'; }",
        ".demo { --theme: re|d; }",
        ".demo { unknown-prop|erty: value; }",
    ] {
        let document = Document::css(css, false);
        assert!(super::super::hover(&document.context(), 0).is_none());
    }
}

#[test]
fn other_style_dialects_keep_original_vue_features_without_standard_css_candidates() {
    for language in ["sass", "stylus", "unknown"] {
        for feature in ["v-bind", ":deep", ":slotted", ":global"] {
            let source = [
                "<style lang=\"",
                language,
                "\">",
                feature,
                "|(theme)</style>",
            ]
            .concat();
            let document = Document::marked(&source, false);
            let items = document.complete();
            assert_eq!(items.len(), 4);
            let hover = super::super::hover(&document.context(), 0).unwrap();
            assert_eq!(hover.range, None);
            assert_eq!(
                hover_documentation(&hover),
                documentation(item(&items, feature))
            );
        }
        let document = Document::marked(
            &["<style lang=\"", language, "\">.a { co|lor: red; }</style>"].concat(),
            false,
        );
        assert_eq!(document.complete().len(), 4);
        assert!(super::super::hover(&document.context(), 0).is_none());
    }
}
