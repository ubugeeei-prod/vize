use super::HoverService;
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{Hover, Position, Range, Url};
use vize_canon::{LspHover, LspHoverContents, LspMarkupContent, LspPosition, LspRange};

const ORIGINAL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/native-attribute-hover-range-original/Field.vue.txt"
);
const CRLF: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/native-attribute-hover-range-original/Field.crlf.vue.txt"
);
const UNICODE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/native-attribute-hover-range-original/Field.unicode.vue.txt"
);
const UNICODE_CRLF: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/native-attribute-hover-range-original/Field.unicode-crlf.vue.txt"
);

fn project(source: &str, marker: &str, contents: LspHoverContents) -> Hover {
    let state = ServerState::new();
    let uri = Url::parse("file:///tmp/NativeAttribute7993.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let offset = source.find(marker).unwrap() + 1;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let (name, tag, source_span) =
        crate::ide::definition::helpers::get_attribute_with_source_span_at_offset(&ctx).unwrap();
    assert_eq!(
        crate::ide::definition::helpers::get_attribute_and_component_at_offset(&ctx),
        Some((name, tag))
    );
    let native = LspHover {
        contents,
        range: Some(LspRange {
            start: LspPosition {
                line: 5,
                character: 17,
            },
            end: LspPosition {
                line: 5,
                character: 24,
            },
        }),
    };
    let original_contents = HoverService::convert_lsp_hover(native.clone()).contents;
    let result = HoverService::project_native_attribute_hover(&ctx, source_span, native);
    assert_eq!(result.contents, original_contents);
    result
}

fn complete_markup() -> LspHoverContents {
    LspHoverContents::Markup(LspMarkupContent {
        kind: "markdown".into(),
        value: "```typescript\n(property) HTMLLabelElement.htmlFor: string\n```\n\nComplete DOM description Ω🧭.\n\n[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLLabelElement/htmlFor)".into(),
    })
}

#[test]
fn reported_native_attribute_aliases_select_the_original_name_and_keep_complete_contents() {
    for source in [ORIGINAL, CRLF] {
        for (marker, line, end) in [(":for", 5, 13), (":id", 6, 12)] {
            let result = project(source, marker, complete_markup());
            assert_eq!(
                result.range,
                Some(Range::new(
                    Position::new(line, 10),
                    Position::new(line, end)
                ))
            );
        }
    }
}

#[test]
fn native_attribute_name_ranges_count_physical_utf16_after_unicode_and_crlf() {
    for source in [UNICODE, UNICODE_CRLF] {
        for (marker, line, start, end) in [(":for", 5, 23, 26), (":id", 6, 22, 24)] {
            let result = project(source, marker, complete_markup());
            assert_eq!(
                result.range,
                Some(Range::new(
                    Position::new(line, start),
                    Position::new(line, end)
                ))
            );
        }
    }
}

#[test]
fn longhand_and_static_native_names_do_not_select_binding_prefixes_or_reflected_properties() {
    let source = "<template><label v-bind:for=\"id\"/><input id=\"field\"/></template>";
    for (marker, start, end) in [("v-bind:for", 24, 27), ("id=", 41, 43)] {
        let result = project(
            source,
            marker,
            LspHoverContents::String("Complete original type and documentation".into()),
        );
        assert_eq!(
            result.range,
            Some(Range::new(Position::new(0, start), Position::new(0, end)))
        );
    }
}

#[test]
fn attribute_values_stay_outside_the_name_projection_and_unprovided_spans_drop_virtual_ranges() {
    let state = ServerState::new();
    let uri = Url::parse("file:///tmp/NativeAttribute7993.vue").unwrap();
    state
        .documents
        .open(uri.clone(), ORIGINAL.into(), 1, "vue".into());
    let offset = ORIGINAL.find(":id=\"id\"").unwrap() + 5;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    assert!(
        crate::ide::definition::helpers::get_attribute_with_source_span_at_offset(&ctx).is_none()
    );
    assert!(crate::ide::definition::helpers::get_attribute_and_component_at_offset(&ctx).is_none());
    let result = project(
        "<template><input v-model:title=\"id\"/></template>",
        "v-model:title",
        complete_markup(),
    );
    assert_eq!(result.range, None);
}
