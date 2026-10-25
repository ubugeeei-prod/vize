use super::HoverService;
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{Hover, Position, Range, Url};
use vize_canon::{
    LspHover, LspHoverContents, LspMarkedString, LspMarkupContent, LspPosition, LspRange,
};

fn contents() -> [LspHoverContents; 4] {
    [
        LspHoverContents::Markup(LspMarkupContent {
            kind: "markdown".into(),
            value: "```typescript\nconst __vizeDomElement: HTMLButtonElement\n```\n\nOriginal DOM documentation Ω🧭.".into(),
        }),
        LspHoverContents::Markup(LspMarkupContent {
            kind: "plaintext".into(),
            value: "Original DOM documentation Ω🧭.".into(),
        }),
        LspHoverContents::String("Original DOM documentation Ω🧭.".into()),
        LspHoverContents::Array(vec![
            LspMarkedString::String("Original DOM documentation Ω🧭.".into()),
            LspMarkedString::LanguageString {
                language: "typescript".into(),
                value: "const __vizeDomElement: HTMLButtonElement".into(),
            },
        ]),
    ]
}

fn project(source: &str, offset: usize, contents: LspHoverContents) -> Hover {
    let state = ServerState::new();
    let uri = Url::parse("file:///tmp/NativeTag10449.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let (tag, span) =
        crate::ide::definition::helpers::get_tag_with_source_span_at_offset(source, offset)
            .unwrap();
    assert_eq!(
        crate::ide::definition::helpers::get_tag_at_offset(source, offset),
        Some(tag)
    );
    let native = LspHover {
        contents,
        range: Some(LspRange {
            start: LspPosition {
                line: 5,
                character: 0,
            },
            end: LspPosition {
                line: 5,
                character: 16,
            },
        }),
    };
    let expected_contents = HoverService::convert_lsp_hover(native.clone()).contents;
    let projected = HoverService::project_native_dom_hover(&ctx, Some(span), native);
    assert_eq!(projected.contents, expected_contents);
    projected
}

#[test]
fn native_tag_ranges_replace_helper_coordinates_with_full_authored_utf16_names() {
    for newline in ["\n", "\r\n"] {
        for native_contents in contents() {
            let source = "<template>\n  🧭<button\n    disabled\n  >Save</button>\n</template>\n"
                .replace('\n', newline);
            let opening = source.find("button").unwrap();
            let closing = source.rfind("button").unwrap();
            for (start, range) in [
                (
                    opening,
                    Range::new(Position::new(1, 5), Position::new(1, 11)),
                ),
                (
                    closing,
                    Range::new(Position::new(3, 9), Position::new(3, 15)),
                ),
            ] {
                // Both ends of the tag retain the name-only lookup's existing
                // inclusive cursor policy, including a multiline opening tag.
                for cursor in [start, start + 3, start + 6] {
                    assert_eq!(
                        project(&source, cursor, native_contents.clone()).range,
                        Some(range)
                    );
                }
            }
        }
    }
}

#[test]
fn native_tag_ranges_share_authored_pug_and_jade_name_spans() {
    for language in ["pug", "Pug", "PUG", "jade", "JADE"] {
        for newline in ["\n", "\r\n"] {
            let source = format!("<!-- Ω🧭 -->\n<template lang=\"{language}\">\n  button#save.action(disabled) Save\n</template>\n").replace('\n', newline);
            let start = source.find("button#").unwrap();
            for native_contents in contents() {
                for cursor in [start, start + 3, start + 6] {
                    assert_eq!(
                        project(&source, cursor, native_contents.clone()).range,
                        Some(Range::new(Position::new(2, 2), Position::new(2, 8)))
                    );
                }
            }
        }
    }
}

#[test]
fn tag_span_lookup_preserves_complete_hyphenated_names_and_excludes_attribute_values() {
    let source = "<template>🧭<my-widget title=\"button\" /></template>";
    let start = source.find("my-widget").unwrap();
    for cursor in [start, start + 3, start + 9] {
        assert_eq!(
            crate::ide::definition::helpers::get_tag_with_source_span_at_offset(source, cursor),
            Some(("my-widget".into(), (start, start + 9)))
        );
    }
    for cursor in [
        source.find("title").unwrap(),
        source.find("button").unwrap(),
        source.len(),
    ] {
        assert_eq!(
            crate::ide::definition::helpers::get_tag_with_source_span_at_offset(source, cursor),
            None
        );
        assert_eq!(
            crate::ide::definition::helpers::get_tag_at_offset(source, cursor),
            None
        );
    }
}
