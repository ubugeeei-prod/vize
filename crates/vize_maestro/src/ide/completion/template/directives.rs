//! Vue / petite-vue / Vize directive completions.

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};

use crate::ide::IdeContext;
use crate::ide::completion::items;

/// Vue directive completions.
pub(crate) fn directive_completions() -> Vec<CompletionItem> {
    vec![
        items::directive_item("v-if", "Conditional rendering", "v-if=\"$1\""),
        items::directive_item("v-else-if", "Else-if block", "v-else-if=\"$1\""),
        items::directive_item("v-else", "Else block", "v-else"),
        items::directive_item("v-for", "List rendering", "v-for=\"$1 in $2\" :key=\"$3\""),
        items::directive_item("v-on", "Event listener", "v-on:$1=\"$2\""),
        items::directive_item("v-bind", "Attribute binding", "v-bind:$1=\"$2\""),
        items::directive_item("v-model", "Two-way binding", "v-model=\"$1\""),
        items::directive_item("v-slot", "Named slot", "v-slot:$1"),
        items::directive_item("v-show", "Toggle visibility", "v-show=\"$1\""),
        items::directive_item("v-pre", "Skip compilation", "v-pre"),
        items::directive_item("v-once", "Render once", "v-once"),
        items::directive_item("v-memo", "Memoize subtree", "v-memo=\"[$1]\""),
        items::directive_item("v-cloak", "Hide until compiled", "v-cloak"),
        items::directive_item("v-text", "Set text content", "v-text=\"$1\""),
        items::directive_item("v-html", "Set innerHTML", "v-html=\"$1\""),
        items::directive_item("@", "Event shorthand", "@$1=\"$2\""),
        items::directive_item(":", "Bind shorthand", ":$1=\"$2\""),
        items::directive_item("#", "Slot shorthand", "#$1"),
    ]
}

/// Vue directive completions, extended with opt-in document-specific directives.
pub(crate) fn contextual_directive_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    let mut completions = directive_completions();
    if ctx.dialect().is_petite_vue() {
        completions.extend(petite_vue_directive_completions());
    }
    completions
}

/// petite-vue directive and lifecycle event completions.
pub(crate) fn petite_vue_directive_completions() -> Vec<CompletionItem> {
    vec![
        petite_vue_item(
            "v-scope",
            "petite-vue scope root",
            "v-scope=\"{ $1 }\"",
            "Marks an HTML region controlled by petite-vue.",
        ),
        petite_vue_item(
            "v-effect",
            "Reactive inline effect",
            "v-effect=\"$1\"",
            "Runs reactive inline statements when referenced state changes.",
        ),
        petite_vue_item(
            "@vue:mounted",
            "petite-vue mounted event",
            "@vue:mounted=\"$1\"",
            "Listens for the petite-vue mounted lifecycle event.",
        ),
        petite_vue_item(
            "@vue:unmounted",
            "petite-vue unmounted event",
            "@vue:unmounted=\"$1\"",
            "Listens for the petite-vue unmounted lifecycle event.",
        ),
    ]
}

#[allow(clippy::disallowed_macros)]
fn petite_vue_item(
    label: &str,
    detail: &str,
    snippet: &str,
    documentation: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(detail.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**{}**\n\n{}\n\n[petite-vue](https://github.com/vuejs/petite-vue)",
                label, documentation
            ),
        })),
        ..Default::default()
    }
}

/// Vize directive completions for use inside HTML comments.
pub(crate) fn vize_directive_completions() -> Vec<CompletionItem> {
    vec![
        items::vize_directive_item(
            "@vize:todo",
            "@vize:todo $1 ",
            "TODO marker (warning in linter, stripped from build)",
        ),
        items::vize_directive_item(
            "@vize:fixme",
            "@vize:fixme $1 ",
            "FIXME marker (error in linter, stripped from build)",
        ),
        items::vize_directive_item(
            "@vize:expected",
            "@vize:expected",
            "Expect error on next line",
        ),
        items::vize_directive_item(
            "@vize:docs",
            "@vize:docs $1 ",
            "Documentation comment (stripped from build)",
        ),
        items::vize_directive_item(
            "@vize:ignore-start",
            "@vize:ignore-start",
            "Begin lint suppression region",
        ),
        items::vize_directive_item(
            "@vize:ignore-end",
            "@vize:ignore-end",
            "End lint suppression region",
        ),
        items::vize_directive_item(
            "@vize:level(warn)",
            "@vize:level($1)",
            "Override next-line diagnostic severity",
        ),
        items::vize_directive_item(
            "@vize:deprecated",
            "@vize:deprecated $1 ",
            "Deprecation warning",
        ),
        items::vize_directive_item("@vize:dev-only", "@vize:dev-only", "Strip in production"),
    ]
}
