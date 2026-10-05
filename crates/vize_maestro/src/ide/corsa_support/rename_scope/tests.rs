#![expect(
    clippy::disallowed_macros,
    reason = "adversarial URI fixture construction"
)]

use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, CreateFile, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};

use super::RenameScope;
use crate::ide::IdeContext;
use crate::ide::corsa_support::canonical_dependency_tests::host_document;
use crate::server::ServerState;

const SOURCE: &str = "<script setup>const value = 1</script><template>{{ value }}</template>";

fn entry(uri: Url) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(
            uri,
            vec![TextEdit {
                range: Range::new(Position::new(0, 8), Position::new(0, 14)),
                new_text: "update".to_string(),
            }],
        )])),
        document_changes: None,
        change_annotations: None,
    }
}

#[test]
fn mixed_library_and_authored_native_edits_refuse_every_container() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    std::fs::write(&path, SOURCE).unwrap();
    let library = root.path().join("node_modules/typescript/lib/lib.dom.d.ts");
    std::fs::create_dir_all(library.parent().unwrap()).unwrap();
    std::fs::write(&library, "interface Events { change: Event }").unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let library_uri = Url::from_file_path(&library).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let native_uri = Url::parse(&document.request_uri).unwrap();
    let mut scope = RenameScope::new(&ctx);
    let original = entry(native_uri.clone());
    assert!(scope.admits_native(&document, &original));
    let mut mixed = original.clone();
    mixed
        .changes
        .as_mut()
        .unwrap()
        .extend(entry(library_uri.clone()).changes.unwrap());
    assert!(!scope.admits_native(&document, &mixed));
    assert_eq!(original, entry(native_uri));

    let mixed_document = WorkspaceEdit {
        changes: original.changes,
        document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: library_uri,
                version: Some(3),
            },
            edits: vec![OneOf::Right(AnnotatedTextEdit {
                text_edit: TextEdit {
                    range: Range::new(Position::new(0, 19), Position::new(0, 25)),
                    new_text: "update".to_string(),
                },
                annotation_id: "event-rename".to_string(),
            })],
        }])),
        change_annotations: None,
    };
    assert!(!scope.admits_native(&document, &mixed_document));
    assert_eq!(std::fs::read_to_string(path).unwrap(), SOURCE);
    assert_eq!(
        std::fs::read_to_string(library).unwrap(),
        "interface Events { change: Event }"
    );
}

#[test]
fn complete_file_identity_refuses_same_path_wrong_scheme_or_authority() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let native = Url::parse(&document.request_uri).unwrap();
    let mut scope = RenameScope::new(&ctx);
    for foreign in [
        Url::parse(&format!("https://foreign.example{}", native.path())).unwrap(),
        Url::parse(&format!("file://foreign.example{}", native.path())).unwrap(),
    ] {
        let mut mixed = entry(native.clone());
        mixed
            .changes
            .as_mut()
            .unwrap()
            .extend(entry(foreign).changes.unwrap());
        assert!(!scope.admits_native(&document, &mixed));
    }
}

#[test]
fn registered_sibling_sources_and_authored_declarations_remain_writable() {
    let root = tempfile::tempdir().unwrap();
    let sibling = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state.set_workspace_folders(vec![
        root.path().to_path_buf(),
        sibling.path().to_path_buf(),
    ]);
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let mut scope = RenameScope::new(&ctx);
    for name in ["shared.ts", "shared.js", "types.d.ts", "space 💥.ts"] {
        let target = sibling.path().join(name);
        std::fs::write(&target, "export const value = 1;").unwrap();
        let edit = entry(Url::from_file_path(target).unwrap());
        assert!(scope.admits_native(&document, &edit));
        assert!(scope.admits_authored(&edit));
    }
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("shared.ts");
    std::fs::write(&target, "export const value = 1;").unwrap();
    assert!(!scope.admits_native(&document, &entry(Url::from_file_path(target).unwrap())));
}

#[cfg(unix)]
#[test]
fn symlinked_workspace_is_owned_but_symlink_escapes_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let physical = root.path().join("physical");
    let logical = root.path().join("workspace");
    std::fs::create_dir_all(&physical).unwrap();
    std::os::unix::fs::symlink(&physical, &logical).unwrap();
    let path = logical.join("App.vue");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(logical.clone());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let mut scope = RenameScope::new(&ctx);
    for path in [logical.join("shared.ts"), physical.join("shared.ts")] {
        std::fs::write(&path, "export const value = 1;").unwrap();
        assert!(scope.admits_native(&document, &entry(Url::from_file_path(path).unwrap())));
    }
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("shared.ts"), "export const value = 1;").unwrap();
    std::os::unix::fs::symlink(outside.path(), logical.join("escape")).unwrap();
    assert!(!scope.admits_native(
        &document,
        &entry(Url::from_file_path(logical.join("escape/shared.ts")).unwrap()),
    ));
}

#[test]
fn captured_open_package_sfc_keeps_its_authored_role_without_blessing_libraries() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("node_modules/@scope/ui/Entry.vue");
    let library = root.path().join("node_modules/typescript/lib/lib.dom.d.ts");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(library.parent().unwrap()).unwrap();
    std::fs::write(&path, SOURCE).unwrap();
    std::fs::write(&library, "interface Events { change: Event }").unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let library_uri = Url::from_file_path(&library).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    state.documents.open(
        library_uri.clone(),
        "interface Events { change: Event }".to_string(),
        1,
        "typescript".to_string(),
    );
    state.update_virtual_docs(&library_uri, "interface Events { change: Event }");
    let ctx = IdeContext::new(&state, &uri, 24).unwrap();
    let document = host_document(&uri, SOURCE);
    let native = Url::parse(&document.request_uri).unwrap();
    let mut scope = RenameScope::new(&ctx);
    assert!(scope.admits_native(&document, &entry(native.clone())));
    assert!(scope.admits_authored(&entry(uri.clone())));
    assert!(!scope.admits_native(&document, &entry(uri.clone())));
    assert!(!scope.admits_authored(&entry(library_uri.clone())));
    let mut mixed = entry(native);
    mixed
        .changes
        .as_mut()
        .unwrap()
        .extend(entry(library_uri).changes.unwrap());
    assert!(!scope.admits_native(&document, &mixed));
}

#[test]
fn mixed_unknown_private_or_synthetic_targets_cannot_leave_partial_edits() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let mut document = host_document(&uri, SOURCE);
    let private = root.path().join("private-session");
    std::fs::create_dir_all(&private).unwrap();
    let unknown_private = private.join("unowned.ts");
    std::fs::write(&unknown_private, "export const value = 1;").unwrap();
    document.session_project_roots.push(private);
    let native = Url::parse(&document.request_uri).unwrap();
    let mut scope = RenameScope::new(&ctx);
    for target in [unknown_private, root.path().join("Missing.vue.ts")] {
        let mut mixed = entry(native.clone());
        mixed
            .changes
            .as_mut()
            .unwrap()
            .extend(entry(Url::from_file_path(target).unwrap()).changes.unwrap());
        assert!(!scope.admits_native(&document, &mixed));
    }
    let real = root.path().join("Actual.vue.ts");
    std::fs::write(&real, "export const value = 1;").unwrap();
    assert!(scope.admits_native(&document, &entry(Url::from_file_path(real).unwrap())));
}

#[cfg(unix)]
#[test]
fn dangling_existing_links_are_not_reinterpreted_as_absent_authored_leaves() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let dangling = root.path().join("dangling.ts");
    std::os::unix::fs::symlink(outside.path().join("absent.ts"), &dangling).unwrap();
    let mut scope = RenameScope::new(&ctx);
    let dangling_edit = entry(Url::from_file_path(&dangling).unwrap());
    assert!(!scope.admits_authored(&dangling_edit));
    assert!(!scope.admits_native(&document, &dangling_edit));
    let absent = Url::from_file_path(root.path().join("new.ts")).unwrap();
    assert!(scope.admits_authored(&entry(absent.clone())));
    let create = WorkspaceEdit {
        changes: None,
        document_changes: Some(DocumentChanges::Operations(vec![
            DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                uri: absent,
                options: None,
                annotation_id: None,
            })),
        ])),
        change_annotations: None,
    };
    assert!(scope.admits_native(&document, &create));
}
