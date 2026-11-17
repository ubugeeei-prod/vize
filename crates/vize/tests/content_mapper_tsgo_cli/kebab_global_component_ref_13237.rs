//! #13237: a ref on a kebab-case `GlobalComponents` tag holds the component.
#![expect(clippy::disallowed_macros, reason = "fixture paths use std strings")]
#![expect(clippy::disallowed_types, reason = "fixture IO uses std strings")]
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

use std::path::{Path, PathBuf};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const FIXTURE: &str = "tests/_fixtures/differential/typechecker/kebab-global-component-ref-13237";

fn install(project: &Path, names: &[&str]) {
    let fixture = workspace_root().join(FIXTURE);
    for name in names {
        let target = project.join(name);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(format!("{name}.txt")), target).unwrap();
    }
}

fn replace(project: &Path, name: &str, from: &str, to: &str) {
    let path = project.join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let probe = source.replace(from, to);
    assert_ne!(probe, source, "{name}: {from}");
    std::fs::write(path, probe).unwrap();
}

#[test]
fn kebab_global_component_refs_hold_the_component_instance() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping kebab global component refs: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-kebab-global-ref-")
        .tempdir_in(&cases_root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    // The exact reported project, plus `v-for`, function and Pug refs and an
    // unregistered custom element.
    install(
        project,
        &[
            "tsconfig.json",
            "src/components.d.ts",
            "src/child.vue",
            "src/parent.vue",
            "src/forms.vue",
            "src/pug.vue",
        ],
    );
    let output = check_project(&native, project, "tsconfig.json");
    assert!(output.status.success(), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));

    // The refs are typed, not `any`: a missing exposed member reports on
    // every kebab spelling, and the unregistered tag is still an `Element`.
    for name in ["src/parent.vue", "src/pug.vue"] {
        replace(
            project,
            name,
            "kebab.value?.focus()",
            "kebab.value?.__vizeMissing",
        );
    }
    replace(
        project,
        "src/forms.vue",
        "looped.value?.[0]?.focus()",
        "looped.value?.[0]?.__vizeMissing",
    );
    replace(
        project,
        "src/forms.vue",
        "element.value?.getAttribute(\"id\")",
        "element.value?.focus()",
    );
    let output = check_project(&native, project, "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "src/forms.vue(7,20): error TS2339: Property '__vizeMissing' does not exist on type '$ComponentInstance'.\n\
         src/forms.vue(8,16): error TS2339: Property 'focus' does not exist on type 'Element'.\n\
         src/parent.vue(8,14): error TS2339: Property '__vizeMissing' does not exist on type '$ComponentInstance'.\n\
         src/pug.vue(6,14): error TS2339: Property '__vizeMissing' does not exist on type '$ComponentInstance'.\n",
        "{}",
        output_text(&output)
    );
}

/// Options API `$refs` reads the same registry when the project types it.
#[test]
fn kebab_global_component_dollar_refs_hold_the_component_instance() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping kebab global component $refs: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-kebab-global-dollar-refs-")
        .tempdir_in(&cases_root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    install(
        project,
        &[
            "tsconfig.json",
            "src/components.d.ts",
            "src/child.vue",
            "src/options.vue",
        ],
    );
    let config_path = project.join("tsconfig.json");
    let mut config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
    config["vueCompilerOptions"] = serde_json::json!({ "inferTemplateDollarRefs": true });
    std::fs::write(config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

    let output = check_project(&native, project, "tsconfig.json");
    assert!(output.status.success(), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));

    replace(
        project,
        "src/options.vue",
        "$refs.kebab?.focus()",
        "$refs.kebab?.__vizeMissing",
    );
    let output = check_project(&native, project, "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "src/options.vue(10,55): error TS2339: Property '__vizeMissing' does not exist on type '$ComponentInstance'.\n",
        "{}",
        output_text(&output)
    );
}
