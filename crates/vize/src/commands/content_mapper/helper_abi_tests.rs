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
        let mut open = open_project_request(2, Value::Null, json!({}));
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
