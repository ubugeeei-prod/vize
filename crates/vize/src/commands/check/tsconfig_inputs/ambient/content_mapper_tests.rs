//! #13093: Nuxt import manifests a tsconfig selects decide mapper ownership.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]
use std::path::Path;

use super::collect_content_mapper_ambient_declarations;
use crate::commands::check::tsconfig_inputs::{
    TsconfigInputCache, declares_vue_template_ref_global,
};

const VUE_TEMPLATE_REF: &str = "export {}\ndeclare global {\n  const useTemplateRef: typeof import(\"vue\").useTemplateRef\n}\n";

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn collect(root: &Path, tsconfig: &str) -> Vec<std::path::PathBuf> {
    write(root, "tsconfig.json", tsconfig);
    collect_content_mapper_ambient_declarations(
        root,
        &root.join("tsconfig.json"),
        &mut TsconfigInputCache::default(),
    )
}

#[test]
fn every_selected_nuxt_import_manifest_owns_the_vue_template_ref() {
    for manifest in [".nuxt/imports.d.ts", ".nuxt/types/imports.d.ts"] {
        let reference = manifest.strip_prefix(".nuxt/").unwrap();
        for (form, tsconfig) in [
            ("files", vize_l0::cstr!(r#"{{"files":["{manifest}"]}}"#)),
            (
                "literal include",
                vize_l0::cstr!(r#"{{"include":["src/**/*.vue","{manifest}"]}}"#),
            ),
            (
                "wildcard include",
                vize_l0::cstr!(r#"{{"include":["src/**/*.vue",".nuxt/**/*.d.ts"]}}"#),
            ),
            (
                "nuxt.d.ts reference",
                vize_l0::cstr!(r#"{{"include":["src/**/*.vue",".nuxt/nuxt.d.ts"]}}"#),
            ),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            write(&root, manifest, VUE_TEMPLATE_REF);
            write(
                &root,
                ".nuxt/nuxt.d.ts",
                &vize_l0::cstr!("/// <reference path=\"{reference}\" />\nexport {{}}\n"),
            );
            write(&root, "src/form.vue", "<template><form /></template>\n");

            let files = collect(&root, &tsconfig);
            assert_eq!(
                files
                    .iter()
                    .filter(|path| **path == root.join(manifest))
                    .count(),
                1,
                "{manifest} via {form}: {files:?}"
            );
            assert!(
                declares_vue_template_ref_global(&files),
                "{manifest} via {form}: {files:?}"
            );
        }
    }
}

#[test]
fn unselected_or_excluded_nuxt_import_manifests_do_not_own_the_vue_template_ref() {
    for tsconfig in [
        r#"{"include":["src/**/*.vue"]}"#,
        r#"{"include":["src/**/*.vue",".nuxt/types/*.d.ts"],"exclude":[".nuxt/types/imports.d.ts"]}"#,
        r#"{"include":["src/**/*.vue",".nuxt/imports.d.ts"]}"#,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        write(&root, ".nuxt/types/imports.d.ts", VUE_TEMPLATE_REF);
        write(&root, "src/form.vue", "<template><form /></template>\n");

        let files = collect(&root, tsconfig);
        assert_eq!(
            files
                .iter()
                .filter(|path| **path == root.join(".nuxt/types/imports.d.ts"))
                .count(),
            0,
            "{tsconfig}: {files:?}"
        );
        assert!(
            !declares_vue_template_ref_global(&files),
            "{tsconfig}: {files:?}"
        );
    }
}

/// A solution-style root (Nuxt's generated `tsconfig.json`) expands into its
/// references exactly as input collection does, so a manifest a referenced
/// project selects owns the ref. A root with its own inputs does not.
#[test]
fn referenced_projects_select_nuxt_import_manifests_like_input_collection() {
    for (root_config, owned) in [
        (
            r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
            true,
        ),
        (
            r#"{"include":["src/**/*.vue"],"references":[{"path":"./tsconfig.app.json"}]}"#,
            false,
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        write(&root, ".nuxt/types/imports.d.ts", VUE_TEMPLATE_REF);
        write(&root, "src/form.vue", "<template><form /></template>\n");
        write(
            &root,
            "tsconfig.app.json",
            r#"{"files":[".nuxt/types/imports.d.ts"],"include":["src/**/*.vue"]}"#,
        );

        let files = collect(&root, root_config);
        assert_eq!(
            files
                .iter()
                .filter(|path| **path == root.join(".nuxt/types/imports.d.ts"))
                .count(),
            usize::from(owned),
            "{root_config}: {files:?}"
        );
        assert_eq!(
            declares_vue_template_ref_global(&files),
            owned,
            "{root_config}: {files:?}"
        );
    }
}
