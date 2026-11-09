//! #12533: a materialized mirror standing in for a Canon project keeps the
//! Canon root's ambient declarations and package view.

use super::{link_canon_dependencies, write_materialized_project_tsconfig};
use crate::lsp_client::session_storage::SessionStorage;
use std::{fs, path::Path};

const VUE_MODULES: &str = "__vize_vue_modules.d.ts";

fn canon_root(declarations: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("tsconfig.json"),
        r#"{"include":["src/a.ts"]}"#,
    )
    .unwrap();
    for name in declarations {
        fs::write(root.path().join(name), "export {};").unwrap();
    }
    fs::write(root.path().join("unrelated.d.ts"), "export {};").unwrap();
    root
}

fn install(root: &Path, version: &str) {
    let package = root.join("node_modules/vue");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("package.json"), version).unwrap();
}

fn resolved_vue(storage: &SessionStorage) -> Option<String> {
    let owner = storage.root().parent().unwrap();
    fs::read_to_string(owner.join("node_modules/vue/package.json")).ok()
}

fn include(project_root: &Path, storage: &SessionStorage) -> serde_json::Value {
    let config = write_materialized_project_tsconfig(project_root, storage.root(), None).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
    parsed["include"].clone()
}

#[test]
fn materialized_canon_config_names_every_root_declaration() {
    let authored = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(authored.path()).unwrap();
    let all = crate::batch::virtual_project::ROOT_DECLARATION_FILES;
    let canon = canon_root(&all);
    let mut expected = vec![serde_json::json!("**/*")];
    expected.extend(all.iter().map(|name| {
        serde_json::json!(
            crate::file_uri::path_to_typescript_name(&canon.path().join(name)).as_str()
        )
    }));
    assert_eq!(include(canon.path(), &storage), serde_json::json!(expected));

    let partial = canon_root(&[VUE_MODULES, crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME]);
    assert_eq!(
        include(partial.path(), &storage),
        serde_json::json!([
            "**/*",
            crate::file_uri::path_to_typescript_name(&partial.path().join(VUE_MODULES)),
            crate::file_uri::path_to_typescript_name(
                &partial
                    .path()
                    .join(crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME)
            )
        ])
    );
    assert_eq!(
        include(authored.path(), &storage),
        serde_json::json!(["**/*"])
    );
}

#[test]
fn canon_dependencies_replace_the_authored_install() {
    let authored = tempfile::tempdir().unwrap();
    install(authored.path(), "authored");
    let storage = SessionStorage::new(authored.path()).unwrap();
    assert_eq!(resolved_vue(&storage).as_deref(), Some("authored"));

    let canon = canon_root(&[VUE_MODULES]);
    install(canon.path(), "canon");
    for _ in 0..2 {
        link_canon_dependencies(canon.path(), storage.root()).unwrap();
        assert_eq!(resolved_vue(&storage).as_deref(), Some("canon"));
    }
    assert_eq!(
        fs::read_to_string(authored.path().join("node_modules/vue/package.json")).unwrap(),
        "authored"
    );
    let owner = storage.root().parent().unwrap().to_path_buf();
    drop(storage);
    assert!(!owner.exists());
    assert_eq!(
        fs::read_to_string(canon.path().join("node_modules/vue/package.json")).unwrap(),
        "canon"
    );
}

#[test]
fn only_canon_roots_with_their_own_install_replace_the_package_view() {
    let authored = tempfile::tempdir().unwrap();
    install(authored.path(), "authored");
    let storage = SessionStorage::new(authored.path()).unwrap();

    // An authored root, a plain install, and a rootless scratch session.
    let plain = tempfile::tempdir().unwrap();
    install(plain.path(), "plain");
    let scratch = canon_root(&[VUE_MODULES]);
    for root in [authored.path(), plain.path(), scratch.path()] {
        link_canon_dependencies(root, storage.root()).unwrap();
        assert_eq!(resolved_vue(&storage).as_deref(), Some("authored"));
    }

    let bare = tempfile::tempdir().unwrap();
    let unlinked = SessionStorage::new(bare.path()).unwrap();
    let canon = canon_root(&[VUE_MODULES]);
    install(canon.path(), "canon");
    link_canon_dependencies(canon.path(), unlinked.root()).unwrap();
    assert_eq!(resolved_vue(&unlinked).as_deref(), Some("canon"));
}
