//! #13237 through `vize check`: a ref on a kebab-case `GlobalComponents` tag
//! holds the component instance, like the PascalCase tag.
#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::check_canon_recent_issues_cli::{
    link_workspace_vue, resolve_test_corsa_path, unique_case_dir, workspace_root,
};
use crate::check_registered_components_cli::copy_pinned_project;
use crate::support::corsa_requirement;

use std::{path::Path, process::Command};

use vize_l0::cstr;

/// The pinned #13237 project; `content_mapper_tsgo_cli` checks the same bytes.
const FIXTURE: &str = "tests/_fixtures/differential/typechecker/kebab-global-component-ref-13237";

fn check_rows(project_root: &Path, corsa_path: &Path) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON report: {error}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let mut rows: Vec<String> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(move |row| cstr!("{name} {row}").to_string())
        })
        .collect();
    rows.sort_unstable();
    rows
}

fn replace(project_root: &Path, name: &str, from: &str, to: &str) {
    let path = project_root.join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let probe = source.replace(from, to);
    assert_ne!(probe, source, "{name}: {from}");
    std::fs::write(path, probe).unwrap();
}

#[test]
fn check_types_kebab_global_component_refs_as_the_component() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("kebab-global-component-ref-13237");
    let _ = std::fs::remove_dir_all(&project_root);
    copy_pinned_project(&workspace_root().join(FIXTURE), &project_root);
    link_workspace_vue(&project_root).unwrap();
    // `$refs` is typed only when the project asks for it.
    let config_path = project_root.join("tsconfig.json");
    let mut config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
    config["vueCompilerOptions"] = serde_json::json!({ "inferTemplateDollarRefs": true });
    std::fs::write(&config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

    assert_eq!(check_rows(&project_root, &corsa_path), Vec::<String>::new());

    for name in ["src/parent.vue", "src/pug.vue"] {
        replace(
            &project_root,
            name,
            "kebab.value?.focus()",
            "kebab.value?.__vizeMissing",
        );
    }
    replace(
        &project_root,
        "src/forms.vue",
        "looped.value?.[0]?.focus()",
        "looped.value?.[0]?.__vizeMissing",
    );
    replace(
        &project_root,
        "src/forms.vue",
        "element.value?.getAttribute(\"id\")",
        "element.value?.focus()",
    );
    replace(
        &project_root,
        "src/options.vue",
        "$refs.kebab?.focus()",
        "$refs.kebab?.__vizeMissing",
    );
    assert_eq!(
        check_rows(&project_root, &corsa_path),
        vec![
            "src/forms.vue error:7:20 [TS2339] Property '__vizeMissing' does not exist on type '$ComponentInstance'.".to_owned(),
            "src/forms.vue error:8:16 [TS2339] Property 'focus' does not exist on type 'Element'.".to_owned(),
            "src/options.vue error:10:55 [TS2339] Property '__vizeMissing' does not exist on type '$ComponentInstance'. (source: <button @click=\"$refs.pascal?.focus(); $refs.kebab?.__vizeMissing\" />; binding: @click)".to_owned(),
            "src/parent.vue error:8:14 [TS2339] Property '__vizeMissing' does not exist on type '$ComponentInstance'.".to_owned(),
            "src/pug.vue error:6:14 [TS2339] Property '__vizeMissing' does not exist on type '$ComponentInstance'.".to_owned(),
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
