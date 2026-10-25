use super::*;
use crate::ide::template_scope;
use crate::server::ServerState;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

const SOURCE: &str = "<script setup lang=\"ts\">\nconst item = 'outside'; const rows = [];\n</script>\n<template lang=\"pug\">\nsection\n  div(v-for=\"(item, outerIndex) in rows\")\n    p 🧭{{ item }}\n    Child(v-slot=\"{ item, slotOnly }\")\n      p {{ item }}\n      em(v-for=\"(item, innerIndex) in rows\") {{ item }}\n  div(v-for=\"(item, siblingOnly) in rows\") {{ item }}\n  p {{ item }}\n</template>\n";

#[test]
fn completion_at_a_quoted_value_end_keeps_the_enclosing_scope() {
    for newline in ["\n", "\r\n"] {
        let source = "<script setup>const item = 'outer'; const rows = [];</script><template lang=\"pug\">\ndiv(v-for=\"(item, outerIndex) in rows\")\n  p(:title=\"item\")\n</template>".replace('\n', newline);
        let state = ServerState::new();
        let uri = Url::parse("file:///tmp/Caret.vue").unwrap();
        let offset = source.find(":title=\"item\"").unwrap() + ":title=\"item".len();
        let ctx = IdeContext::testing_completion(&state, &uri, offset, source);
        assert!(
            analyzed(&ctx).is_none(),
            "the quote is not an authored identifier byte"
        );
        let (croquis, cursor) = analyzed_for_completion(&ctx).unwrap();
        let mut names: Vec<_> = template_scope::bindings_visible_at(croquis, cursor)
            .into_iter()
            .filter(|(_, _, kind)| *kind == ScopeKind::VFor)
            .map(|(name, _, _)| name)
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["item", "outerIndex"]);
    }
}

#[test]
fn mapped_scopes_keep_nearest_declarations_and_complete_local_names() {
    for language in ["pug", "jade", "PUG", "JADE"] {
        for newline in ["\n", "\r\n"] {
            let source = SOURCE
                .replace("lang=\"pug\"", &vize_l0::cstr!("lang=\"{language}\""))
                .replace('\n', newline);
            let state = ServerState::new();
            let uri = Url::parse("file:///tmp/Scopes.vue").unwrap();
            for (usage, declaration, kind, locals) in [
                (
                    "🧭{{ item",
                    "(item, outerIndex)",
                    TemplateScopeBindingKind::Value,
                    vec!["item", "outerIndex"],
                ),
                (
                    "p {{ item",
                    "{ item, slotOnly }",
                    TemplateScopeBindingKind::SlotProp,
                    vec!["item", "outerIndex", "slotOnly"],
                ),
                (
                    "rows\") {{ item",
                    "(item, innerIndex)",
                    TemplateScopeBindingKind::Value,
                    vec!["innerIndex", "item", "outerIndex", "slotOnly"],
                ),
            ] {
                let usage_start = source.find(usage).unwrap();
                let offset = source.get(usage_start..).unwrap().find("item").unwrap() + usage_start;
                let declaration = source.find(declaration).unwrap()
                    + if kind == TemplateScopeBindingKind::SlotProp {
                        2
                    } else {
                        1
                    };
                let ctx = IdeContext::testing(&state, &uri, offset, source.clone());
                let binding = TemplateScopeBinding {
                    name: "item".into(),
                    start: declaration,
                    end: declaration + 4,
                    kind,
                };
                assert_eq!(binding_at(&ctx, "item"), Some(binding.clone()));
                assert_eq!(
                    template_scope::definition(&ctx, "item"),
                    Some(GotoDefinitionResponse::Scalar(binding.location(&ctx)))
                );
                let (croquis, cursor) = analyzed(&ctx).unwrap();
                let mut actual: Vec<_> = template_scope::bindings_visible_at(croquis, cursor)
                    .into_iter()
                    .filter(|(_, _, kind)| matches!(kind, ScopeKind::VFor | ScopeKind::VSlot))
                    .map(|(name, _, _)| name)
                    .collect();
                actual.sort_unstable();
                assert_eq!(actual, locals);
                let first = scope(&ctx).unwrap();
                assert!(
                    std::ptr::eq(first, scope(&ctx).unwrap()),
                    "one derivation and analysis per request"
                );
            }
            let offset = source.rfind("{{ item").unwrap() + 3;
            let ctx = IdeContext::testing(&state, &uri, offset, source.clone());
            assert_eq!(
                binding_at(&ctx, "item"),
                None,
                "dedented sibling cannot see prior loops"
            );
        }
    }
}

#[test]
fn multiline_attributes_and_encoded_preceding_values_keep_exact_identifiers() {
    for newline in ["\n", "\r\n"] {
        for quote in ['\'', '"'] {
            let source = vize_l0::cstr!("<script setup>const 条目 = 0; const rows = [];</script>\n<template lang=\"jade\">\nli(\n  :title={quote}a < b && c{quote}\n  v-for={quote}(条目, index) in rows{quote}\n) 🧭{{{{ 条目 }}}}\n</template>").replace('\n', newline);
            let state = ServerState::new();
            let uri = Url::parse("file:///tmp/Multiline.vue").unwrap();
            let offset = source.rfind("条目").unwrap();
            let ctx = IdeContext::testing(&state, &uri, offset, source.clone());
            let declaration = source.find("(条目, index)").unwrap() + 1;
            assert_eq!(
                binding_at(&ctx, "条目"),
                Some(TemplateScopeBinding {
                    name: "条目".into(),
                    start: declaration,
                    end: declaration + "条目".len(),
                    kind: TemplateScopeBindingKind::Value
                })
            );
        }
    }
}

#[test]
fn iterable_expression_does_not_see_its_own_alias_and_refused_pug_has_no_scope() {
    for body in [
        "li(v-for=\"item in item\") {{ item }}",
        "mixin broken(item)\n  li {{ item }}",
    ] {
        let source = vize_l0::cstr!(
            "<script setup>const item = [];</script><template lang=\"pug\">\n{body}\n</template>"
        );
        let state = ServerState::new();
        let uri = Url::parse("file:///tmp/Boundary.vue").unwrap();
        let offset = source
            .find("in item")
            .map_or_else(|| source.rfind("{{ item").unwrap() + 3, |start| start + 3);
        let ctx = IdeContext::testing(&state, &uri, offset, source.into());
        assert_eq!(binding_at(&ctx, "item"), None);
    }
}
