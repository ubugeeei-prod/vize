use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::{Document, documentation, edit, item};

#[test]
fn font_family_is_available_in_an_ordinary_declaration_from_the_complete_catalog() {
    let items = Document::css(".demo { font-fa| }", false).complete();
    let font_family = item(&items, "font-family");
    assert_eq!(font_family.kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(edit(font_family).new_text, "font-family: ");
    assert!(documentation(font_family).contains("Properties/font-family"));
    assert!(documentation(font_family).contains("**Syntax**"));
}

#[test]
fn original_vue_order_and_all_insertion_fields_stay_exact() {
    let items = Document::css(".demo { col| }", false).complete();
    let expected = [
        ("v-bind", "Vue CSS: v-bind()", "v-bind($1)"),
        (":deep", "Vue CSS: :deep()", ":deep($1)"),
        (":slotted", "Vue CSS: :slotted()", ":slotted($1)"),
        (":global", "Vue CSS: :global()", ":global($1)"),
    ];
    for (actual, (label, detail, snippet)) in items.iter().zip(expected) {
        let expected = CompletionItem {
            label: label.into(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(detail.into()),
            insert_text: Some(snippet.into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        };
        let mut insertion = actual.clone();
        insertion.documentation = None;
        assert_eq!(insertion, expected);
        assert!(documentation(actual).contains("**Example**"));
        assert!(!documentation(actual).contains("$1"));
    }
    assert!(items.len() > 4);
}

#[test]
fn property_completion_filters_prefix_and_replaces_whole_token_with_one_colon() {
    for (css, replacement) in [
        (".demo { col| }", "color: "),
        (".demo { co|lor: red; }", "color"),
    ] {
        let document = Document::css(css, false);
        let items = document.complete();
        let color = item(&items, "color");
        assert_eq!(color.kind, Some(CompletionItemKind::PROPERTY));
        assert_eq!(color.insert_text_format, Some(InsertTextFormat::PLAIN_TEXT));
        assert_eq!(edit(color).new_text, replacement);
        let ctx = document.context();
        let token_start = ctx.content.find("{ ").unwrap() + 2;
        let token_length = if replacement == "color" { 5 } else { 3 };
        let start = crate::ide::offset_to_position(&ctx.content, token_start);
        let end = crate::ide::offset_to_position(&ctx.content, token_start + token_length);
        assert_eq!(edit(color).range.start.line, start.0);
        assert_eq!(edit(color).range.start.character, start.1);
        assert_eq!(edit(color).range.end.line, end.0);
        assert_eq!(edit(color).range.end.character, end.1);
        assert!(items[4..].iter().all(|item| item.label.starts_with("co")));
        assert!(!items.iter().any(|item| item.label == "width"));
        let docs = documentation(color);
        assert!(docs.contains("Sets the color of an element's text"));
        assert!(docs.contains("**Syntax**\n\n```css\n<color>"));
        assert!(
            docs.contains("https://developer.mozilla.org/docs/Web/CSS/Reference/Properties/color")
        );
    }
}

#[test]
fn value_completions_remain_owned_and_colors_are_offered_only_for_color_properties() {
    let display = Document::css(".demo { display: fl|; }", false).complete();
    let flex = item(&display, "flex");
    assert_eq!(edit(flex).new_text, "flex");
    assert!(documentation(flex).contains("display: flex;"));
    assert!(!display.iter().any(|item| item.label == "auto"));

    let colors = Document::css(".demo { color: re|; }", false).complete();
    let red = item(&colors, "red");
    assert_eq!(edit(red).new_text, "red");
    assert!(documentation(red).contains("#ff0000"));
    assert!(documentation(red).contains("color: red;"));
    assert!(
        documentation(red).contains("https://developer.mozilla.org/en-US/docs/Web/CSS/named-color")
    );
    item(&colors, "rebeccapurple");
    assert!(!colors.iter().any(|item| item.label == "flex"));

    let width = Document::css(".demo { width: au|; }", false).complete();
    assert_eq!(edit(item(&width, "auto")).new_text, "auto");
    assert!(documentation(item(&width, "auto")).contains("width: auto;"));
    let width = Document::css(".demo { width: re|; }", false).complete();
    assert!(
        !width
            .iter()
            .any(|item| matches!(item.label.as_str(), "red" | "rebeccapurple"))
    );
}

#[test]
fn every_standard_wide_keyword_has_selected_example_and_its_specification_section() {
    let items = Document::css(".demo { display: |; }", false).complete();
    for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        let selected = item(&items, keyword);
        assert_eq!(edit(selected).new_text, keyword);
        let docs = documentation(selected);
        assert!(docs.contains(&["display: ", keyword, ";"].concat()));
        assert!(docs.contains(&["https://www.w3.org/TR/css-cascade-5/#", keyword].concat()));
    }
}

#[test]
fn comments_strings_and_custom_properties_keep_the_existing_vue_fallback() {
    for css in [
        ".demo { /* display: fl| */ }",
        ".demo { content: 'display: fl|'; }",
        ".demo { --theme: re|; }",
        ".demo { --co|: red; }",
    ] {
        let items = Document::css(css, false).complete();
        assert_eq!(
            items
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            ["v-bind", ":deep", ":slotted", ":global"]
        );
    }
}

#[test]
fn pseudo_and_at_rule_completions_keep_full_names_and_primary_catalog_links() {
    for (css, label, link, text) in [
        (
            ".demo:ho| {}",
            ":hover",
            "Selectors/:hover",
            "pointing device",
        ),
        (
            ".demo::bef| {}",
            "::before",
            "Selectors/::before",
            "pseudo-element",
        ),
        (
            "@med| (min-width: 40rem) {}",
            "@media",
            "At-rules/@media",
            "media type",
        ),
    ] {
        let items = Document::css(css, false).complete();
        let selected = item(&items, label);
        assert_eq!(edit(selected).new_text, label);
        let docs = documentation(selected);
        assert!(docs.contains(text), "{docs}");
        assert!(
            docs.contains(
                &[
                    "https://developer.mozilla.org/docs/Web/CSS/Reference/",
                    link
                ]
                .concat()
            )
        );
        assert!(docs.contains("**Docs**"));
    }
}
