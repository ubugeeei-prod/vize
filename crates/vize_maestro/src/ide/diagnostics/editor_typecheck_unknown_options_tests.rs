//! Source-built DiagnosticService vectors; this is not a stdio-RPC witness.
use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config,
};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};
#[path = "../../../../vize/tests/support/unknown_template_fixture.rs"]
mod fixture;

#[test]
fn original_unknown_options_preserve_complete_editor_vectors() {
    let Some(corsa) = resolve_test_tsgo_binary() else {
        return;
    };
    let messages = fixture::messages(&corsa).unwrap();
    let full: Vec<_> = messages
        .into_iter()
        .zip([
            (2353, Range::new(Position::new(7, 23), Position::new(7, 35))),
            (2339, Range::new(Position::new(8, 5), Position::new(8, 16))),
            (2339, Range::new(Position::new(9, 9), Position::new(9, 26))),
        ])
        .map(|(message, (code, range))| Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(code)),
            source: Some("vize/types".into()),
            message,
            ..Default::default()
        })
        .collect();
    for (name, config, indexes) in fixture::cases().unwrap() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        fixture::project(&root, &config).unwrap();
        write_corsa_config(&root, &corsa);
        let expected: Vec<_> = indexes
            .iter()
            .map(|index| full.get(*index).unwrap().clone())
            .collect();
        for (file, source, expected) in [
            ("src/Child.vue", fixture::CHILD, Vec::new()),
            ("src/Parent.vue", fixture::PARENT, expected),
        ] {
            let uri = Url::from_file_path(root.join(file)).unwrap();
            let state = state_for_fixture(&root, &uri, source);
            state.load_workspace_config(&root);
            let actual = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
            if let Some(capture) = std::env::var_os("VIZE_UNKNOWN_TEMPLATE_CAPTURE") {
                let output = std::path::PathBuf::from(capture)
                    .join(vize_l0::cstr!("editor-{name}").as_str());
                fixture::write(&output, file, source).unwrap();
                fixture::write(&output, "tsconfig.json", &config).unwrap();
                fixture::write(
                    &output,
                    vize_l0::cstr!("{file}.diagnostics.json").as_str(),
                    serde_json::to_vec(&actual).unwrap(),
                )
                .unwrap();
            }
            assert_eq!(actual, expected, "{name}: {file} whole editor response");
        }
        assert_eq!(
            std::fs::read(root.join("src/Child.vue")).unwrap(),
            fixture::CHILD.as_bytes()
        );
        assert_eq!(
            std::fs::read(root.join("src/Parent.vue")).unwrap(),
            fixture::PARENT.as_bytes()
        );
        assert_eq!(std::fs::read(root.join("tsconfig.json")).unwrap(), config);
    }
}
