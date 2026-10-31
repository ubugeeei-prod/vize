//! Real Windows path namespaces and compiled unsaved dependency inputs.

use super::*;

fn document(documents: &[(String, String)], uri: &str, expected: &str) {
    let matches = documents
        .iter()
        .filter(|(candidate, _)| candidate == uri)
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches.first().unwrap().1.as_str(), expected);
}

#[test]
fn verbatim_windows_overlay_inputs_preserve_materialized_revision_bytes() {
    let input: serde_json::Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/windows-verbatim-overlay-11145/input.json"
    )))
    .unwrap();
    assert_eq!(input["schema"], 1);
    assert_eq!(
        input["id"],
        "typechecker/regression/windows-verbatim-overlay-11145"
    );
    let source = input["main"].as_str().unwrap();
    let script_source = input["scriptMain"].as_str().unwrap();
    let saved_child = input["child"].as_str().unwrap();
    let saved_support = input["support"].as_str().unwrap();
    let config = input["configText"].as_str().unwrap();
    let phases = input["phases"].as_array().unwrap();
    assert_eq!(
        phases
            .iter()
            .map(|phase| phase["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "valid",
            "dependency-error",
            "dependency-repair",
            "dependency-close"
        ]
    );
    let work = tempfile::tempdir().unwrap();
    let root = work.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("src")).unwrap();
    let host = root.join("src/App.vue");
    let script = root.join("src/main.ts");
    let child = root.join("src/Child.vue");
    let support = root.join("src/support.ts");
    std::fs::write(&host, source).unwrap();
    std::fs::write(&script, script_source).unwrap();
    std::fs::write(&child, saved_child).unwrap();
    std::fs::write(&support, saved_support).unwrap();
    std::fs::write(root.join("tsconfig.json"), config).unwrap();
    let verbatim_support = support.canonicalize().unwrap();
    let support_key = vize_carton::path::canonicalize_non_verbatim(&support);
    let verbatim_child = child.canonicalize().unwrap();
    let child_key = vize_carton::path::canonicalize_non_verbatim(&child);
    assert_ne!(verbatim_support, support_key);
    assert_ne!(verbatim_child, child_key);
    let session = EditorMirrorSession::new();
    let mut completed = 0;
    for phase in phases {
        let support_overlay = phase["support"].as_str();
        let child_overlay = phase["child"].as_str();
        assert_eq!(support_overlay.is_some(), child_overlay.is_some());
        let mut inputs = Vec::new();
        if let Some(text) = support_overlay {
            inputs.push((verbatim_support.clone(), text));
        }
        if let Some(text) = child_overlay {
            inputs.push((verbatim_child.clone(), text));
        }
        let normalized = canonical_overlays(&inputs);
        assert_eq!(normalized.get(&support_key).copied(), support_overlay);
        assert_eq!(normalized.get(&child_key).copied(), child_overlay);
        let project = prepare(&session, &host, source, &inputs, &[]);
        let support_member = project
            .host
            .materialized_sources
            .iter()
            .find(|member| member.source_path == support_key)
            .unwrap();
        let child_member = project
            .host
            .materialized_sources
            .iter()
            .find(|member| member.source_path == child_key)
            .unwrap();
        let expected_support = support_overlay.unwrap_or(saved_support);
        let expected_child = child_overlay.unwrap_or(saved_child);
        assert_eq!(support_member.source.as_str(), expected_support);
        assert_eq!(support_member.code.as_str(), expected_support);
        assert_eq!(child_member.source.as_str(), expected_child);
        assert_eq!(
            std::fs::read(&support_member.materialized_path).unwrap(),
            expected_support.as_bytes()
        );
        assert_eq!(
            std::fs::read(&child_member.materialized_path).unwrap(),
            child_member.code.as_bytes()
        );
        let dependency = project
            .host
            .dependencies
            .iter()
            .find(|dependency| {
                vize_carton::path::canonicalize_non_verbatim(&dependency.source_path) == child_key
            })
            .unwrap();
        assert_eq!(dependency.source.as_str(), expected_child);
        assert_eq!(dependency.code, child_member.code);
        let child_uri = crate::file_uri::path_to_file_uri(&child_member.materialized_path);
        document(&project.documents, &child_uri, child_member.code.as_str());
        if support_overlay.is_some() {
            let uri = crate::file_uri::path_to_file_uri(&support_member.materialized_path);
            document(&project.documents, &uri, expected_support);
        }
        let request_path = crate::file_uri::path_to_file_uri(&script);
        let (_, script_documents, script_root, _) =
            crate::corsa_bridge::script_document::build_script_virtual_project(
                &script,
                &request_path,
                script_source,
                oxc_span::SourceType::ts(),
                Default::default(),
                &inputs,
            );
        let script_root = script_root.unwrap();
        let project_root = project.session_project_root.as_ref().unwrap();
        let child_relative = child_member
            .materialized_path
            .strip_prefix(project_root)
            .unwrap();
        let script_child_uri = crate::file_uri::path_to_file_uri(&script_root.join(child_relative));
        document(
            &script_documents,
            &script_child_uri,
            child_member.code.as_str(),
        );
        if support_overlay.is_some() {
            let relative = support_member
                .materialized_path
                .strip_prefix(project_root)
                .unwrap();
            let uri = crate::file_uri::path_to_file_uri(&script_root.join(relative));
            document(&script_documents, &uri, expected_support);
        }
        assert_eq!(std::fs::read(&host).unwrap(), source.as_bytes());
        assert_eq!(std::fs::read(&script).unwrap(), script_source.as_bytes());
        assert_eq!(std::fs::read(&child).unwrap(), saved_child.as_bytes());
        assert_eq!(std::fs::read(&support).unwrap(), saved_support.as_bytes());
        assert_eq!(
            std::fs::read(root.join("tsconfig.json")).unwrap(),
            config.as_bytes()
        );
        completed += 1;
    }
    assert_eq!(completed, 4);
}
