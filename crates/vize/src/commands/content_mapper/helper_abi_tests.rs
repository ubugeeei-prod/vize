//! Shared lowering requires the complete installed declaration contract.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_l0::String as CompactString;

use super::test_support::{
    exchange, frames, initialize_request, open_project_request, transform_request,
};

pub(super) struct InstalledSdk {
    _directory: tempfile::TempDir,
    pub config: PathBuf,
    pub package: PathBuf,
}

impl InstalledSdk {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("app");
        let package = directory.path().join("node_modules/vize");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&package).unwrap();
        let config = project.join("tsconfig.json");
        std::fs::write(&config, "{}").unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm/cli");
        copy_declarations(&source, &package);
        std::fs::copy(source.join("package.json"), package.join("package.json")).unwrap();
        let vue = directory.path().join("node_modules/vue");
        std::fs::create_dir_all(&vue).unwrap();
        std::fs::write(vue.join("package.json"), "{}").unwrap();
        Self {
            _directory: directory,
            config,
            package,
        }
    }

    pub fn transform(&self) -> CompactString {
        self.transform_with_options(json!({}))
    }

    fn open_result(&self) -> Value {
        self.open_result_with_options(json!({}))
    }

    fn open_result_with_options(&self, compiler_options: Value) -> Value {
        let mut open = open_project_request(2, Value::Null, compiler_options);
        open["params"]["configFileName"] = json!(self.config);
        let responses = exchange(&frames(&[initialize_request(), open]));
        responses[1]["result"].clone()
    }

    fn transform_with_options(&self, compiler_options: Value) -> CompactString {
        let mut open = open_project_request(2, Value::Null, compiler_options);
        open["params"]["configFileName"] = json!(self.config);
        let responses = exchange(&frames(&[
            initialize_request(),
            open,
            transform_request(
                3,
                "<script setup lang=\"ts\">defineProps<{ benchCount: number }>();</script><template><div /></template>",
            ),
        ]));
        CompactString::from(responses[2]["result"]["text"].as_str().unwrap())
    }

    fn rename(&self, file: &str, before: &str, after: &str) {
        let file = self.package.join(file);
        let source = std::fs::read_to_string(&file).unwrap();
        assert_eq!(source.matches(before).count(), 1, "{}", file.display());
        std::fs::write(file, source.replacen(before, after, 1)).unwrap();
    }

    #[cfg(unix)]
    fn move_to_pnpm_store(&self, same_vue: bool) {
        let root = self._directory.path();
        let physical = root.join("store/vize/node_modules/vize");
        std::fs::create_dir_all(physical.parent().unwrap()).unwrap();
        std::fs::rename(&self.package, &physical).unwrap();
        std::os::unix::fs::symlink(&physical, &self.package).unwrap();
        let helper_vue = physical.parent().unwrap().join("vue");
        if same_vue {
            std::os::unix::fs::symlink(root.join("node_modules/vue"), helper_vue).unwrap();
        } else {
            std::fs::create_dir_all(&helper_vue).unwrap();
            std::fs::write(helper_vue.join("package.json"), "{}").unwrap();
        }
    }
}

fn copy_declarations(source: &Path, target: &Path) {
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if entry.file_type().unwrap().is_dir() {
            if name == "generated" {
                std::fs::create_dir_all(target.join(&name)).unwrap();
                copy_declarations(&entry.path(), &target.join(&name));
            }
        } else if name.to_str().is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        }) {
            std::fs::copy(entry.path(), target.join(name)).unwrap();
        }
    }
}

fn assert_self_contained(text: &str) {
    assert!(
        !text.contains("import type {} from \"vize/virtual-ts-checker\";"),
        "an unqualified SDK must not own generated types: {text}"
    );
    assert!(text.contains("type $ComponentRawProps<"), "{text}");
}

#[test]
fn intact_sdk_keeps_the_shared_lowering() {
    let sdk = InstalledSdk::new();
    let text = sdk.transform();
    assert!(
        text.contains("import type {} from \"vize/virtual-ts-checker\";"),
        "{text}"
    );
    assert!(!text.contains("type $ComponentInput<"), "{text}");
}

#[test]
fn every_conditional_type_entry_must_supply_the_consumed_helper() {
    for entry in ["virtual-ts.d.ts", "virtual-ts.d.mts", "virtual-ts.d.cts"] {
        let sdk = InstalledSdk::new();
        sdk.rename(
            entry,
            "type $ComponentRawProps<",
            "type $MissingComponentRawProps<",
        );
        assert_self_contained(&sdk.transform());
    }
}

#[test]
fn an_unavailable_private_dependency_cannot_enable_shared_lowering() {
    let sdk = InstalledSdk::new();
    std::fs::remove_file(sdk.package.join("virtual-ts-emits.d.ts")).unwrap();
    assert_self_contained(&sdk.transform());
}

#[test]
fn a_missing_private_binding_invalidates_the_same_dependency_closure() {
    let sdk = InstalledSdk::new();
    sdk.rename(
        "virtual-ts-emits.d.ts",
        "type $LooseEmitPropsFactory<",
        "type $MissingLooseEmitPropsFactory<",
    );
    assert_self_contained(&sdk.transform());
}

#[test]
fn a_conditional_export_cannot_redirect_the_qualified_sdk() {
    let sdk = InstalledSdk::new();
    let manifest = sdk.package.join("package.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    value["exports"]["./virtual-ts"]["require"]["types"] = json!("./virtual-ts.d.ts");
    std::fs::write(manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    assert_self_contained(&sdk.transform());
}

#[test]
fn user_resolution_overrides_cannot_bypass_the_declaration_contract() {
    for compiler_options in [
        json!({"paths": {"vize/*": ["./custom-sdk/*"]}}),
        json!({"moduleSuffixes": [".custom", ""]}),
    ] {
        let sdk = InstalledSdk::new();
        assert_self_contained(&sdk.transform_with_options(compiler_options));
    }
}

#[test]
fn sdk_repairs_invalidate_the_open_project_identity_and_watch_closure() {
    let sdk = InstalledSdk::new();
    let before = sdk.open_result();
    let declaration = sdk.package.join("virtual-ts.d.mts");
    let original = std::fs::read(&declaration).unwrap();
    std::fs::remove_file(&declaration).unwrap();
    let incomplete = sdk.open_result();
    assert_ne!(before["configIdentity"], incomplete["configIdentity"]);
    assert!(
        incomplete["watchedFiles"]
            .as_array()
            .unwrap()
            .contains(&json!(declaration))
    );
    std::fs::write(declaration, original).unwrap();
    assert_eq!(before, sdk.open_result());
}

#[cfg(unix)]
#[test]
fn pnpm_helpers_use_their_physical_vue_peer_by_default() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    assert_self_contained(&sdk.transform());
}

#[cfg(unix)]
#[test]
fn pnpm_helpers_with_the_same_physical_vue_peer_keep_shared_types() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(true);
    assert!(
        sdk.transform()
            .contains("import type {} from \"vize/virtual-ts-checker\";")
    );
}

#[cfg(unix)]
#[test]
fn preserve_symlinks_uses_the_logical_vue_authority() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    assert!(
        sdk.transform_with_options(json!({"preserveSymlinks": true}))
            .contains("import type {} from \"vize/virtual-ts-checker\";")
    );
}

#[cfg(unix)]
#[test]
fn pnpm_peer_repairs_are_watched_and_change_the_project_identity() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    let physical = std::fs::canonicalize(&sdk.package).unwrap();
    let peer = physical.parent().unwrap().join("vue");
    let before = sdk.open_result();
    assert!(
        before["watchedFiles"]
            .as_array()
            .unwrap()
            .contains(&json!(peer.join("package.json")))
    );
    std::fs::remove_dir_all(&peer).unwrap();
    std::os::unix::fs::symlink(sdk._directory.path().join("node_modules/vue"), peer).unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    assert!(
        sdk.transform()
            .contains("import type {} from \"vize/virtual-ts-checker\";")
    );
}

#[test]
fn a_new_closer_vue_installation_is_watched_before_it_exists() {
    let sdk = InstalledSdk::new();
    let closer = sdk.config.parent().unwrap().join("node_modules/vue");
    let before = sdk.open_result();
    assert!(
        before["watchedFiles"]
            .as_array()
            .unwrap()
            .contains(&json!(closer.join("package.json")))
    );
    std::fs::create_dir_all(&closer).unwrap();
    std::fs::write(closer.join("package.json"), "{}").unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    assert_self_contained(&sdk.transform());
}

#[test]
fn absent_wildcard_targets_keep_sharing_until_a_redirect_can_exist() {
    let sdk = InstalledSdk::new();
    let options = json!({"baseUrl": ".", "paths": {"*": ["types/*"]}});
    std::fs::write(
        &sdk.config,
        serde_json::to_vec(&json!({"compilerOptions": options})).unwrap(),
    )
    .unwrap();
    let types = sdk.config.parent().unwrap().join("types");
    std::fs::create_dir_all(&types).unwrap();
    let before = sdk.open_result_with_options(options.clone());
    assert!(
        before["watchedFiles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| { Path::new(file.as_str().unwrap()).ends_with("types/vize") })
    );
    assert!(
        sdk.transform_with_options(options.clone())
            .contains("import type {} from \"vize/virtual-ts-checker\";")
    );
    std::fs::create_dir_all(types.join("vize")).unwrap();
    std::fs::write(types.join("vize/virtual-ts.d.ts"), "export {};\n").unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result_with_options(options.clone())["configIdentity"]
    );
    assert_self_contained(&sdk.transform_with_options(options));
}

#[test]
fn first_sdk_installation_is_in_the_missing_lookup_watch_closure() {
    let sdk = InstalledSdk::new();
    let held = sdk._directory.path().join("held-sdk");
    std::fs::rename(&sdk.package, &held).unwrap();
    let before = sdk.open_result();
    assert!(
        before["watchedFiles"]
            .as_array()
            .unwrap()
            .contains(&json!(sdk.package.join("package.json")))
    );
    std::fs::rename(held, &sdk.package).unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    assert!(
        sdk.transform()
            .contains("import type {} from \"vize/virtual-ts-checker\";")
    );
}

#[cfg(unix)]
#[test]
fn closer_sdk_candidates_are_watched_before_an_installed_parent() {
    let sdk = InstalledSdk::new();
    let closer = sdk.config.parent().unwrap().join("node_modules/vize");
    let before = sdk.open_result();
    assert!(
        before["watchedFiles"]
            .as_array()
            .unwrap()
            .contains(&json!(closer.join("package.json")))
    );
    std::fs::create_dir_all(closer.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&sdk.package, closer).unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    assert!(
        sdk.transform()
            .contains("import type {} from \"vize/virtual-ts-checker\";")
    );
}

#[cfg(unix)]
#[test]
fn a_manifest_symlink_cannot_move_the_sdk_vue_authority() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    let metadata = sdk._directory.path().join("metadata/package.json");
    std::fs::create_dir_all(metadata.parent().unwrap()).unwrap();
    let manifest = sdk.package.join("package.json");
    std::fs::rename(&manifest, &metadata).unwrap();
    std::os::unix::fs::symlink(metadata, manifest).unwrap();
    assert_self_contained(&sdk.transform());
}

#[cfg(unix)]
#[test]
fn shared_manifest_bytes_do_not_merge_distinct_vue_package_directories() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    let project = sdk._directory.path().join("node_modules/vue/package.json");
    let physical = std::fs::canonicalize(&sdk.package).unwrap();
    let peer = physical.parent().unwrap().join("vue/package.json");
    let metadata = sdk._directory.path().join("metadata/vue.json");
    std::fs::create_dir_all(metadata.parent().unwrap()).unwrap();
    std::fs::rename(&project, &metadata).unwrap();
    std::fs::remove_file(&peer).unwrap();
    std::os::unix::fs::symlink(&metadata, project).unwrap();
    std::os::unix::fs::symlink(metadata, peer).unwrap();
    assert_self_contained(&sdk.transform());
}
