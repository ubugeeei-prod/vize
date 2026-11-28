use super::resolve_native_component_file;
use crate::{
    ide::{DefinitionService, HoverService, IdeContext},
    server::ServerState,
    test_support::native_component::Workspace,
};
use serde_json::json;
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

const GENERIC_PROP_HOVER: &str = "**depth**\n\n_Component prop_\n\n```typescript\ndepth: number\n```\n\n**Requirement**\n\nRequired\n\n**Example**\n\n```vue\n<Child depth=\"...\" />\n<Child :depth=\"value\" />\n```\n\n**Docs**\n\n[Vue Component Props](https://vuejs.org/guide/components/props.html)";
const CONCRETE_PROP_HOVER: &str = "```typescript\n(property) depth: string\n```";

#[test]
fn native_selected_package_branch_supplies_tag_definition_and_generic_prop_hover() {
    crate::runtime::block_on(async {
        let Some(runtime) = crate::test_support::tsgo() else {
            return;
        };
        for (custom, selected, props, expected_prop) in [
            (
                false,
                "Level11",
                "{ depth: number; items?: readonly T[]; }",
                GENERIC_PROP_HOVER,
            ),
            (true, "Custom", "{ depth: string; }", CONCRETE_PROP_HOVER),
        ] {
            let workspace = Workspace::new(custom);
            let source = Workspace::source("Level11").replace("\n", "\r\n");
            let state = ServerState::new();
            workspace.open(&state, &source, 1);
            let bridge = workspace.bridge(runtime.clone()).await;
            let tag = IdeContext::new(&state, &workspace.uri, source.rfind("<Child").unwrap() + 2)
                .unwrap();
            // The synchronous resolver keeps failing closed on the retained
            // conditional and nonexistent legacy candidates.
            assert!(DefinitionService::definition(&tag).is_none());
            let expected = workspace.target(selected);
            assert_eq!(
                resolve_native_component_file(&tag, "Child", &bridge).await,
                Some(expected.clone())
            );
            let definition =
                DefinitionService::definition_with_corsa(&tag, Some(Arc::clone(&bridge)))
                    .await
                    .unwrap();
            assert_eq!(
                serde_json::to_value(definition).unwrap(),
                json!({
                    "uri": Url::from_file_path(expected).unwrap(),
                    "range": {"start":{"line":0,"character":0},"end":{"line":0,"character":0}}
                })
            );
            let hover = HoverService::hover_with_corsa(&tag, Some(Arc::clone(&bridge)))
                .await
                .unwrap();
            let expected_tag = vize_l0::cstr!(
                "```typescript\nconst Child: VueComponent &\n{{\n  props: {props};\n}}\n```\n\nVue component: {selected}.vue"
            );
            assert_eq!(
                serde_json::to_value(hover).unwrap(),
                json!({
                    "contents":{"kind":"markdown","value":expected_tag.as_str()},
                    "range":{"start":{"line":4,"character":11},"end":{"line":4,"character":16}}
                })
            );
            let prop = IdeContext::new(&state, &workspace.uri, source.rfind(":depth").unwrap() + 3)
                .unwrap();
            let hover = HoverService::hover_with_corsa(&prop, Some(Arc::clone(&bridge)))
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(hover).unwrap(),
                json!({
                    "contents":{"kind":"markdown","value":expected_prop},
                    "range":{"start":{"line":4,"character":18},"end":{"line":4,"character":23}}
                })
            );
            bridge.shutdown().await.unwrap();
        }
    });
}

#[test]
fn native_import_lookup_retains_the_request_snapshot_across_same_length_edits() {
    crate::runtime::block_on(async {
        let Some(runtime) = crate::test_support::tsgo() else {
            return;
        };
        let workspace = Workspace::new(false);
        let source = Workspace::source("Alpha");
        let changed = Workspace::source("Bravo");
        assert_eq!(source.len(), changed.len());
        let state = ServerState::new();
        workspace.open(&state, &source, 1);
        let bridge = workspace.bridge(runtime).await;
        let old =
            IdeContext::new(&state, &workspace.uri, source.rfind("<Child").unwrap() + 2).unwrap();
        assert_eq!(
            resolve_native_component_file(&old, "Child", &bridge).await,
            Some(workspace.target("Alpha"))
        );
        workspace.open(&state, &changed, 2);
        let captured = resolve_native_component_file(&old, "Child", &bridge).await;
        assert!(
            captured.is_none() || captured == Some(workspace.target("Alpha")),
            "{captured:?}"
        );
        let current =
            IdeContext::new(&state, &workspace.uri, changed.rfind("<Child").unwrap() + 2).unwrap();
        assert_eq!(
            resolve_native_component_file(&current, "Child", &bridge).await,
            Some(workspace.target("Bravo"))
        );
        bridge.shutdown().await.unwrap();
    });
}
