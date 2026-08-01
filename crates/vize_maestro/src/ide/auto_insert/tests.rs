use tower_lsp::lsp_types::Url;

use super::*;
use crate::server::ServerState;

fn context<'a>(
    state: &'a ServerState,
    uri: &'a Url,
    source: String,
    offset: usize,
) -> IdeContext<'a> {
    IdeContext::with_content(state, uri, offset, source)
}

#[test]
fn markup_snippets_match_the_vue_language_server_oracle() {
    crate::runtime::block_on(async {
        let state = ServerState::new();
        let uri = Url::parse("file:///fixture.vue").unwrap();

        let source = "<template>{{}}</template>".to_string();
        let start = source.find("{{}}").unwrap() + 1;
        let ctx = context(&state, &uri, source, start + 1);
        assert_eq!(
            AutoInsertService::snippet(&ctx, start + 1, start, "{}").await,
            Some(" $0 ".to_string())
        );

        let source = "<template><div class=></div></template>".to_string();
        let start = source.find('=').unwrap();
        let ctx = context(&state, &uri, source, start + 1);
        assert_eq!(
            AutoInsertService::snippet(&ctx, start + 1, start, "=").await,
            Some("\"$1\"".to_string())
        );

        let source = "<template><section></template>".to_string();
        let start = source[10..].find('>').unwrap() + 10;
        let ctx = context(&state, &uri, source, start + 1);
        assert_eq!(
            AutoInsertService::snippet(&ctx, start + 1, start, ">").await,
            Some("$0</section>".to_string())
        );

        let source = "<template><section title=\"<x>\"></template>".to_string();
        let start = source.find("\"></template>").unwrap() + 1;
        let ctx = context(&state, &uri, source, start + 1);
        assert_eq!(
            AutoInsertService::snippet(&ctx, start + 1, start, ">").await,
            Some("$0</section>".to_string())
        );

        let source = "<template><section></</template>".to_string();
        let start = source.find("</").unwrap() + 1;
        let ctx = context(&state, &uri, source, start + 1);
        assert_eq!(
            AutoInsertService::snippet(&ctx, start + 1, start, "/").await,
            Some("section>".to_string())
        );
    });
}

#[test]
fn markup_snippets_reject_script_text_duplicates_void_tags_and_bad_carets() {
    crate::runtime::block_on(async {
        let state = ServerState::new();
        let uri = Url::parse("file:///fixture.vue").unwrap();
        for (source, needle, needle_shift, change) in [
            ("<script>const value=1</script>", "=", 0, "="),
            ("<template><input></template>", "<input>", 6, ">"),
            ("<template><div></div></template>", "<div>", 4, ">"),
            ("<template>{{}}</template>", "{{}}", 3, "{"),
        ] {
            let start = source.find(needle).unwrap() + needle_shift;
            let ctx = context(&state, &uri, source.to_string(), start + 1);
            assert_eq!(
                AutoInsertService::snippet(&ctx, start + 1, start, change).await,
                None,
                "{source}"
            );
        }
    });
}

#[test]
fn dot_value_candidate_is_conservative_before_corsa_type_queries() {
    let state = ServerState::new();
    let uri = Url::parse("file:///fixture.vue").unwrap();
    for (line, expected) in [
        ("count", true),
        ("const count", false),
        ("state.count", false),
        ("count.value", false),
        ("watch(count", false),
        ("const item = { count: 1 }", false),
    ] {
        let source = format!("<script setup lang=\"ts\">\n{line}\n</script>");
        let selection = source.find("count").unwrap() + "count".len();
        let ctx = context(&state, &uri, source, selection);
        assert_eq!(
            dot_value_candidate(&ctx, selection, selection - 1, "t"),
            expected,
            "{line}"
        );
    }
}

#[cfg(feature = "native")]
#[test]
fn corsa_ref_classification_uses_only_quick_info_code() {
    use vize_canon::{LspHoverContents, LspMarkedString, LspMarkupContent};

    assert!(corsa::hover_is_ref(LspHoverContents::Markup(
        LspMarkupContent {
            kind: "markdown".to_string(),
            value: "```typescript\nconst count: Ref<number>\n```\nA counter.".to_string(),
        }
    )));
    assert!(!corsa::hover_is_ref(LspHoverContents::Markup(
        LspMarkupContent {
            kind: "markdown".to_string(),
            value: "```typescript\nconst count: number\n```\nReturns a Ref<number>.".to_string(),
        }
    )));
    assert!(!corsa::hover_is_ref(LspHoverContents::String(
        "const count: MyRef<number>".to_string(),
    )));
    assert!(!corsa::hover_is_ref(LspHoverContents::Array(vec![
        LspMarkedString::String("Returns a Ref<number>.".to_string()),
        LspMarkedString::LanguageString {
            language: "typescript".to_string(),
            value: "const count: number".to_string(),
        },
    ])));
}
