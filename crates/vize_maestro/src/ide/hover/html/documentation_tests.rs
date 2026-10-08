use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range, Url};
use vize_canon::{
    LspHover, LspHoverContents, LspMarkedString, LspMarkupContent, LspPosition, LspRange,
};

use super::{HoverService, documentation, elements};
use crate::{
    ide::{IdeContext, position_to_offset},
    server::ServerState,
};

const APP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/html-element-hover-documentation/App.vue.txt"
);
const CRLF: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/html-element-hover-documentation/App.crlf.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/html-element-hover-documentation/controls.expected.json"
);
const LINKS: &str = "\n\n**Docs**\n\n[MDN reference](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/button)\n\n[HTML Living Standard](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)";
const DESCRIPTION: &str =
    "\n\nProvides an interactive button that can trigger an action or submit a form.";

#[test]
fn complete_fallback_hovers_preserve_authored_utf16_ranges_for_open_closing_and_multiline_tags() {
    let expected: Vec<serde_json::Value> = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(expected.len(), 20);
    for source in [APP, CRLF] {
        let uri = Url::parse("file:///tmp/HtmlDocumentation.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        state.update_virtual_docs(&uri, source);
        for row in &expected {
            let position: Position = serde_json::from_value(row["position"].clone()).unwrap();
            let offset = position_to_offset(source, position.line, position.character).unwrap();
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let actual = crate::runtime::block_on(HoverService::hover_with_corsa(&ctx, None));
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                row["result"],
                "{} {}",
                row["tag"],
                row["role"]
            );
            // Observe the same synchronous routing without async test executor
            // scheduling: unrelated paths must never enter descriptor lookup.
            elements::take_lookup_count();
            let _ = HoverService::hover(&ctx);
            assert_eq!(
                elements::take_lookup_count(),
                usize::from(row["role"] != "unrelated")
            );
        }
    }
}

#[test]
fn every_native_content_variant_keeps_complete_type_text_and_the_existing_range() {
    let originals = [
        (LspHoverContents::Markup(LspMarkupContent {
            kind: "markdown".into(),
            value: "```typescript\nconst __vizeDomElement: HTMLButtonElement\n```\n\nOriginal native Ω🧭 documentation.".into(),
        }), "```typescript\nconst __vizeDomElement: HTMLButtonElement\n```\n\nOriginal native Ω🧭 documentation."),
        (LspHoverContents::Markup(LspMarkupContent {
            kind: "plaintext".into(),
            value: "(const) __vizeDomElement: HTMLButtonElement".into(),
        }), "```typescript\n(const) __vizeDomElement: HTMLButtonElement\n```"),
        (LspHoverContents::String("(const) __vizeDomElement: HTMLButtonElement".into()), "```typescript\n(const) __vizeDomElement: HTMLButtonElement\n```"),
        (LspHoverContents::Array(vec![
            LspMarkedString::String("Native original docs Ω🧭.".into()),
            LspMarkedString::LanguageString { language: "typescript".into(), value: "const __vizeDomElement: HTMLButtonElement".into() },
        ]), "Native original docs Ω🧭.\n\n```typescript\nconst __vizeDomElement: HTMLButtonElement\n```"),
    ];
    for (contents, converted_original) in originals {
        let native = LspHover {
            contents,
            range: Some(LspRange {
                start: LspPosition {
                    line: 7,
                    character: 11,
                },
                end: LspPosition {
                    line: 7,
                    character: 21,
                },
            }),
        };
        let actual = documentation::enrich_native_html(
            HoverService::convert_lsp_hover(native),
            "button",
            "HTML element",
        );
        let expected = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: [converted_original, DESCRIPTION, LINKS].concat(),
            }),
            range: Some(Range::new(Position::new(7, 11), Position::new(7, 21))),
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn native_other_namespaces_and_builtins_return_the_original_hover_without_html_lookup() {
    let original = Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Whole original native type Ω🧭 and documentation.".into(),
        }),
        range: Some(Range::new(Position::new(2, 3), Position::new(2, 9))),
    };
    for tag in [
        "svg",
        "circle",
        "math",
        "mi",
        "Widget",
        "my-widget",
        "template",
        "param",
        "slot",
        "component",
        "teleport",
        "suspense",
    ] {
        elements::take_lookup_count();
        let category =
            crate::ide::corsa_support::native_dom_tag_info(tag).map_or("", |info| info.category);
        assert_eq!(
            documentation::enrich_native_html(original.clone(), tag, category),
            original
        );
        assert_eq!(
            elements::take_lookup_count(),
            0,
            "unexpected descriptor lookup for {tag}"
        );
    }
}

#[test]
fn carrying_native_category_preserves_the_complete_virtual_document_and_definition_offsets() {
    const PREFIX: &str = "/// <reference lib=\"es2022\" />\n/// <reference lib=\"dom\" />\n/// <reference lib=\"dom.iterable\" />\ntype __VizeDomElement = ";
    const SUFFIX: &str =
        ";\ndeclare const __vizeDomElement: __VizeDomElement;\n__vizeDomElement;\n";
    for (tag, category, expression, hover_offset) in [
        (
            "button",
            "HTML element",
            "HTMLElementTagNameMap[\"button\"]",
            203,
        ),
        (
            "circle",
            "SVG element",
            "SVGElementTagNameMap[\"circle\"]",
            202,
        ),
        (
            "math",
            "MathML element",
            "MathMLElementTagNameMap[\"math\"]",
            203,
        ),
    ] {
        let doc = crate::ide::corsa_support::html_tag_virtual_document(tag).unwrap();
        assert_eq!(doc.category, category);
        assert_eq!(doc.content.as_str(), [PREFIX, expression, SUFFIX].concat());
        assert_eq!(doc.definition_offset, 120);
        assert_eq!(doc.hover_offset, hover_offset);
    }
    for tag in [
        "component",
        "template",
        "slot",
        "teleport",
        "suspense",
        "param",
    ] {
        assert!(crate::ide::corsa_support::html_tag_virtual_document(tag).is_none());
    }
}
