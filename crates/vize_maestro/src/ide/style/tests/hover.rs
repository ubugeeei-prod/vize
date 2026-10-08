use super::{Document, documentation, hover_documentation, item};

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
