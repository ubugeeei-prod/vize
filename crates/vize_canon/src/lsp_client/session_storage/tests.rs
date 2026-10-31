use super::{SessionStorage, write_owned_file};
use crate::{
    file_uri::{file_uri_to_path, path_to_file_uri},
    lsp_client::{CorsaProjectClient, session_paths::build_materialized_session_document_uri},
};
use std::{fs, path::Path};

#[test]
fn preferred_materialized_member_skips_hidden_directories_but_keeps_package_members() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let mut uris = Vec::new();
    for name in [
        ".nuxt/imports.d.ts",
        "src/.generated/types.d.ts",
        "src/z.ts",
        "src/a.ts",
        "node_modules/package/index.d.ts",
    ] {
        let path = storage.root().join(name);
        write_owned_file(storage.root(), &path, b"export {};").unwrap();
        uris.push(path_to_file_uri(&path));
    }
    fn select(uris: &[vize_l0::String], root: &Path) -> Option<vize_l0::String> {
        crate::lsp_client::session_paths::preferred_materialized_document(
            uris.iter(),
            root,
            crate::batch::SourceFilePolicy::default(),
        )
    }
    let package = path_to_file_uri(&storage.root().join("node_modules/package/index.d.ts"));
    assert_eq!(select(&uris, storage.root()), Some(package.clone()));
    uris.reverse();
    assert_eq!(select(&uris, storage.root()), Some(package));
    let visible: Vec<_> = uris
        .iter()
        .filter(|uri| uri.contains("src/a.ts") || uri.contains("src/z.ts"))
        .cloned()
        .collect();
    assert_eq!(
        select(&visible, storage.root()),
        Some(path_to_file_uri(&storage.root().join("src/a.ts")))
    );
    let rejected = [
        path_to_file_uri(&storage.root().join(".nuxt/imports.d.ts")),
        path_to_file_uri(&storage.root().join("src/.generated/types.d.ts")),
        path_to_file_uri(&storage.root().join("missing.ts")),
        path_to_file_uri(&project.path().join("outside.ts")),
        "untitled://outside.ts".into(),
    ];
    fs::write(project.path().join("outside.ts"), "export {};").unwrap();
    assert_eq!(select(&rejected, storage.root()), None);
}

#[test]
fn preferred_materialized_member_uses_native_script_extensions_and_javascript_policy() {
    use crate::{
        batch::SourceFilePolicy, lsp_client::session_paths::preferred_materialized_document,
    };
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    for extension in [
        "ts", "tsx", "mts", "cts", "d.ts", "vue", "txt", "js", "jsx", "mjs", "cjs",
    ] {
        let path = storage
            .root()
            .join(vize_l0::cstr!("App.{extension}").as_str());
        write_owned_file(storage.root(), &path, b"export {};").unwrap();
        let uris = [path_to_file_uri(&path)];
        for (options, javascript) in [
            (serde_json::json!({}), false),
            (serde_json::json!({"allowJs":true}), true),
            (serde_json::json!({"checkJs":true}), true),
            (serde_json::json!({"allowJs":false,"checkJs":true}), false),
        ] {
            let policy = SourceFilePolicy::from_compiler_options(options.as_object().unwrap());
            let expected = match extension {
                "ts" | "tsx" | "mts" | "cts" | "d.ts" => Some(path_to_file_uri(&path)),
                "js" | "jsx" | "mjs" | "cjs" if javascript => Some(path_to_file_uri(&path)),
                _ => None,
            };
            assert_eq!(
                preferred_materialized_document(uris.iter(), storage.root(), policy),
                expected
            );
        }
    }
}

#[test]
fn materialized_member_policy_rereads_the_upcoming_config_and_extends_chain() {
    let project = tempfile::tempdir().unwrap();
    let base = project.path().join("base.jsonc");
    fs::write(
        project.path().join("tsconfig.json"),
        r#"{/* selected config */"extends":"./base.jsonc","compilerOptions":{"checkJs":true}}"#,
    )
    .unwrap();
    let mut client = CorsaProjectClient::empty_for_test(project.path().to_path_buf());
    let root = client.overlay_storage.root().to_path_buf();
    let config = crate::lsp_client::lifecycle_setup::write_materialized_project_tsconfig(
        project.path(),
        &root,
        None,
    )
    .unwrap();
    let member = root.join("member.js");
    write_owned_file(&root, &member, b"export const value = 1;").unwrap();
    let uri = path_to_file_uri(&member);
    for allow_js in [false, true, false] {
        fs::write(
            &base,
            serde_json::to_vec(&serde_json::json!({"compilerOptions":{"allowJs":allow_js}}))
                .unwrap(),
        )
        .unwrap();
        let policy = client.materialized_source_file_policy(&config).unwrap();
        assert_eq!(
            crate::lsp_client::session_paths::preferred_materialized_document(
                std::iter::once(&uri),
                &root,
                policy
            ),
            allow_js.then(|| uri.clone())
        );
    }
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn scratch_project_is_an_owned_sibling_outside_the_materialized_include_scope() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let root = storage.root().to_path_buf();
    let scratch = storage.create_scratch_root().unwrap();
    assert_eq!(scratch.parent(), root.parent());
    assert_eq!(scratch, root.parent().unwrap().join("scratch"));
    assert_ne!(scratch, root);
    fs::write(scratch.join("__vize_vue_modules.d.ts"), "scratch only").unwrap();
    write_owned_file(&root, &root.join("main.ts"), b"materialized only").unwrap();
    assert!(!root.join("__vize_vue_modules.d.ts").exists());
    assert!(!scratch.join("main.ts").exists());
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 0);
    drop(storage);
    assert!(!root.exists());
    assert!(!scratch.exists());
}

#[test]
fn materialized_scratch_config_retains_only_its_two_generated_declarations() {
    use crate::lsp_client::lifecycle_setup::{
        write_materialized_project_tsconfig, write_shared_helper_decls, write_temp_tsconfig,
        write_vue_module_stubs,
    };
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let scratch = storage.create_scratch_root().unwrap();
    write_temp_tsconfig(&scratch).unwrap();
    write_vue_module_stubs(&scratch).unwrap();
    write_shared_helper_decls(&scratch).unwrap();
    let unrelated = scratch.join("unrelated.ts");
    fs::write(&unrelated, "declare const mustStayOutside: unique symbol;").unwrap();
    let config = write_materialized_project_tsconfig(&scratch, storage.root(), None).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
    assert_eq!(
        parsed["include"],
        serde_json::json!([
            "**/*",
            crate::file_uri::path_to_typescript_name(&scratch.join("__vize_vue_modules.d.ts")),
            crate::file_uri::path_to_typescript_name(
                &scratch.join(crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME)
            )
        ])
    );
    assert_eq!(parsed.get("files"), None);
    assert_eq!(parsed["exclude"], serde_json::json!([]));
    assert_eq!(
        fs::read(scratch.join(crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME)).unwrap(),
        crate::virtual_ts::SHARED_PREAMBLE_DTS.as_bytes()
    );
    let authored =
        write_materialized_project_tsconfig(project.path(), storage.root(), None).unwrap();
    let authored: serde_json::Value = serde_json::from_slice(&fs::read(authored).unwrap()).unwrap();
    assert_eq!(authored.get("files"), None);
    assert_eq!(authored["include"], serde_json::json!(["**/*"]));
    assert!(unrelated.is_file());
}

#[test]
fn preferred_materialized_member_refuses_foreign_config_and_incomplete_membership() {
    let storage = SessionStorage::new(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let config = storage.root().join("tsconfig.json");
    let foreign = storage.root().join("other.json");
    let member = storage.root().join("main.ts");
    fs::write(&config, "{}").unwrap();
    fs::write(&foreign, "{}").unwrap();
    fs::write(&member, "export const value = 1;").unwrap();
    let mut project: corsa::api::ProjectResponse = serde_json::from_value(serde_json::json!({"id":"fixture","configFileName":config,"compilerOptions":{},"rootFiles":[member]})).unwrap();
    let uri = path_to_file_uri(&member);
    let validate = crate::lsp_client::session::validate_materialized_project_selection;
    assert_eq!(validate(&project, &config, Some(&uri)), Ok(()));
    project.config_file_name = foreign.to_string_lossy().as_ref().into();
    let refusal = vize_l0::cstr!(
        "Preferred materialized Corsa member did not retain its exact configured project"
    );
    assert_eq!(
        validate(&project, &config, Some(&uri)),
        Err(refusal.clone())
    );
    assert_eq!(validate(&project, &config, None), Ok(()));
    project.config_file_name = config.to_string_lossy().as_ref().into();
    project.root_files.clear();
    assert_eq!(validate(&project, &config, Some(&uri)), Err(refusal));
}

#[test]
fn private_owners_are_unique_and_drop_only_their_own_complete_mirror() {
    let project = tempfile::tempdir().unwrap();
    let original = project.path().join("source.ts");
    fs::write(&original, "export const value = 1;\r\n").unwrap();
    let before = fs::read(&original).unwrap();
    let first = SessionStorage::new(project.path()).unwrap();
    let second = SessionStorage::new(project.path()).unwrap();
    assert_ne!(first.root(), second.root());
    assert_eq!(
        first
            .root()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap(),
        std::env::temp_dir().canonicalize().unwrap()
    );
    let retained = second.root().join("source.ts");
    write_owned_file(second.root(), &retained, b"second owner").unwrap();
    let removed = first.root().to_path_buf();
    write_owned_file(first.root(), &removed.join("source.ts"), b"first owner").unwrap();
    drop(first);
    assert!(!removed.exists());
    assert_eq!(fs::read(&retained).unwrap(), b"second owner");
    assert_eq!(fs::read(&original).unwrap(), before);
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 1);
}

#[test]
fn dependency_link_is_a_read_only_lookup_sibling_of_the_writable_namespace() {
    let project = tempfile::tempdir().unwrap();
    let package = project.path().join("node_modules/vue");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("index.d.ts"), "export const original: 1;").unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let dependency = storage
        .root()
        .parent()
        .unwrap()
        .join("node_modules/vue/index.d.ts");
    assert_eq!(
        dependency.canonicalize().unwrap(),
        package.join("index.d.ts").canonicalize().unwrap()
    );
    assert!(write_owned_file(storage.root(), &dependency, b"must not write").is_err());
    let mirrored = storage.root().join("node_modules/vue/generated.ts");
    write_owned_file(storage.root(), &mirrored, b"export const local = true;").unwrap();
    assert_eq!(
        fs::read(package.join("index.d.ts")).unwrap(),
        b"export const original: 1;"
    );
    assert!(!package.join("generated.ts").exists());
    let owner = storage.root().parent().unwrap().to_path_buf();
    drop(storage);
    assert!(!owner.exists());
    assert_eq!(
        fs::read(package.join("index.d.ts")).unwrap(),
        b"export const original: 1;"
    );
}

#[test]
fn document_mappings_cannot_escape_the_private_namespace() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    for external in [
        project.path().join("src/App.vue.ts"),
        project.path().join("src/../escape.ts"),
        std::env::temp_dir().join("different/Outside.vue.ts"),
    ] {
        let mapped = build_materialized_session_document_uri(
            &path_to_file_uri(&external),
            project.path(),
            storage.root(),
        )
        .unwrap();
        let path = file_uri_to_path(&mapped).unwrap();
        let relative = path.strip_prefix(storage.root()).unwrap();
        assert_eq!(storage.root().join(relative), path);
        assert!(
            !path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        );
        write_owned_file(storage.root(), &path, b"private bytes").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"private bytes");
    }
    assert!(!project.path().join("src/App.vue.ts").exists());
    assert!(!project.path().join("src/../escape.ts").exists());
}

#[cfg(windows)]
#[test]
fn drive_prefixes_and_unc_paths_remain_private_and_distinct() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let mut mapped = Vec::new();
    for path in [
        r"C:\outside\App.vue.ts",
        r"D:\outside\App.vue.ts",
        r"\\server\share\App.vue.ts",
    ] {
        let uri = build_materialized_session_document_uri(
            &path_to_file_uri(Path::new(path)),
            project.path(),
            storage.root(),
        )
        .unwrap();
        let path = file_uri_to_path(&uri).unwrap();
        let relative = path.strip_prefix(storage.root()).unwrap();
        assert_eq!(storage.root().join(relative), path);
        assert_eq!(
            mapped.iter().filter(|previous| **previous == path).count(),
            0
        );
        write_owned_file(storage.root(), &path, b"private Windows member").unwrap();
        mapped.push(path);
    }
}

fn link(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    junction::create(source, target).unwrap();
}

#[test]
fn linked_private_root_parent_and_leaf_refuse_open_writes() {
    for boundary in ["root", "parent", "leaf"] {
        let project = tempfile::tempdir().unwrap();
        let storage = SessionStorage::new(project.path()).unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("member.ts");
        fs::write(&target, b"external bytes").unwrap();
        let destination = storage.root().join("nested/member.ts");
        match boundary {
            "root" => {
                fs::remove_dir(storage.root()).unwrap();
                link(outside.path(), storage.root());
            }
            "parent" => {
                link(outside.path(), &storage.root().join("nested"));
            }
            _ => {
                fs::create_dir(destination.parent().unwrap()).unwrap();
                #[cfg(unix)]
                std::os::unix::fs::symlink(&target, &destination).unwrap();
                #[cfg(windows)]
                fs::hard_link(&target, &destination).unwrap();
            }
        }
        #[cfg(unix)]
        assert!(write_owned_file(storage.root(), &destination, b"new bytes").is_err());
        #[cfg(windows)]
        if boundary == "leaf" {
            write_owned_file(storage.root(), &destination, b"new bytes").unwrap();
            assert_eq!(fs::read(&destination).unwrap(), b"new bytes");
        } else {
            assert!(write_owned_file(storage.root(), &destination, b"new bytes").is_err());
        }
        assert_eq!(fs::read(&target).unwrap(), b"external bytes");
    }
}

#[test]
fn dropping_a_client_removes_its_private_storage_after_the_owner() {
    let project = tempfile::tempdir().unwrap();
    let client = CorsaProjectClient::empty_for_test(project.path().to_path_buf());
    let mirror = client.overlay_storage.root().to_path_buf();
    assert!(mirror.exists());
    assert_eq!(client.project_root, project.path());
    assert_eq!(client.cwd, project.path());
    drop(client);
    assert!(!mirror.exists());
    assert!(project.path().exists());
}

#[test]
fn writer_rejects_all_parent_components_before_creating_missing_directories() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let path = storage.root().join("missing/../../node_modules/escape.ts");
    assert!(write_owned_file(storage.root(), &path, b"must never write").is_err());
    assert!(!storage.root().join("missing").exists());
    assert!(
        !storage
            .root()
            .parent()
            .unwrap()
            .join("node_modules")
            .exists()
    );
}

#[test]
fn failed_shutdown_retains_only_the_private_owner_for_unproven_native_lifetime() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let owner = storage.root().parent().unwrap().to_path_buf();
    storage.retain_after_failed_shutdown();
    drop(storage);
    assert!(owner.exists());
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 0);
    fs::remove_dir_all(owner).unwrap();
}

#[test]
fn failed_retirement_is_sticky_after_later_successful_shutdown() {
    let project = tempfile::tempdir().unwrap();
    let storage = SessionStorage::new(project.path()).unwrap();
    let owner = storage.root().parent().unwrap().to_path_buf();
    storage.observe_retirement(&Err("injected close failure".into()));
    storage.observe_retirement(&Ok(()));
    drop(storage);
    assert!(owner.exists());
    fs::remove_dir_all(owner).unwrap();
}
