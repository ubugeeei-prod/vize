//! Required-flag trivia must stay correct in both real checker entry points.

use std::{path::PathBuf, process::Command};

use serde_json::{Value, json};
use vize_l0::cstr;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text,
};

#[test]
fn optional_prop_trivia_keeps_checker_and_mapper_contracts() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact optional prop spacing: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name the real host");
    for newline in ["\n", "\r\n"] {
        let project = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(project.path().join("src")).unwrap();
        std::fs::create_dir_all(project.path().join("node_modules")).unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        std::fs::write(
            project.path().join("tsconfig.json"),
            serde_json::to_vec_pretty(&json!({
                "compilerOptions": {"strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true, "skipLibCheck": true},
                "include": ["src/**/*.vue"]
            })).unwrap(),
        ).unwrap();
        std::fs::write(
            project.path().join("tsconfig.mapper.json"),
            serde_json::to_vec_pretty(&json!({
                "extends": "./tsconfig.json",
                "contentMappers": [{"package": "vize", "extensions": [".vue"]}]
            }))
            .unwrap(),
        )
        .unwrap();
        let child = r#"<script setup lang="ts">
defineProps<{
  spaced? : string;
  tabbed?	: string;
  commented? /* question */ : string;
  compact?: string;
  readonly read? : string;
  "scope.name"? : string;
  café? : string;
  required : number;
}>()
</script>
<template><div>{{ required }}</div></template>
"#;
        std::fs::write(
            project.path().join("src/Child.vue"),
            child.replace('\n', newline),
        )
        .unwrap();
        for (template, invalid) in [
            ("<Child :required=\"1\" />", false),
            ("<Child required=\"wrong\" />", true),
        ] {
            let app = cstr!(
                "<script setup lang=\"ts\">\nimport Child from './Child.vue'\n</script>\n<template>\n  {template}\n</template>\n"
            );
            std::fs::write(
                project.path().join("src/App.vue"),
                app.replace('\n', newline),
            )
            .unwrap();
            let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .env("CORSA_PATH", &tsgo)
                .args([
                    "check",
                    "--no-config",
                    "--tsconfig",
                    "tsconfig.json",
                    "--format",
                    "json",
                ])
                .output()
                .unwrap();
            let packet: Value = serde_json::from_slice(&cli.stdout).unwrap();
            assert_eq!(
                cli.status.code(),
                Some(i32::from(invalid)),
                "{}",
                output_text(&cli)
            );
            assert_eq!(packet["warningCount"], 0);
            assert_eq!(packet["errorCount"], usize::from(invalid));
            let files = packet["files"].as_array().unwrap();
            let [app, child] = files.as_slice() else {
                panic!("{packet}")
            };
            assert_eq!(child, &json!({"file": "src/Child.vue", "diagnostics": []}));
            if invalid {
                assert_eq!(
                    app,
                    &json!({"file": "src/App.vue", "diagnostics": ["error:5:10 [TS2322] Type 'string' is not assignable to type 'number'. (source: <Child required=\"wrong\" />; binding: required)"]})
                );
            } else {
                assert_eq!(app, &json!({"file": "src/App.vue", "diagnostics": []}));
            }
            let mapper = check_project(&tsgo, project.path(), "tsconfig.mapper.json");
            assert_eq!(mapper.stderr, b"", "{}", output_text(&mapper));
            if invalid {
                assert_eq!(mapper.status.code(), Some(2), "{}", output_text(&mapper));
                let diagnostics = std::str::from_utf8(&mapper.stdout)
                    .unwrap()
                    .lines()
                    .collect::<Vec<_>>();
                assert_eq!(
                    diagnostics,
                    [
                        "src/App.vue(5,10): error TS2322: Type 'string' is not assignable to type 'number'."
                    ],
                    "{}",
                    output_text(&mapper)
                );
            } else {
                assert_success(&mapper);
                assert_eq!(mapper.stdout, b"", "{}", output_text(&mapper));
            }
        }
    }
}
