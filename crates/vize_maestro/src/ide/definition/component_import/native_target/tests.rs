use super::resolve_native_component_file;
use crate::{
    ide::{DefinitionService, HoverService, IdeContext},
    server::ServerState,
    test_support::native_component::Workspace,
};
use std::sync::Arc;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Hover, HoverContents, Url};

fn markdown(hover: Hover) -> String {
    match hover.contents {
        HoverContents::Markup(content) => content.value,
        other => panic!("expected Markdown, got {other:?}"),
    }
}

fn target(response: GotoDefinitionResponse) -> Url {
    match response {
        GotoDefinitionResponse::Scalar(location) => location.uri,
        other => panic!("expected one authored target, got {other:?}"),
    }
}

#[test]
fn native_selected_package_branch_supplies_tag_definition_and_generic_prop_hover() {
    crate::runtime::block_on(async {
        let Some(runtime) = crate::test_support::tsgo() else {
            return;
        };
        for (custom, selected, prop_type) in
            [(false, "Level11", "number"), (true, "Custom", "string")]
        {
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
            assert_eq!(target(definition).to_file_path().unwrap(), expected);
            let hover = HoverService::hover_with_corsa(&tag, Some(Arc::clone(&bridge)))
                .await
                .unwrap();
            let tag_text = markdown(hover);
            assert!(
                tag_text.contains(vize_l0::cstr!("Vue component: {selected}.vue").as_str()),
                "{tag_text}"
            );
            assert!(!tag_text.contains("Wrong.vue"), "{tag_text}");
            let prop = IdeContext::new(&state, &workspace.uri, source.rfind(":depth").unwrap() + 3)
                .unwrap();
            let hover = HoverService::hover_with_corsa(&prop, Some(Arc::clone(&bridge)))
                .await
                .unwrap();
            let prop_text = markdown(hover);
            assert!(prop_text.contains("depth"), "{prop_text}");
            assert!(prop_text.contains(prop_type), "{prop_text}");
            assert!(!prop_text.contains("boolean"), "{prop_text}");
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
