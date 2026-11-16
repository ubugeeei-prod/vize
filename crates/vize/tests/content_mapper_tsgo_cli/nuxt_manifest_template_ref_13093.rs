//! #13093: Nuxt's generated import manifest owns `useTemplateRef` for the mapper.
#![expect(clippy::disallowed_macros, reason = "fixture paths use std strings")]
#![expect(clippy::disallowed_types, reason = "fixture IO uses std strings")]
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

use std::path::{Path, PathBuf};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const FIXTURE: &str =
    "tests/_fixtures/differential/typechecker/content-mapper-nuxt-template-ref-13093";

fn install_case(project: &Path, manifest: &str, selection: &str) {
    let fixture = workspace_root().join(FIXTURE);
    install_mapper_manifest(project);
    install_vue_package(project);
    for name in ["src/form.vue", "src/child.vue", "src/parent.vue"] {
        let target = project.join(name);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(format!("{name}.txt")), target).unwrap();
    }
    let target = project.join(manifest);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::copy(fixture.join("imports.d.ts.txt"), target).unwrap();
    let reference = manifest.strip_prefix(".nuxt/").unwrap();
    std::fs::write(
        project.join(".nuxt/nuxt.d.ts"),
        format!("/// <reference path=\"{reference}\" />\nexport {{}}\n"),
    )
    .unwrap();
    let tsconfig = std::fs::read_to_string(fixture.join("tsconfig.json.txt")).unwrap();
    let selected = tsconfig.replace("\".nuxt/types/imports.d.ts\"", &format!("\"{selection}\""));
    assert_eq!(
        selected.matches(selection).count(),
        1,
        "the reported include entry is replaced exactly"
    );
    std::fs::write(project.join("tsconfig.json"), selected).unwrap();
}

#[test]
fn nuxt_import_manifests_type_dom_and_component_template_refs() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Nuxt manifest conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    for manifest in [".nuxt/types/imports.d.ts", ".nuxt/imports.d.ts"] {
        for selection in [manifest, ".nuxt/nuxt.d.ts"] {
            let project = tempfile::Builder::new()
                .prefix("content-mapper-nuxt-template-ref-")
                .tempdir_in(&cases_root)
                .unwrap();
            install_case(project.path(), manifest, selection);

            let output = check_project(&native, project.path(), "tsconfig.json");
            assert!(
                output.status.success(),
                "{manifest} via {selection}: {}",
                output_text(&output)
            );
            assert_eq!(output.stdout, b"", "{}", output_text(&output));
            assert_eq!(output.stderr, b"", "{}", output_text(&output));

            // The refs are typed, not `any`: a wrong call on the element
            // (`HTMLFormElement` has a string index signature, so a missing
            // member would not prove it) and a missing exposed member report.
            for (name, from, to) in [
                (
                    "src/form.vue",
                    "form.value?.reportValidity()",
                    "form.value?.reportValidity(1)",
                ),
                (
                    "src/parent.vue",
                    "child.value?.focus()",
                    "child.value?.__vizeMissingComponent",
                ),
            ] {
                let path = project.path().join(name);
                let source = std::fs::read_to_string(&path).unwrap();
                let probe = source.replace(from, to);
                assert_ne!(probe, source);
                std::fs::write(path, probe).unwrap();
            }
            let output = check_project(&native, project.path(), "tsconfig.json");
            assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(
                stdout,
                "src/form.vue(5,37): error TS2554: Expected 0 arguments, but got 1.\n\
                 src/parent.vue(7,16): error TS2339: Property '__vizeMissingComponent' does not exist on type '$ComponentInstance'.\n",
                "{manifest} via {selection}: {}",
                output_text(&output)
            );
        }
    }
}
