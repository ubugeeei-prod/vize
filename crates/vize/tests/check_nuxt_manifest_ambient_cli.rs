//! #13093: a Nuxt import manifest the tsconfig selects stays a program root
//! for `vize check` unless Nuxt detection supplies its declarations.
#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use std::{path::Path, process::Command};

use crate::check_canon_recent_issues_cli::{
    link_workspace_vue, resolve_test_corsa_path, unique_case_dir, workspace_root,
};
use crate::support::corsa_requirement;

const FIXTURE: &str = "tests/_fixtures/differential/typechecker/check-nuxt-manifest-ambient-13093";
const REPORTED_SELECTION: &str = r#""include": ["src/**/*.vue", ".nuxt/types/imports.d.ts"]"#;

fn create_project(name: &str, manifest: &str, selection: &str, nuxt: bool) -> std::path::PathBuf {
    let fixture = workspace_root().join(FIXTURE);
    let project = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project);
    std::fs::create_dir_all(project.join("src")).unwrap();
    link_workspace_vue(&project).unwrap();
    for file in ["form", "child", "parent"] {
        std::fs::copy(
            fixture.join(vize_l0::cstr!("src/{file}.vue.txt").as_str()),
            project.join(vize_l0::cstr!("src/{file}.vue").as_str()),
        )
        .unwrap();
    }
    let target = project.join(manifest);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::copy(fixture.join("imports.d.ts.txt"), target).unwrap();
    // The pinned reported tsconfig, with its selection of the manifest varied.
    let reported = std::fs::read_to_string(fixture.join("tsconfig.json.txt")).unwrap();
    let selected = reported.replace(REPORTED_SELECTION, selection);
    assert_eq!(reported.matches(REPORTED_SELECTION).count(), 1);
    std::fs::write(project.join("tsconfig.json"), selected).unwrap();
    if nuxt {
        std::fs::write(project.join("nuxt.config.ts"), "export default {}\n").unwrap();
    }
    project
}

fn diagnostics(project: &Path, corsa_path: &Path) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!(
            "{error}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| {
                    let text = diagnostic.as_str().unwrap();
                    format!("{name} {}", text.split(" (source:").next().unwrap())
                })
        })
        .collect()
}

#[test]
fn selected_nuxt_import_manifests_declare_their_globals_with_or_without_nuxt_detection() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    for (index, (manifest, selection, nuxt)) in [
        (
            ".nuxt/types/imports.d.ts",
            r#""include":["src/**/*.vue",".nuxt/types/imports.d.ts"]"#,
            false,
        ),
        (
            ".nuxt/imports.d.ts",
            r#""include":["src/**/*.vue",".nuxt/imports.d.ts"]"#,
            false,
        ),
        (
            ".nuxt/types/imports.d.ts",
            r#""files":[".nuxt/types/imports.d.ts"],"include":["src/**/*.vue"]"#,
            false,
        ),
        // A detected Nuxt project supplies the manifest as its own stubs,
        // which still own `useTemplateRef`.
        (
            ".nuxt/types/imports.d.ts",
            r#""include":["src/**/*.vue",".nuxt/types/imports.d.ts"]"#,
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let project = create_project(
            vize_l0::cstr!("nuxt-manifest-ambient-{index}").as_str(),
            manifest,
            selection,
            nuxt,
        );
        assert_eq!(
            diagnostics(&project, &corsa_path),
            Vec::<String>::new(),
            "{manifest} via {selection}"
        );
        // The refs are typed by Vue's composable, not `any`.
        for (file, from, to) in [
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
            let path = project.join(file);
            let source = std::fs::read_to_string(&path).unwrap();
            std::fs::write(&path, source.replace(from, to)).unwrap();
        }
        let reported = diagnostics(&project, &corsa_path);
        assert_eq!(
            reported.len(),
            2,
            "{manifest} via {selection}: {reported:?}"
        );
        assert_eq!(
            reported
                .iter()
                .map(|line| line.split(' ').take(3).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>(),
            vec![
                "src/form.vue error:5:37 [TS2554]".to_owned(),
                "src/parent.vue error:7:16 [TS2339]".to_owned(),
            ],
            "{manifest} via {selection}: {reported:?}"
        );
        let _ = std::fs::remove_dir_all(&project);
    }

    // A manifest the tsconfig never selects is not part of the program.
    let project = create_project(
        "nuxt-manifest-ambient-unselected",
        ".nuxt/types/imports.d.ts",
        r#""include":["src/**/*.vue"]"#,
        false,
    );
    let reported = diagnostics(&project, &corsa_path);
    assert_eq!(
        reported
            .iter()
            .map(|line| line.split(' ').take(3).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>(),
        vec![
            "src/form.vue error:2:14 [TS2304]".to_owned(),
            "src/parent.vue error:4:15 [TS2304]".to_owned(),
        ],
        "{reported:?}"
    );
    let _ = std::fs::remove_dir_all(&project);
}

const REFERENCED_FIXTURE: &str =
    "tests/_fixtures/differential/typechecker/nuxt-referenced-import-manifest-template-ref";

/// Nuxt 4's default layout: the root tsconfig only references
/// `.nuxt/tsconfig.app.json`, which includes `nuxt.d.ts`, and the import
/// manifest is reached solely through that file's `/// <reference path>`.
fn create_referenced_manifest_project(name: &str) -> std::path::PathBuf {
    let fixture = workspace_root().join(REFERENCED_FIXTURE);
    let project = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project);
    link_workspace_vue(&project).unwrap();
    for (input, target) in [
        ("tsconfig.json.txt", "tsconfig.json"),
        ("tsconfig.app.json.txt", ".nuxt/tsconfig.app.json"),
        ("nuxt.d.ts.txt", ".nuxt/nuxt.d.ts"),
        ("imports.d.ts.txt", ".nuxt/types/imports.d.ts"),
        ("app/pages/index.vue.txt", "app/pages/index.vue"),
    ] {
        let target = project.join(target);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(input), target).unwrap();
    }
    std::fs::write(project.join("nuxt.config.ts"), "export default {}\n").unwrap();
    project
}

#[test]
fn a_manifest_reached_only_through_nuxt_references_owns_use_template_ref() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = create_referenced_manifest_project("nuxt-referenced-manifest-template-ref");
    assert_eq!(diagnostics(&project, &corsa_path), Vec::<String>::new());

    // The ref is the registered `<main>`, not `unknown`: an unknown member of
    // the element and a key the template never registers are both reported.
    let page = project.join("app/pages/index.vue");
    let source = std::fs::read_to_string(&page).unwrap();
    let edited = source
        .replace("el.value?.tagName", "el.value?.__vizeMissingMember")
        .replace(
            "const el = useTemplateRef(\"el\")",
            "const el = useTemplateRef(\"el\")\nconst missing = useTemplateRef(\"missing\")",
        );
    assert_ne!(edited, source);
    std::fs::write(&page, edited).unwrap();
    let reported = diagnostics(&project, &corsa_path);
    assert_eq!(
        reported
            .iter()
            .map(|line| line.split(" (").next().unwrap().to_owned())
            .collect::<Vec<_>>(),
        vec![
            "app/pages/index.vue error:3:32 [TS2339] Property 'missing' does not exist on type '$TemplateRefs'.".to_owned(),
            "app/pages/index.vue error:6:25 [TS2339] Property '__vizeMissingMember' does not exist on type 'HTMLElement'.".to_owned(),
        ],
        "{reported:?}"
    );
    let _ = std::fs::remove_dir_all(&project);
}
