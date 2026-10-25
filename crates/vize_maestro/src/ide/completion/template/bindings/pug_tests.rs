use super::analyzed_template_binding_completions;
use crate::ide::{IdeContext, template_scope::pug};
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;

#[test]
fn unmapped_and_refused_pug_preserve_script_bindings_without_admitting_locals() {
    for language in ["pug", "jade"] {
        for newline in ["\n", "\r\n"] {
            for (body, marker, admitted) in [
                (
                    "div(v-for=\"(localOnly, indexOnly) in rows\")\n  p {{ outer }}",
                    "  p",
                    true,
                ),
                (
                    "mixin refused(localOnly)\n  p {{ outer }}",
                    "outer }}",
                    false,
                ),
            ] {
                let source = vize_l0::cstr!("<script setup lang=\"ts\">const outer = 1; const rows = [];</script>\n<template lang=\"{language}\">\n{body}\n</template>").replace('\n', newline);
                let state = ServerState::new();
                let uri = Url::parse("file:///tmp/Fallback.vue").unwrap();
                let at = source.find(marker).unwrap();
                let ctx = IdeContext::testing_completion(&state, &uri, at, source);
                assert!(pug::analyzed_for_completion(&ctx).is_none());
                assert_eq!(pug::cached_analysis(&ctx).is_some(), admitted);
                let mut labels: Vec<_> = analyzed_template_binding_completions(&ctx, true)
                    .into_iter()
                    .map(|item| item.label)
                    .filter(|label| {
                        ["outer", "rows", "localOnly", "indexOnly"].contains(&label.as_str())
                    })
                    .collect();
                labels.sort_unstable();
                assert_eq!(labels, ["outer", "rows"]);
            }
        }
    }
}
