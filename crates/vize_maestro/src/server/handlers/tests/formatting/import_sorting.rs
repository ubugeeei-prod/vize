//! Configured import sorting reaches all formatting handlers from one snapshot.

use super::*;
use std::path::Path;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-import-sorting-6098/App.vue.txt"
));
const SORTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue"
));
const CONFIG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-import-sorting-6098/vite.config.mjs.txt"
));

fn configure(
    server: &MaestroServer,
    root: &Path,
    formatter: serde_json::Value,
) -> std::io::Result<()> {
    let source = vize_l0::cstr!("export default {{ vue: {{ formatter: {formatter} }} }};");
    std::fs::write(root.join("vite.config.mjs"), source.as_bytes())?;
    server.state.load_format_config(root);
    Ok(())
}

fn apply(source: &str, edits: &[TextEdit]) -> Option<Vec<u8>> {
    let mut bytes = source.as_bytes().to_vec();
    for edit in edits.iter().rev() {
        let start = crate::ide::position_to_offset(
            source,
            edit.range.start.line,
            edit.range.start.character,
        )?;
        let end =
            crate::ide::position_to_offset(source, edit.range.end.line, edit.range.end.character)?;
        bytes.splice(start..end, edit.new_text.bytes());
    }
    Some(bytes)
}

fn all_responses(server: &MaestroServer, uri: &Url) -> Result<[Option<Vec<TextEdit>>; 3]> {
    let mut range = range_params(uri.clone());
    range.range = Range::new(Position::new(1, 0), Position::new(3, 0));
    Ok([
        futures::executor::block_on(server.formatting(formatting_params(uri.clone())))?,
        futures::executor::block_on(server.range_formatting(range))?,
        futures::executor::block_on(server.on_type_formatting(on_type_params(uri.clone())))?,
    ])
}

#[test]
fn configured_document_sorting_matches_the_original_whole_reference_and_fixed_point() {
    let project = crate::test_support::tempdir().unwrap();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let service = server_with(&uri, SOURCE, "vue");
    let server = service.inner();
    std::fs::write(project.path().join("vite.config.mjs"), CONFIG).unwrap();
    server.state.load_format_config(project.path());
    let edits = futures::executor::block_on(server.formatting(formatting_params(uri.clone())))
        .unwrap()
        .unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(14, 0)),
            new_text: SORTED.to_owned(),
        }]
    );
    let pinned: Vec<TextEdit> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/editor-import-sorting-6098/response.expected.json"
    )))
    .unwrap();
    assert_eq!(edits, pinned);
    assert_eq!(
        std::str::from_utf8(&apply(SOURCE, &edits).unwrap()).unwrap(),
        SORTED
    );
    for version in [2, 3] {
        server
            .state
            .documents
            .open(uri.clone(), SORTED.to_owned(), version, "vue".to_owned());
        assert_eq!(
            all_responses(server, &uri).unwrap(),
            [Some(Vec::new()), Some(Vec::new()), Some(Vec::new())]
        );
    }
}

#[test]
fn workspace_config_installation_carries_the_authored_sorting_policy() {
    let project = crate::test_support::tempdir().unwrap();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let service = server_with(&uri, SOURCE, "vue");
    let server = service.inner();
    std::fs::write(project.path().join("vite.config.mjs"), CONFIG).unwrap();
    server.state.load_workspace_config(project.path());
    let edits = futures::executor::block_on(server.formatting(formatting_params(uri)))
        .unwrap()
        .unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(14, 0)),
            new_text: SORTED.to_owned(),
        }]
    );
}

#[test]
fn a_script_selection_sorts_imports_without_editing_the_template_or_block_tags() {
    let project = crate::test_support::tempdir().unwrap();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let service = server_with(&uri, SOURCE, "vue");
    let server = service.inner();
    configure(
        server,
        project.path(),
        serde_json::json!({ "sortImports": {} }),
    )
    .unwrap();
    let mut params = range_params(uri.clone());
    params.range = Range::new(Position::new(1, 0), Position::new(3, 0));
    let edits = futures::executor::block_on(server.range_formatting(params))
        .unwrap()
        .unwrap();
    let body = SORTED.get(24..SORTED.find("</script>").unwrap()).unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(0, 24), Position::new(9, 0)),
            new_text: body.to_owned(),
        }]
    );
    assert_eq!(
        std::str::from_utf8(&apply(SOURCE, &edits).unwrap()).unwrap(),
        SORTED
    );
}

#[test]
fn default_and_explicitly_disabled_sorting_keep_all_original_responses() {
    for options in [None, Some(serde_json::json!({ "sortImports": false }))] {
        let project = crate::test_support::tempdir().unwrap();
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        let service = server_with(&uri, SOURCE, "vue");
        let server = service.inner();
        if let Some(options) = options {
            configure(server, project.path(), options).unwrap();
        }
        assert_eq!(
            all_responses(server, &uri).unwrap(),
            [Some(Vec::new()), Some(Vec::new()), Some(Vec::new())]
        );
    }
}

#[test]
fn on_type_never_reorders_imports_and_only_removes_a_known_indent() {
    let project = crate::test_support::tempdir().unwrap();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let service = server_with(&uri, SOURCE, "vue");
    let server = service.inner();
    configure(
        server,
        project.path(),
        serde_json::json!({ "sortImports": {} }),
    )
    .unwrap();
    let mut params = on_type_params(uri.clone());
    params.text_document_position.position = Position::new(1, 3);
    assert_eq!(
        futures::executor::block_on(server.on_type_formatting(params.clone())).unwrap(),
        Some(Vec::new())
    );
    let indented = SORTED.replacen("import { computed }", "  import { computed }", 1);
    server
        .state
        .documents
        .open(uri.clone(), indented.clone(), 2, "vue".to_owned());
    let edits = futures::executor::block_on(server.on_type_formatting(params))
        .unwrap()
        .unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(1, 0), Position::new(1, 2)),
            new_text: "".to_owned(),
        }]
    );
    assert_eq!(
        std::str::from_utf8(&apply(&indented, &edits).unwrap()).unwrap(),
        SORTED
    );
}

#[test]
fn invalid_sorting_refuses_every_handler_and_does_not_reuse_the_previous_policy() {
    let project = crate::test_support::tempdir().unwrap();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let service = server_with(&uri, SOURCE, "vue");
    let server = service.inner();
    configure(
        server,
        project.path(),
        serde_json::json!({ "sortImports": {} }),
    )
    .unwrap();
    assert!(
        server
            .state
            .try_formatter_context_for_path(&project.path().join("App.vue"))
            .is_ok()
    );
    configure(server, project.path(), serde_json::json!({ "sortImports": { "groups": ["external", { "newlinesBetween": true }] } })).unwrap();
    let error = server
        .state
        .try_formatter_context_for_path(&project.path().join("App.vue"))
        .err()
        .unwrap();
    assert_eq!(
        error,
        "Failed to format script: sortImports boundary must sit between two groups"
    );
    assert_eq!(all_responses(server, &uri).unwrap(), [None, None, None]);
    configure(
        server,
        project.path(),
        serde_json::json!({ "sortImports": false }),
    )
    .unwrap();
    assert_eq!(
        all_responses(server, &uri).unwrap(),
        [Some(Vec::new()), Some(Vec::new()), Some(Vec::new())]
    );
}
