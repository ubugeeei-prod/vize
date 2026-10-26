//! Constants that hold an SFC present the authored contract (#10787).

use std::fs;

use tower_lsp::lsp_types::{Hover, HoverContents, Url};

use super::{component_contract_inline, vue_component_import_hover};
use crate::ide::IdeContext;
use crate::ide::hover::HoverService;
use crate::server::ServerState;

const CHILD: &str = "<script setup lang=\"ts\">\ndefineProps<{ title: string }>();\ndefineEmits<{ close: [] }>();\n</script>\n\n<template>\n  <h2>{{ title }}</h2>\n</template>\n";

const CONTRACT: &str = ": VueComponent &\n{\n  props: { title: string };\n  emits: { close: [] };\n}\n```\n\nVue component: MyChild.vue";

struct Fixture {
    _directory: tempfile::TempDir,
    state: ServerState,
    uri: Url,
    source: String,
}

impl Fixture {
    fn new(script: &str) -> Self {
        let directory = crate::test_support::tempdir().unwrap();
        fs::write(directory.path().join("MyChild.vue"), CHILD).unwrap();
        fs::write(
            directory.path().join("index.ts"),
            "export { default as Child } from './MyChild.vue';",
        )
        .unwrap();
        let source = format!(
            "<script setup lang=\"ts\">\n{script}\n</script>\n\n<template>\n  <LazyChild title=\"a\" />\n</template>\n"
        );
        let path = directory.path().join("App.vue");
        fs::write(&path, &source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);
        Self {
            _directory: directory,
            state,
            uri,
            source,
        }
    }

    fn ctx(&self, needle: &str) -> IdeContext<'_> {
        let offset = self.source.find(needle).unwrap() + 1;
        IdeContext::new(&self.state, &self.uri, offset).unwrap()
    }

    fn declaration(&self, name: &str) -> IdeContext<'_> {
        let offset = self.source.find(name).unwrap() + 1;
        IdeContext::new(&self.state, &self.uri, offset).unwrap()
    }

    fn script_hover(&self, name: &str) -> Option<String> {
        let ctx = self.declaration(name);
        futures::executor::block_on(HoverService::hover_script_with_corsa(&ctx, true, None))
            .and_then(markdown)
    }
}

fn markdown(hover: Hover) -> Option<String> {
    match hover.contents {
        HoverContents::Markup(content) => Some(content.value),
        _ => None,
    }
}

#[test]
fn alias_and_async_component_constants_show_the_import_contract() {
    for (script, name) in [
        (
            "import MyChild from './MyChild.vue';\nconst LazyChild = MyChild;",
            "LazyChild",
        ),
        (
            "import { Child } from './index';\nconst Same = Child;\nconst LazyChild = (Same);",
            "LazyChild",
        ),
        (
            "import { defineAsyncComponent } from 'vue';\nconst LazyChild = defineAsyncComponent(() => import('./MyChild.vue'));",
            "LazyChild",
        ),
        (
            "const LazyChild = defineAsyncComponent({ loader: () => import(\"./MyChild.vue\"), delay: 0 });",
            "LazyChild",
        ),
    ] {
        let fixture = Fixture::new(script);
        assert_eq!(
            fixture.script_hover(name),
            Some(format!("```typescript\nconst {name}{CONTRACT}")),
            "{script}"
        );
        assert_eq!(
            HoverService::hover(&fixture.ctx("<LazyChild")).and_then(markdown),
            Some(format!("```typescript\nconst LazyChild{CONTRACT}")),
            "{script}"
        );
        assert_eq!(
            component_contract_inline(&fixture.ctx("LazyChild"), name).as_deref(),
            Some("VueComponent & { props: { title: string }; emits: { close: [] } }")
        );
    }
}

#[test]
fn reassignable_annotated_or_foreign_values_keep_their_own_type() {
    for script in [
        "import MyChild from './MyChild.vue';\nlet LazyChild = MyChild;",
        "import type { Component } from 'vue';\nimport MyChild from './MyChild.vue';\nconst LazyChild: Component = MyChild;",
        "import { defineAsyncComponent } from './local';\nconst LazyChild = defineAsyncComponent(() => import('./MyChild.vue'));",
        "const LazyChild = LazyChild;",
    ] {
        let fixture = Fixture::new(script);
        let ctx = fixture.declaration("LazyChild");
        assert_eq!(
            vue_component_import_hover(&ctx, "LazyChild"),
            None,
            "{script}"
        );
        assert_eq!(
            component_contract_inline(&fixture.ctx("LazyChild"), "LazyChild"),
            None,
            "{script}"
        );
    }
}
