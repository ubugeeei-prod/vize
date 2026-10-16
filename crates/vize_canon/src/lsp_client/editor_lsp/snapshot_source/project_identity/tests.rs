use super::*;
use serde_json::json;
use std::path::PathBuf;

fn config() -> PathBuf {
    std::env::temp_dir()
        .join("vize-snapshot-project")
        .join("tsconfig.json")
}
fn foreign_config() -> PathBuf {
    std::env::temp_dir()
        .join("vize-foreign-project")
        .join("tsconfig.json")
}

fn descriptor() -> ProjectResponse {
    serde_json::from_value(json!({"id":"native-project-1","configFileName":config(),
        "compilerOptions":{"strict":true,"paths":{"@/*":["./src/*"]}},
        "rootFiles":["/workspace/a.ts","/workspace/b.ts"]}))
    .unwrap()
}

#[test]
fn matches_complete_native_descriptor_behind_unrelated_project() {
    let selected = descriptor();
    let mut foreign = selected.clone();
    foreign.id = "native-project-2".into();
    foreign.config_file_name = foreign_config().to_str().unwrap().into();
    let projects = [foreign, selected.clone()];
    let admitted = matching_project(&projects, &config(), &selected).unwrap();
    assert!(
        std::ptr::eq(admitted, &projects[1]),
        "native snapshot owns the selected authority"
    );
}

#[test]
fn refuses_absent_foreign_duplicate_and_every_changed_typed_descriptor_field() {
    let selected = descriptor();
    let config = config();
    let config = config.as_path();
    assert!(matching_project(&[], config, &selected).is_none());
    let foreign = foreign_config();
    let aliased = config.parent().unwrap().join(".").join("tsconfig.json");
    let nul = vize_l0::cstr!("{}\0", config.display());
    for path in [
        Path::new("tsconfig.json"),
        foreign.as_path(),
        aliased.as_path(),
        Path::new(nul.as_str()),
    ] {
        assert!(matching_project(std::slice::from_ref(&selected), path, &selected).is_none());
    }
    let mut changes = Vec::new();
    let mut changed = selected.clone();
    changed.id = "foreign-handle".into();
    changes.push(changed);
    let mut changed = selected.clone();
    changed.config_file_name = foreign_config().to_str().unwrap().into();
    changes.push(changed);
    let mut changed = selected.clone();
    changed.compiler_options["strict"] = json!(false);
    changes.push(changed);
    let mut changed = selected.clone();
    changed.compiler_options["paths"]["@/*"] = json!(["./foreign/*"]);
    changes.push(changed);
    let mut changed = selected.clone();
    changed.root_files.reverse();
    changes.push(changed);
    let mut changed = selected.clone();
    changed.root_files.pop();
    changes.push(changed);
    for changed in changes {
        assert!(matching_project(std::slice::from_ref(&selected), config, &changed).is_none());
    }
    assert!(matching_project(&[selected.clone(), selected.clone()], config, &selected).is_none());
    let mut duplicate_config = selected.clone();
    duplicate_config.id = "other-handle".into();
    assert!(matching_project(&[selected.clone(), duplicate_config], config, &selected).is_none());
    let mut duplicate_handle = selected.clone();
    duplicate_handle.config_file_name = foreign_config().to_str().unwrap().into();
    assert!(matching_project(&[selected.clone(), duplicate_handle], config, &selected).is_none());
}

#[test]
fn refuses_empty_native_project_handle_even_when_descriptor_matches() {
    let mut selected = descriptor();
    selected.id = "".into();
    assert!(matching_project(std::slice::from_ref(&selected), &config(), &selected).is_none());
}
