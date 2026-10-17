//! Complete native membership and immutable admission across query scopes.

use super::*;

fn native_documents(project: &CorsaVueVirtualProject) -> Vec<(String, String)> {
    let mut documents = project.documents.clone();
    if let Some(root) = project.session_project_root.as_deref() {
        project
            .host
            .source_catalog
            .extend_native_project_documents(root, &mut documents);
    }
    documents.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    documents
}

#[test]
fn materialization_preserves_typed_configuration_failure_until_physical_repair() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let host = root.join("Host.vue");
    let config = root.join("tsconfig.json");
    std::fs::write(&host, CHILD).unwrap();
    std::fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    let session = EditorMirrorSession::new();
    let build = || {
        let mut env = environment(&session);
        env.project_root = Some(&root);
        env.tsconfig_path = Some(&config);
        build_vue_virtual_workspace_project(&host, CHILD, Default::default(), &[], &[], env)
    };
    for _ in 0..2 {
        let Err(error) = build() else {
            panic!("an unresolved authored base must reject materialization");
        };
        assert!(matches!(&error, CorsaBridgeError::Configuration(_)));
        assert_eq!(
            vize_carton::cstr!("{error}"),
            vize_carton::cstr!(
                "Configuration error: cannot resolve tsconfig extends \"./base.json\" from {}",
                config.display()
            )
            .as_str()
        );
    }
    std::fs::write(root.join("base.json"), "{}").unwrap();
    let repaired = build().unwrap();
    let source = repaired
        .host
        .materialized_sources
        .iter()
        .find(|source| source.source_path == host)
        .unwrap();
    assert_eq!(source.source_path, host);
    assert_eq!(source.source, CHILD);
    assert_eq!(std::fs::read_to_string(&host).unwrap(), CHILD);
}

#[test]
fn native_live_union_contains_closed_sibling_dependencies_before_a_later_query() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("A.vue");
    let b = root.path().join("B.vue");
    let child = root.path().join("Child.vue");
    let dormant = root.path().join("Dormant.vue");
    const B: &str =
        "<script setup lang='ts'>const count = 1;</script><template>{{ count }}</template>";
    for (path, source) in [(&a, HOST), (&b, B), (&child, CHILD), (&dormant, CHILD)] {
        std::fs::write(path, source).unwrap();
    }
    std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
    let session = EditorMirrorSession::new();
    let both = [(a.clone(), HOST), (b.clone(), B)];
    let first = prepare(&session, &a, HOST, &both, &[]);
    let first_packet = first.documents.clone();
    assert_eq!(
        first_packet.len(),
        2,
        "A's exact own/imported query closure"
    );
    let second = prepare(&session, &b, B, &both, &[]);
    let second_packet = second.documents.clone();
    assert!(
        second.host.dependencies.is_empty(),
        "B has no authored imports"
    );
    let mut expected = first_packet.clone();
    expected.push((second.host.request_uri.clone(), second.host.code.clone()));
    expected.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        expected.len(),
        3,
        "only two admitted live hosts and A's closed dependency"
    );
    assert_eq!(native_documents(&second), expected);
    let repeated_a = prepare(&session, &a, HOST, &both, &[]);
    assert_eq!(native_documents(&repeated_a), expected);
    assert_eq!(first.documents, first_packet);
    assert_eq!(second.documents, second_packet);

    let closed = prepare(&session, &b, B, &[(b.clone(), B)], &[]);
    assert_eq!(
        native_documents(&closed),
        vec![(closed.host.request_uri.clone(), closed.host.code.clone())]
    );
    let reopened = prepare(&session, &a, HOST, &both, &[]);
    let current_b = prepare(&session, &b, B, &both, &[]);
    let mut reopened_expected = reopened.documents.clone();
    reopened_expected.push((
        current_b.host.request_uri.clone(),
        current_b.host.code.clone(),
    ));
    reopened_expected.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    reopened_expected.dedup();
    assert_eq!(native_documents(&current_b), reopened_expected);
    assert_eq!(reopened_expected, expected);
    let unsaved = HOST.replace("日本語 🦀", "unsaved 🦀");
    let edited_overlays = [(a.clone(), unsaved.as_str()), (b.clone(), B)];
    let edited = prepare(&session, &a, &unsaved, &edited_overlays, &[]);
    let after_edit = prepare(&session, &b, B, &edited_overlays, &[]);
    let mut edited_expected = edited.documents.clone();
    edited_expected.push((
        after_edit.host.request_uri.clone(),
        after_edit.host.code.clone(),
    ));
    edited_expected.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    edited_expected.dedup();
    assert_eq!(native_documents(&after_edit), edited_expected);
    assert_ne!(edited.host.code, first.host.code);
    assert!(
        !after_edit
            .host
            .source_catalog
            .shares_revision_with(&second.host.source_catalog)
    );
    assert_eq!(first.documents, first_packet);
    assert_eq!(std::fs::read_to_string(&a).unwrap(), HOST);
    assert_eq!(std::fs::read_to_string(&b).unwrap(), B);
    assert_eq!(std::fs::read_to_string(&child).unwrap(), CHILD);
    assert_eq!(std::fs::read_to_string(&dormant).unwrap(), CHILD);
    session.clear();
    let after_retirement = prepare(&session, &b, B, &[(b.clone(), B)], &[]);
    assert_eq!(
        native_documents(&after_retirement),
        vec![(
            after_retirement.host.request_uri.clone(),
            after_retirement.host.code.clone()
        )]
    );
    assert!(
        !after_retirement
            .host
            .source_catalog
            .shares_revision_with(&second.host.source_catalog)
    );
}

#[test]
fn native_union_keeps_effective_tsconfig_namespaces_and_retired_sessions_separate() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("A.vue");
    let b = root.path().join("B.vue");
    for path in [&a, &b] {
        std::fs::write(path, CHILD).unwrap();
    }
    let a_config = root.path().join("a.json");
    let b_config = root.path().join("b.json");
    std::fs::write(
        &a_config,
        r#"{"compilerOptions":{"strict":true},"files":["A.vue"]}"#,
    )
    .unwrap();
    std::fs::write(
        &b_config,
        r#"{"compilerOptions":{"strict":false},"files":["B.vue"]}"#,
    )
    .unwrap();
    let shell = root.path().join("tsconfig.json");
    std::fs::write(
        &shell,
        r#"{"files":[],"references":[{"path":"./a.json"},{"path":"./b.json"}]}"#,
    )
    .unwrap();
    let session = EditorMirrorSession::new();
    let open = |path: &Path, config: &Path, session: &EditorMirrorSession| {
        let mut env = environment(session);
        env.project_root = Some(root.path());
        env.tsconfig_path = Some(config);
        build_vue_virtual_workspace_project(
            path,
            CHILD,
            Default::default(),
            &[(a.clone(), CHILD), (b.clone(), CHILD)],
            &[],
            env,
        )
        .unwrap()
    };
    let first = open(&a, &shell, &session);
    let other = open(&b, &shell, &session);
    assert_ne!(first.session_project_root, other.session_project_root);
    let mut foreign_view = Vec::new();
    first.host.source_catalog.extend_native_project_documents(
        other.session_project_root.as_deref().unwrap(),
        &mut foreign_view,
    );
    assert!(
        foreign_view.is_empty(),
        "a same-directory reference owner cannot borrow sibling native identities"
    );
    assert_eq!(native_documents(&first), first.documents);
    assert_eq!(native_documents(&other), other.documents);
    assert!(
        !first
            .host
            .source_catalog
            .shares_revision_with(&other.host.source_catalog)
    );
    let independent_session = EditorMirrorSession::new();
    let independent = open(&a, &shell, &independent_session);
    assert_ne!(first.session_project_root, independent.session_project_root);
    assert_eq!(native_documents(&independent), independent.documents);
    assert!(
        !first
            .host
            .source_catalog
            .shares_revision_with(&independent.host.source_catalog)
    );
}

#[test]
fn requested_membership_captures_only_the_successfully_materialized_native_revision() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    let app = root.path().join("App.vue");
    for (path, source) in [(&host, HOST), (&child, CHILD), (&app, CHILD)] {
        std::fs::write(path, source).unwrap();
    }
    std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
    let session = EditorMirrorSession::new();
    let overlays = [(host.clone(), HOST)];
    let requested = [(app.clone(), CHILD)];
    let navigation = prepare(&session, &host, HOST, &overlays, &requested);
    let navigation_packet = navigation.documents.clone();
    let navigation_native = native_documents(&navigation);
    assert_eq!(navigation_native.len(), 3);
    let ordinary = prepare(&session, &host, HOST, &overlays, &[]);
    let ordinary_packet = ordinary.documents.clone();
    let mut ordinary_expected = ordinary_packet.clone();
    ordinary_expected.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(ordinary_expected.len(), 2);
    assert_eq!(native_documents(&ordinary), ordinary_expected);
    for (uri, code) in &ordinary_expected {
        let path = crate::file_uri::file_uri_to_path(uri).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), code.as_str());
    }
    assert_eq!(native_documents(&navigation), navigation_native);
    let reopened = prepare(&session, &host, HOST, &overlays, &requested);
    assert_eq!(native_documents(&reopened), navigation_native);
    for (uri, code) in native_documents(&reopened) {
        let path = crate::file_uri::file_uri_to_path(&uri).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), code.as_str());
    }
    assert_eq!(native_documents(&ordinary), ordinary_expected);
    assert_eq!(native_documents(&navigation), navigation_native);
    assert_eq!(ordinary.documents, ordinary_packet);
    assert_eq!(navigation.documents, navigation_packet);
    for (path, source) in [(&host, HOST), (&child, CHILD), (&app, CHILD)] {
        assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    }
}
