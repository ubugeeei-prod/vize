//! Component tags show the definition's contract, with the same typed
//! presentation used for an imported SFC identifier.

use tower_lsp::lsp_types::Hover;

use super::HoverService;
#[cfg(feature = "native")]
use super::{HoverBuilder, component_import::component_contract_markdown};
use crate::ide::IdeContext;
#[cfg(feature = "native")]
use crate::ide::{definition::helpers, is_component_tag, kebab_to_pascal};

impl HoverService {
    pub(super) fn hover_component_tag(ctx: &IdeContext<'_>) -> Option<Hover> {
        #[cfg(feature = "native")]
        {
            let tag = helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
            if !is_component_tag(&tag) {
                return None;
            }
            let name = if tag.contains('-') {
                kebab_to_pascal(&tag)
            } else {
                tag
            };
            let contract = component_contract_markdown(ctx, &name)?;
            Some(HoverBuilder::new().description(&contract).build())
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = ctx;
            None
        }
    }
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::HoverService;
    use crate::{ide::IdeContext, server::ServerState};
    use std::fs;
    use tower_lsp::lsp_types::{HoverContents, MarkupKind, Url};

    const CHILD: &str =
        include_str!("../../../tests/fixtures/component-definition-hover/Child.vue");
    const PARENT: &str =
        include_str!("../../../tests/fixtures/component-definition-hover/Parent.vue");

    fn markdown(parent: &str, unsaved_child: Option<&str>) -> std::io::Result<String> {
        let directory = tempfile::tempdir()?;
        let child_path = directory.path().join("Child.vue");
        fs::write(&child_path, CHILD)?;
        fs::write(
            directory.path().join("index.ts"),
            "export { default as Child } from './Child.vue';",
        )?;
        let parent_path = directory.path().join("Parent.vue");
        fs::write(&parent_path, parent)?;
        let uri = Url::from_file_path(&parent_path)
            .map_err(|()| std::io::Error::other("parent fixture URI"))?;
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), parent.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, parent);
        if let Some(child) = unsaved_child {
            let child_uri = Url::from_file_path(&child_path)
                .map_err(|()| std::io::Error::other("child fixture URI"))?;
            state
                .documents
                .open(child_uri.clone(), child.to_string(), 2, "vue".to_string());
            state.update_virtual_docs(&child_uri, child);
        }
        let offset = parent
            .find("<Child")
            .or_else(|| parent.find("<alias-child"))
            .ok_or_else(|| std::io::Error::other("component tag in parent fixture"))?
            + 2;
        let ctx = IdeContext::new(&state, &uri, offset)
            .ok_or_else(|| std::io::Error::other("fixture IDE context"))?;
        let hover = HoverService::hover(&ctx)
            .ok_or_else(|| std::io::Error::other("component contract hover"))?;
        let HoverContents::Markup(content) = hover.contents else {
            return Err(std::io::Error::other("expected Markdown contract"));
        };
        if content.kind != MarkupKind::Markdown {
            return Err(std::io::Error::other("expected Markdown kind"));
        }
        Ok(content.value)
    }

    #[test]
    fn tag_hover_shows_declared_typed_props_emits_slots_and_models() {
        let value = markdown(PARENT, None).expect("component definition hover");
        for expected in [
            "```typescript",
            "const Child: VueComponent",
            "props:",
            "message: string",
            "count?: number",
            "emits:",
            "save: [value: number]",
            "slots:",
            "item(props: { row: string; index: number })",
            "model:",
            "\"title\": string",
        ] {
            assert!(value.contains(expected), "missing {expected}: {value}");
        }
        for unwanted in [
            "Passed props",
            "Component usage",
            "Listeners",
            "Required props not passed",
            "usage-only",
            "#provided",
        ] {
            assert!(
                !value.contains(unwanted),
                "usage leaked into definition hover: {value}"
            );
        }
    }

    #[test]
    fn tag_hover_reads_unsaved_definition_changes() {
        let changed = CHILD
            .replace("count?: number", "count?: boolean")
            .replace("row: string", "row: number");
        let value = markdown(PARENT, Some(&changed)).expect("unsaved component definition hover");
        assert!(value.contains("count?: boolean"), "{value}");
        assert!(value.contains("row: number"), "{value}");
        assert!(!value.contains("count?: number"), "{value}");
    }

    #[test]
    fn kebab_tag_hover_resolves_aliased_barrel_definition() {
        let parent = PARENT.replace(
            "import Child from './Child.vue'",
            "import { Child as AliasChild } from './index'",
        );
        let parent = parent
            .replace("<Child", "<alias-child")
            .replace("</Child>", "</alias-child>");
        let value = markdown(&parent, None).expect("aliased component definition hover");
        assert!(value.contains("const AliasChild: VueComponent"), "{value}");
        assert!(value.contains("slots:"), "{value}");
    }

    #[test]
    fn runtime_contract_names_remain_valid_typescript() {
        let child =
            include_str!("../../../tests/fixtures/component-definition-hover/RuntimeChild.vue");
        let value = markdown(PARENT, Some(child)).expect("runtime component definition hover");
        for expected in [
            r#""font-size": number"#,
            r#""update:title": unknown[]"#,
            "'header-title'(props: { value: string })",
            r#""quoted\"model": boolean"#,
        ] {
            assert!(value.contains(expected), "missing {expected}: {value}");
        }
    }
}
