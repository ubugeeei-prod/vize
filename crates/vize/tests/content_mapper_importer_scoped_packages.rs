//! Standard-tsgo oracle for importer-scoped package identity (#4002).

#![allow(clippy::disallowed_macros, clippy::disallowed_methods)]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::json;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn install_package(root: &Path, app: &str, prop: &str, ty: &str) {
    let package = format!("apps/{app}/node_modules/@scope/ui");
    write(
        root,
        &format!("{package}/package.json"),
        &format!(
            r##"{{
  "name":"@scope/ui",
  "exports":{{".":{{"oracle":"./src/Selected.vue","types":"./src/Fallback.vue"}}}},
  "imports":{{"#internal":"./src/Internal.vue"}}
}}"##
        ),
    );
    write(
        root,
        &format!("{package}/src/Internal.vue"),
        &format!("<script setup lang=\"ts\">defineProps<{{ {prop}: {ty} }}>()</script>"),
    );
    write(
        root,
        &format!("{package}/src/Selected.vue"),
        &format!(
            r##"<script setup lang="ts">
import Internal from "#internal"
type Private = InstanceType<typeof Internal>["$props"]
const privateProps: Private = {{ {prop}: {} }}
void privateProps
defineProps<{{ {prop}: {ty} }}>()
</script>"##,
            if ty == "string" { "'ok'" } else { "1" }
        ),
    );
    write(
        root,
        &format!("{package}/src/Fallback.vue"),
        "<script setup lang=\"ts\">defineProps<{ fallbackOnly: Date }>()</script>",
    );
}

fn install_mapper_and_vue(root: &Path, vue: &Path) {
    write(
        root,
        "node_modules/vize/package.json",
        &serde_json::to_string_pretty(&json!({
            "name": "vize",
            "private": true,
            "tsContentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "extensions": { ".vue": ".tsx" },
                "compilerOptions": ["noUnusedLocals"]
            }
        }))
        .unwrap(),
    );
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, root.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue, root.join("node_modules/vue")).unwrap();
}

fn run(tsgo: &Path, root: &Path) -> std::process::Output {
    Command::new(tsgo)
        .current_dir(root)
        .args([
            "--loadExternalPlugins",
            "--noEmit",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ])
        .output()
        .unwrap()
}

#[test]
fn standard_tsgo_content_mapper_matches_duplicate_package_identities() {
    let configured = (
        std::env::var_os(TSGO_ENV).map(PathBuf::from),
        std::env::var_os(VUE_ENV).map(PathBuf::from),
    );
    let (tsgo, vue) = match configured {
        (Some(tsgo), Some(vue)) => (tsgo, vue),
        (None, None) => {
            eprintln!("skipping importer package Content Mapper oracle");
            return;
        }
        _ => panic!("{TSGO_ENV} and {VUE_ENV} must be configured together"),
    };
    assert!(tsgo.is_file() && vue.join("package.json").is_file());
    let root = tempfile::tempdir().unwrap();
    install_mapper_and_vue(root.path(), &vue);
    write(
        root.path(),
        "tsconfig.json",
        r#"{
  "compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","customConditions":["oracle"]},
  "contentMappers":[{"package":"vize","extensions":[".vue"]}],
  "include":["apps/*/src/**/*.ts"]
}"#,
    );
    install_package(root.path(), "alpha", "alpha", "string");
    install_package(root.path(), "bravo", "bravo", "number");
    for (app, prop, value) in [("alpha", "alpha", "'ok'"), ("bravo", "bravo", "1")] {
        write(
            root.path(),
            &format!("apps/{app}/src/entry.ts"),
            &format!(
                "import Widget from '@scope/ui'\ntype Props = InstanceType<typeof Widget>['$props']\nexport const props: Props = {{ {prop}: {value} }}\n"
            ),
        );
    }
    let clean = run(&tsgo, root.path());
    assert!(
        clean.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&clean.stdout),
        String::from_utf8_lossy(&clean.stderr)
    );

    write(
        root.path(),
        "apps/bravo/src/entry.ts",
        "import Widget from '@scope/ui'\ntype Props = InstanceType<typeof Widget>['$props']\nexport const props: Props = { alpha: 'wrong' }\n",
    );
    let broken = run(&tsgo, root.path());
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&broken.stdout),
        String::from_utf8_lossy(&broken.stderr)
    );
    assert!(
        !broken.status.success() && output.contains("TS2353"),
        "{output}"
    );
    assert!(output.contains("apps/bravo/src/entry.ts"), "{output}");
    assert!(!output.contains("apps/alpha/src/entry.ts"), "{output}");
}
