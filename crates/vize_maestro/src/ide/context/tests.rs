use super::IdeContext;
use crate::server::ServerState;
use crate::virtual_code::BlockType;
use tower_lsp::lsp_types::Url;

const CONTEXT_SFC: &str = "<script setup>\r\nconst count = 1\r\n</script>\r\n<template><div title=\"雪😀\"  class=\"a\">{{ count }}</div></template>";

fn context_responses(ctx: &IdeContext<'_>) -> serde_json::Value {
    use crate::ide::{CodeActionService, TypeService, ecosystem, offset_to_position};
    use tower_lsp::lsp_types::{Position, Range};

    let gap = ctx.content.find("  class").unwrap_or(0);
    let (line, character) = offset_to_position(&ctx.content, gap);
    let position = Position::new(line, character);
    let completions = TypeService::get_completions(ctx);
    serde_json::json!({
        "type": TypeService::get_type_at(ctx).map(|info| info.display.to_string()),
        "completion": completions.iter().find(|item| item.label == "count")
            .and_then(|item| item.detail.as_ref()).map(|detail| detail.to_string()),
        "actions": CodeActionService::code_actions(ctx, Range::new(position, position)),
        "fixes": CodeActionService::get_all_fixes(ctx),
        "ecosystem": ecosystem::completions(ctx),
    })
}

fn context_at_count<'a>(state: &'a ServerState, uri: &'a Url, text: &str) -> IdeContext<'a> {
    let offset = text
        .rfind("count")
        .map_or(0, |start| start + "count".len() - 1);
    IdeContext::with_content(state, uri, offset, text.into())
}

#[test]
fn resident_context_consumers_keep_typed_results_and_authored_edits_after_changes() {
    use vize_resident::DescriptorStats;

    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Counter.vue").unwrap();
    let original = context_at_count(&state, &uri, CONTEXT_SFC);
    let first = context_responses(&original);
    assert_eq!(first["type"], "number");
    assert_eq!(first["completion"], "number");
    assert_eq!(first["actions"].as_array().unwrap().len(), 2);
    assert_eq!(
        first["fixes"]["changes"][uri.as_str()]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    let warm = context_at_count(&state, &uri, CONTEXT_SFC);
    assert_eq!(context_responses(&warm), first);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 0
        }
    );

    let edited = CONTEXT_SFC
        .replace("= 1", "= 'hello'")
        .replace("<template>", "\r\n<template>");
    let updated = context_at_count(&state, &uri, &edited);
    let changed = context_responses(&updated);
    assert_eq!(changed["type"], "string");
    assert_eq!(changed["completion"], "string");
    assert_ne!(changed["fixes"], first["fixes"]);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    let clean = ServerState::new();
    assert_eq!(
        changed,
        context_responses(&context_at_count(&clean, &uri, &edited))
    );
    assert_eq!(
        context_responses(&original),
        first,
        "in-flight contexts retain their own revision"
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 0,
            parses: 0
        }
    );
}

#[test]
fn resident_context_consumers_preserve_spaced_declarations_and_annotations() {
    use crate::ide::TypeService;

    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Counter.vue").unwrap();
    for (declaration, expected) in [
        ("const   count = 1", "number"),
        ("let  count = true", "boolean"),
        ("var   count: string = 'hello'", "string"),
    ] {
        let source = CONTEXT_SFC.replace("const count = 1", declaration);
        let ctx = context_at_count(&state, &uri, &source);
        assert_eq!(TypeService::get_type_at(&ctx).unwrap().display, expected);
        assert_eq!(context_responses(&ctx)["completion"], expected);
    }
}

#[test]
fn resident_context_consumers_share_parse_rejection_and_recovery() {
    use vize_resident::DescriptorStats;

    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Counter.vue").unwrap();
    for parses in [1, 0] {
        let broken = context_at_count(&state, &uri, "<template>{{count}}");
        assert_eq!(
            context_responses(&broken),
            serde_json::json!({
                "type": null, "completion": null, "actions": [], "fixes": null, "ecosystem": [],
            })
        );
        assert_eq!(
            state.resident.take_stats(),
            DescriptorStats { lookups: 1, parses }
        );
    }
    let fixed = context_responses(&context_at_count(&state, &uri, CONTEXT_SFC));
    assert_eq!(fixed["type"], "number");
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    assert_eq!(
        context_responses(&context_at_count(&state, &uri, CONTEXT_SFC)),
        fixed
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
}

#[test]
fn script_insertion_points_do_not_change_byte_or_neighbor_classification() {
    let state = ServerState::new();
    for uri in [
        "file:///workspace/App.vue",
        "file:///workspace/Example.art.vue",
    ] {
        let uri = Url::parse(uri).unwrap();
        for (opening, expected) in [
            ("<script lang=\"ts\">", BlockType::Script),
            ("<script setup lang=\"ts\">", BlockType::ScriptSetup),
        ] {
            for body in ["", "\r\n", "const label = '\u{96ea}\u{1f600}';"] {
                let source = [opening, body, "</script>"].concat();
                let end = opening.len() + body.len();
                let point = IdeContext::with_content(&state, &uri, end, source.clone());
                let insertion =
                    IdeContext::with_content_for_completion(&state, &uri, end, source.clone());
                assert_eq!(point.block_type, None);
                assert_eq!(insertion.block_type, Some(expected));
                assert_eq!(insertion.offset, end);
                assert_eq!(insertion.content, source);
                for offset in [opening.len() - 1, end + 1, source.len()] {
                    let point = IdeContext::with_content(&state, &uri, offset, source.clone());
                    let insertion = IdeContext::with_content_for_completion(
                        &state,
                        &uri,
                        offset,
                        source.clone(),
                    );
                    assert_eq!(point.block_type, None);
                    assert_eq!(insertion.block_type, None);
                }
                if !body.is_empty() {
                    let offset = opening.len();
                    let point = IdeContext::with_content(&state, &uri, offset, source.clone());
                    let insertion = IdeContext::with_content_for_completion(
                        &state,
                        &uri,
                        offset,
                        source.clone(),
                    );
                    assert_eq!(point.block_type, Some(expected));
                    assert_eq!(insertion.block_type, Some(expected));
                }
            }
        }
    }
}

#[test]
fn self_closing_scripts_have_no_body_insertion_point() {
    let state = ServerState::new();
    for uri in [
        "file:///workspace/App.vue",
        "file:///workspace/Example.art.vue",
    ] {
        let uri = Url::parse(uri).unwrap();
        for source in ["<script lang=\"ts\"/>", "<script setup lang=\"ts\"/>"] {
            let insertion =
                IdeContext::with_content_for_completion(&state, &uri, source.len(), source.into());
            assert_eq!(insertion.block_type, None);
        }
    }
}

#[test]
fn non_script_blocks_keep_their_existing_boundaries() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    for (source, end, expected) in [
        (
            "<template>x</template>",
            "<template>x".len(),
            BlockType::Template,
        ),
        ("<style>x</style>", "<style>x".len(), BlockType::Style(0)),
    ] {
        for (offset, expected) in [(end - 1, Some(expected)), (end, None), (end + 1, None)] {
            let point = IdeContext::with_content(&state, &uri, offset, source.into());
            let insertion =
                IdeContext::with_content_for_completion(&state, &uri, offset, source.into());
            assert_eq!(point.block_type, expected);
            assert_eq!(insertion.block_type, expected);
        }
    }
}

#[test]
fn root_pattern_requests_share_subject_classification_and_completion_insertion_points() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vize.config.json"),
        r#"{"experimentals":{"patternedTemplate":true}}"#,
    )
    .unwrap();
    let enabled = ServerState::new();
    enabled.load_workspace_config(project.path());
    let disabled = ServerState::new();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let source = "<template v-match=\"result\" lang=\"html\"><p v-when=\"_\"/></template>";
    let start = source.find("result").unwrap();
    let end = start + "result".len();
    for offset in 0..source.find('>').unwrap() {
        let expected = (start..=end)
            .contains(&offset)
            .then_some(BlockType::Template);
        assert_eq!(
            IdeContext::with_content_for_completion(&enabled, &uri, offset, source.into())
                .block_type,
            expected,
            "offset {offset}"
        );
        assert_eq!(
            IdeContext::with_content(&enabled, &uri, offset, source.into()).block_type,
            (start..end)
                .contains(&offset)
                .then_some(BlockType::Template)
        );
        assert_eq!(
            IdeContext::with_content_for_completion(&disabled, &uri, offset, source.into())
                .block_type,
            None
        );
    }
    for source in [
        "<template src=\"./other.html\" v-match=\"result\"/>",
        "<template lang=\"pug\" v-match=\"result\">p</template>",
        "<template v-match:arg=\"result\"><p/></template>",
        "<template v-match.once=\"result\"><p/></template>",
    ] {
        let offset = source.find("result").unwrap();
        assert_eq!(
            IdeContext::with_content_for_completion(&enabled, &uri, offset, source.into())
                .block_type,
            None,
            "{source}"
        );
    }
}
