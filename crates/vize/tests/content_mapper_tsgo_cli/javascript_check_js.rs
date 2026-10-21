use std::path::{Path, PathBuf};

use serde_json::json;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

fn write(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

fn project() -> tempfile::TempDir {
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-check-js-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    write(
        project.path().join("src/App.vue"),
        r#"<script setup>
import LocalOptions from './LocalOptions.vue'
import LocalSetup from './LocalSetup.vue'
import Widget from 'plain-widget/Widget.vue'
const count = 1
</script>
<template>{{ count.toFixed("bad") }}<LocalOptions /><LocalSetup /><Widget /></template>
"#,
    );
    write(
        project.path().join("src/LocalOptions.vue"),
        r#"<script>
export default {
  data() { return { count: 1 } }
}
</script>
<template>{{ count.toFixed("bad") }}</template>
"#,
    );
    write(
        project.path().join("src/LocalSetup.vue"),
        r#"<script setup>
const count = 1
</script>
<template>{{ count.toFixed("bad") }}</template>
"#,
    );
    write(
        project
            .path()
            .join("src/node_modules/plain-widget/package.json"),
        r#"{"name":"plain-widget","private":true,"main":"index.js"}"#,
    );
    write(
        project
            .path()
            .join("src/node_modules/plain-widget/index.js"),
        "export {};\n",
    );
    write(
        project
            .path()
            .join("src/node_modules/plain-widget/Widget.vue"),
        r#"<script setup>
const count = 1
</script>
<template>{{ count.toFixed("bad") }}</template>
"#,
    );
    write(
        project.path().join("src/shims.d.ts"),
        "declare module 'plain-widget/Widget.vue';\n",
    );
    write(
        project.path().join("src/OptIn.vue"),
        r#"<script setup>
// @ts-check
const count = 1
</script>
<template>{{ count.toFixed("bad") }}</template>
"#,
    );
    for (name, check_js, include) in [
        (
            "tsconfig.json",
            false,
            vec![
                "src/App.vue",
                "src/LocalOptions.vue",
                "src/LocalSetup.vue",
                "src/node_modules/plain-widget/Widget.vue",
                "src/shims.d.ts",
            ],
        ),
        (
            "tsconfig.check.json",
            true,
            vec![
                "src/App.vue",
                "src/LocalOptions.vue",
                "src/LocalSetup.vue",
                "src/node_modules/plain-widget/Widget.vue",
                "src/shims.d.ts",
            ],
        ),
        ("tsconfig.opt-in.json", false, vec!["src/OptIn.vue"]),
    ] {
        write(
            project.path().join(name),
            &serde_json::to_string_pretty(&json!({
                "compilerOptions": {
                    "allowJs": true,
                    "checkJs": check_js,
                    "strict": true,
                    "module": "preserve",
                    "moduleResolution": "bundler",
                    "types": [],
                },
                "contentMappers": [{
                    "package": "vize",
                    "extensions": [".vue"],
                }],
                "include": include,
                "exclude": [],
            }))
            .unwrap(),
        );
    }
    project
}

fn diagnostic_paths<'a>(output: &'a str, code: &str) -> Vec<&'a str> {
    let mut paths = output
        .lines()
        .filter_map(|line| {
            let (location, message) = line.split_once(": error ")?;
            let (actual_code, _) = message.split_once(':')?;
            if actual_code != code {
                return None;
            }
            location.rsplit_once('(').map(|(path, _)| path)
        })
        .collect::<Vec<_>>();
    paths.sort_unstable();
    paths
}

#[test]
fn plain_javascript_sfc_diagnostics_follow_check_js_including_node_modules() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );
    let project = project();

    let unchecked = check_project(&tsgo, project.path(), "tsconfig.json");
    assert!(unchecked.status.success(), "{}", output_text(&unchecked));

    let checked = check_project(&tsgo, project.path(), "tsconfig.check.json");
    let checked_text = output_text(&checked);
    assert!(!checked.status.success(), "{checked_text}");
    assert_eq!(
        diagnostic_paths(&checked_text, "TS2345"),
        [
            "src/App.vue",
            "src/LocalOptions.vue",
            "src/LocalSetup.vue",
            "src/node_modules/plain-widget/Widget.vue",
        ],
        "checkJs diagnostics did not cover the complete local and package set:\n{checked_text}"
    );

    let opted_in = check_project(&tsgo, project.path(), "tsconfig.opt-in.json");
    let opted_in_text = output_text(&opted_in);
    assert!(!opted_in.status.success(), "{opted_in_text}");
    assert_eq!(
        diagnostic_paths(&opted_in_text, "TS2345"),
        ["src/OptIn.vue"],
        "a leading // @ts-check did not opt the JavaScript SFC in:\n{opted_in_text}"
    );
}
