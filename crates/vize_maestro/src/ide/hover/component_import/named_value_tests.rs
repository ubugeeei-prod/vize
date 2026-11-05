//! SFC default exports and ordinary named values have distinct hover owners.

use std::fs;

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range, Url};

use super::{component_contract_inline, vue_component_import_hover};
use crate::ide::IdeContext;
use crate::ide::definition::component_import::resolve_component_value_file;
use crate::server::ServerState;

const CHILD: &str = "<script lang=\"ts\">export const childValue: number = 1;</script>\n<script setup lang=\"ts\">defineProps<{ title: string }>(); defineEmits<{ close: [] }>();</script>\n<template><h2>{{ title }}</h2></template>\n";
const INLINE: &str = "VueComponent & { props: { title: string }; emits: { close: [] } }";
const MARKDOWN: &str = "```typescript\nconst Subject: VueComponent &\n{\n  props: { title: string };\n  emits: { close: [] };\n}\n```\n\nVue component: Child.vue";

#[test]
fn named_sfc_values_never_claim_the_default_component_contract() {
    for script in [
        "import { childValue as Subject } from './Child.vue';",
        "import Child, { childValue as Subject } from './Child.vue';",
        "import { childValue } from './Child.vue'; const Subject = childValue;",
        "import { childValue } from './Child.vue'; const Subject = (childValue);",
        "import { childValue as Subject } from './barrel';",
        "import * as Subject from './Child.vue';",
    ] {
        with_subject(script, |ctx, _| {
            assert_eq!(
                (
                    resolve_component_value_file(ctx, "Subject"),
                    component_contract_inline(ctx, "Subject"),
                    vue_component_import_hover(ctx, "Subject"),
                ),
                (None, None, None),
                "{script}"
            );
        });
    }
}

#[test]
fn default_imports_and_component_aliases_retain_the_whole_authored_contract() {
    for script in [
        "import Subject from './Child.vue';",
        "import { default as Subject } from './Child.vue';",
        "import Child, { default as Subject, childValue } from './Child.vue';",
        "import { Child as Subject } from './barrel';",
        "import Child from './Child.vue'; const Subject = Child;",
        "import { defineAsyncComponent } from 'vue'; const Subject = defineAsyncComponent(() => import('./Child.vue'));",
    ] {
        with_subject(script, |ctx, child| {
            let (line, character) = crate::ide::offset_to_position(&ctx.content, ctx.offset);
            let expected = Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: MARKDOWN.to_owned(),
                }),
                range: Some(Range::new(
                    Position::new(line, character),
                    Position::new(line, character + "Subject".len() as u32),
                )),
            };
            assert_eq!(
                (
                    resolve_component_value_file(ctx, "Subject")
                        .map(|path| path.canonicalize().unwrap()),
                    component_contract_inline(ctx, "Subject"),
                    vue_component_import_hover(ctx, "Subject"),
                ),
                (
                    Some(child.canonicalize().unwrap()),
                    Some(INLINE.to_owned()),
                    Some(expected),
                ),
                "{script}"
            );
        });
    }
}

fn with_subject(script: &str, check: impl FnOnce(&IdeContext<'_>, &std::path::Path)) {
    let directory = crate::test_support::tempdir().unwrap();
    let child = directory.path().join("Child.vue");
    fs::write(&child, CHILD).unwrap();
    fs::write(
        directory.path().join("barrel.ts"),
        "export { default as Child, childValue } from './Child.vue';",
    )
    .unwrap();
    let source = format!(
        "<script setup lang=\"ts\">\n{script}\n</script>\n<template><Subject title=\"ok\" /></template>\n"
    );
    let offset = source.rfind("Subject").unwrap();
    let path = directory.path().join("App.vue");
    fs::write(&path, &source).unwrap();
    let uri = Url::from_file_path(path).unwrap();
    let state = ServerState::new();
    let ctx = IdeContext::testing(&state, &uri, offset, source);
    check(&ctx, &child);
}
