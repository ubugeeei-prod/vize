use super::{contains_cursor, quoted_token, source_type};
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::Url;

mod cost;

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/string-literal-completion-original/Page.vue.txt"
));

fn at(source: &str, needle: &str) -> usize {
    source.find(needle).expect("unique cursor") + needle.len()
}

fn context(source: &str, offset: usize) -> bool {
    let state = ServerState::new();
    let uri = Url::parse("file:///Page.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, source);
    contains_cursor(&IdeContext::at_completion(&state, &uri, offset).unwrap())
}

#[test]
fn original_script_and_template_quotes_own_both_reported_cursors() {
    for (line, character, needle) in [(3, 17, "t(\""), (8, 11, "<p>{{ t(\"")] {
        let offset = ORIGINAL
            .lines()
            .take(line)
            .map(|line| line.len() + 1)
            .sum::<usize>()
            + character;
        assert_eq!(offset, at(ORIGINAL, needle));
        assert!(context(ORIGINAL, offset), "{line}:{character}");
    }
}

#[test]
fn lexer_distinguishes_full_literals_from_comments_regex_and_template_code() {
    for (source, needle, expected) in [
        ("t(\"form.help\")", "t(\"", true),
        ("t('form.help')", "t('", true),
        ("t(\"雪🌸form.help\")", "雪🌸", true),
        ("t(\"don't\")", "don't", true),
        (r#"t("escaped\"quote")"#, r#"escaped\""#, true),
        ("t(\"line\\\ncontinued\")", "continued", true),
        ("t(\"line\\\rcontinued\")", "continued", true),
        ("t(\"line\\\r\ncontinued\")", "continued", true),
        ("t('line\\\rcontinued')", "continued", true),
        ("// t(\"line\\\rcontinued\")", "continued", false),
        ("/* t(\"line\\\rcontinued\") */", "continued", false),
        ("const rx = /line\\\rcontinued/;", "continued", false),
        ("t(`line\ncontinued`)", "continued", true),
        ("t(\"unfinished", "unfinished", true),
        ("t('unfinished", "unfinished", true),
        ("t(`unfinished", "unfinished", true),
        ("// t(\"unfinished", "unfinished", false),
        ("/* t('unfinished", "unfinished", false),
        ("const rx = /[\"']unfinished", "unfinished", false),
        ("// t(\"form.help\")\nconst title = 1", "t(\"", false),
        ("/* t('form.help') */ const title = 1", "t('", false),
        (r#"const rx = /["']form/;"#, "form", false),
        ("t(`prefix-${title}`)", "title", false),
        ("t(\"form.help\"); title", "title", false),
        ("t(\"form.help\")", "t(\"form.help\"", false),
        ("const title = 1", "title", false),
    ] {
        let span = quoted_token(source, at(source, needle), source_type(Some("ts")));
        assert_eq!(span.is_some(), expected, "{source:?}/{needle:?}");
        if let Some(span) = span {
            assert!(source.get(span).is_some(), "whole UTF8 token");
        }
    }
    assert!(quoted_token("t('雪')", 4, source_type(Some("ts"))).is_none());
}

#[test]
fn source_domains_do_not_mistake_markup_or_regex_quotes_for_arguments() {
    for (source, needle, expected) in [
        ("<template><p>{{ t('help') }}</p></template>", "t('", true),
        (
            "<template><p :title=\"t('help')\" /></template>",
            "t('",
            true,
        ),
        (
            "<template><p title=\"t('help')\" /></template>",
            "t('",
            false,
        ),
        ("<template><p v-if=\"t('help')\" /></template>", "t('", true),
        (
            "<template><p @click=\"t('help')\" /></template>",
            "t('",
            true,
        ),
        (
            "<template><!-- {{ t('help') }} --></template>",
            "t('",
            false,
        ),
        (
            "<template><p>{{ /[\"']/.test(title) }}</p></template>",
            "[\"",
            false,
        ),
        (
            "<template><p>{{ `text-${title}` }}</p></template>",
            "title",
            false,
        ),
        (
            "<script>const t = (key) => key; t('help')</script>",
            "t('",
            true,
        ),
        (
            "<script setup>const t = (key) => key; t('help')</script>",
            "t('",
            true,
        ),
    ] {
        assert_eq!(context(source, at(source, needle)), expected, "{source}");
    }
    let crlf = ORIGINAL
        .replace('\n', "\r\n")
        .replace("form.help", "雪🌸form.help");
    assert!(context(&crlf, at(&crlf, "雪🌸")));
    let bare_cr = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/string-literal-completion-original/bare-cr-continuation.vue.txt"
    ));
    assert!(context(bare_cr, at(bare_cr, "form.\\\rna")));
    assert!(context(bare_cr, at(bare_cr, "<p>{{ t(\"")));
}

#[test]
fn unprovided_native_values_never_fabricate_identifier_extras() {
    for needle in ["t(\"", "<p>{{ t(\""] {
        let state = ServerState::new();
        let uri = Url::parse("file:///Page.vue").unwrap();
        state
            .documents
            .open(uri.clone(), ORIGINAL.into(), 1, "vue".into());
        state.update_virtual_docs(&uri, ORIGINAL);
        let ctx = IdeContext::at_completion(&state, &uri, at(ORIGINAL, needle)).unwrap();
        assert!(super::super::CompletionService::complete(&ctx).is_none());
    }
}
