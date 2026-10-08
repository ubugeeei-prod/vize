//! The same whole corpus drives pure selection and native-required CLI emit.
use super::{TsconfigInputCache, load_tsconfig_declaration_options};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-diamond-3984/input.json"
));

fn project() -> (TempDir, PathBuf) {
    let temporary = TempDir::new().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let input: Value = serde_json::from_str(INPUT).unwrap();
    for (path, content) in input["files"].as_object().unwrap() {
        let target = root.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, content.as_str().unwrap()).unwrap();
    }
    (temporary, root)
}

fn set_config(root: &Path, update: impl FnOnce(&mut Value)) {
    let path = root.join("tsconfig.json");
    let mut config: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    update(&mut config);
    fs::write(path, serde_json::to_string(&config).unwrap()).unwrap();
}

fn selected(root: &Path) -> Vec<std::string::String> {
    super::collect_default_check_files(
        root,
        Some(&root.join("tsconfig.json")),
        false,
        &mut TsconfigInputCache::default(),
    )
    .iter()
    .map(|path| {
        path.strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    })
    .collect()
}

#[test]
fn later_sibling_replays_complete_shared_input_and_declaration_values() {
    let (_temporary, root) = project();
    assert_eq!(selected(&root), ["src/base/App.vue", "src/base/main.ts"]);
    let mut cache = TsconfigInputCache::default();
    let inputs = cache.load(&root.join("tsconfig.json")).unwrap();
    assert_eq!(inputs.allow_js, Some(false));
    let declarations = load_tsconfig_declaration_options(&root.join("tsconfig.json"));
    assert_eq!(
        declarations.declaration_dir,
        Some(root.join("configs/../types-base"))
    );
    assert_eq!(
        declarations.out_dir.unwrap(),
        root.join("configs/../dist-base")
    );
    assert_eq!(declarations.declaration_map, Some(true));
}

#[test]
fn reversing_siblings_retains_first_branch_own_overrides() {
    let (_temporary, root) = project();
    set_config(&root, |config| {
        config["extends"] = serde_json::json!(["./configs/second.json", "./configs/first.json"])
    });
    assert_eq!(selected(&root), ["src/first/first.ts"]);
    let mut cache = TsconfigInputCache::default();
    assert_eq!(
        cache.load(&root.join("tsconfig.json")).unwrap().allow_js,
        Some(true)
    );
    let declarations = load_tsconfig_declaration_options(&root.join("tsconfig.json"));
    assert_eq!(
        declarations.declaration_dir,
        Some(root.join("configs/../types-first"))
    );
    assert_eq!(
        declarations.out_dir,
        Some(root.join("configs/../dist-first"))
    );
    assert_eq!(declarations.declaration_map, Some(false));
}

#[test]
fn local_empty_selections_and_false_maps_override_both_siblings() {
    let (_temporary, root) = project();
    set_config(&root, |config| {
        config["files"] = serde_json::json!([]);
        config["include"] = serde_json::json!([]);
        config["exclude"] = serde_json::json!([]);
        config["compilerOptions"] = serde_json::json!({"allowJs":false,"declarationMap":false,"declarationDir":"own-types","outDir":"own-dist"});
    });
    assert!(selected(&root).is_empty());
    let mut cache = TsconfigInputCache::default();
    let input = cache.load(&root.join("tsconfig.json")).unwrap();
    assert!(input.has_files && input.has_includes && input.has_excludes);
    assert!(input.files.is_empty() && input.includes.is_empty() && input.excludes.is_empty());
    assert_eq!(input.allow_js, Some(false));
    let declarations = load_tsconfig_declaration_options(&root.join("tsconfig.json"));
    assert_eq!(declarations.declaration_dir, Some(root.join("own-types")));
    assert_eq!(declarations.out_dir, Some(root.join("own-dist")));
    assert_eq!(declarations.declaration_map, Some(false));
}
