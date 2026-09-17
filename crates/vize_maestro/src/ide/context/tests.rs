use super::IdeContext;
use crate::server::ServerState;
use crate::virtual_code::BlockType;
use tower_lsp::lsp_types::Url;

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
fn root_pattern_completion_is_limited_to_enabled_subject_insertion_points() {
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
            None
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
