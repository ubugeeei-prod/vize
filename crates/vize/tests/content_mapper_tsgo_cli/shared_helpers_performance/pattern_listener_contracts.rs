use super::*;
use sha2::{Digest, Sha256};

#[test]
fn official_pattern_listener_control_matches_pinned_reporter_inputs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/content-mapper/nuxt-pattern-emit-10195.json"
    ))
    .unwrap();
    assert_eq!(fixture["typescript"], "7.1.0-dev.20261017.1");
    assert_eq!(fixture["vue"], "3.5.43");
    let control = fixture["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "pattern-injections-keep-listener-errors")
        .unwrap();
    assert_eq!(control["codes"], json!([2322]));
    assert_eq!(control["status"], 2);

    for (file, pinned, expected_sha256) in [
        (
            "tsconfig.json",
            include_str!(
                "../../../../../tests/_fixtures/differential/typechecker/content-mapper-nuxt-emit-10195/tsconfig.json.txt"
            ),
            "bed6b6f5f33d8a7d30e8f6c3585d833486637812a78150f3a177ab2009ae5175",
        ),
        (
            "src/augment.d.ts",
            include_str!(
                "../../../../../tests/_fixtures/differential/typechecker/content-mapper-nuxt-emit-10195/src/augment.d.ts.txt"
            ),
            "72824330228b07923b47dea72f5762eebe50bfe8dd9fe09204e85dcf2cd8302d",
        ),
        (
            "src/MyItem.vue",
            include_str!(
                "../../../../../tests/_fixtures/differential/typechecker/content-mapper-nuxt-emit-10195/src/MyItem.vue.txt"
            ),
            "158ba53462766943952897e3a358d974ef10ae2f9b673b8cc48caeb44fa9e4de",
        ),
        (
            "src/App.vue",
            include_str!(
                "../../../../../tests/_fixtures/differential/typechecker/content-mapper-nuxt-emit-10195/src/App.invalid.vue.txt"
            ),
            "9dc4d658c634bd3ca1947857f0213117d2aa4e9b3c6f4125807e075859ac202d",
        ),
    ] {
        let actual = control["files"][file].as_str().unwrap();
        assert_eq!(actual, pinned, "official control drifted at {file}");
        let actual_sha256 = Sha256::digest(actual.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(
            actual_sha256, expected_sha256,
            "pinned digest drifted at {file}"
        );
    }
}

#[test]
fn standard_tsgo_preserves_pattern_listener_and_open_root_contracts() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping shared-helper listener receipt: {TSGO_ENV} is not set");
        return;
    };
    assert_eq!(
        tsgo.is_file(),
        true,
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-pattern-listener-")
        .tempdir_in(&cases_root)
        .unwrap();
    for shared_helpers in [false, true] {
        for (owner, declared_listener, root, fallthrough_attributes, vue_target, invalid_status) in [
            (
                "fallthrough-omitted",
                false,
                ListenerRoot::Native,
                None,
                None,
                2,
            ),
            (
                "fallthrough-true",
                false,
                ListenerRoot::Native,
                Some(true),
                None,
                2,
            ),
            (
                "fallthrough-false",
                false,
                ListenerRoot::Native,
                Some(false),
                None,
                0,
            ),
            ("declared", true, ListenerRoot::Native, Some(false), None, 2),
            (
                "no-root-open",
                false,
                ListenerRoot::NoTemplate,
                None,
                None,
                0,
            ),
            (
                "unresolved-root-open",
                false,
                ListenerRoot::Unresolved,
                None,
                None,
                0,
            ),
            (
                "component-root-open",
                false,
                ListenerRoot::Resolved,
                None,
                None,
                0,
            ),
            (
                "target-v2-open",
                false,
                ListenerRoot::Native,
                None,
                Some(json!(2)),
                0,
            ),
            (
                "target-v2-7-open",
                false,
                ListenerRoot::Native,
                None,
                Some(json!(2.7)),
                0,
            ),
        ] {
            for valid_listener in [false, true] {
                let mode = if shared_helpers { "shared" } else { "embedded" };
                let case = if valid_listener { "valid" } else { "invalid" };
                let project = receipt.path().join(cstr!("{mode}-{owner}-{case}").as_str());
                prepare_pattern_listener_project(
                    &project,
                    shared_helpers,
                    declared_listener,
                    root,
                    valid_listener,
                    fallthrough_attributes,
                    vue_target.clone(),
                );
                let output = check_project(&tsgo, &project, "tsconfig.json");
                assert_eq!(output.stderr, b"", "{}", output_text(&output));
                let stdout = std::str::from_utf8(&output.stdout).unwrap();
                if valid_listener || invalid_status == 0 {
                    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
                    assert_eq!(stdout, "");
                    continue;
                }

                assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
                let diagnostics = stdout
                    .lines()
                    .filter(|line| line.contains(": error TS"))
                    .collect::<Vec<_>>();
                assert_eq!(diagnostics.len(), 1, "{}", output_text(&output));
                let (identity, _) = diagnostics[0]
                    .split_once(": error TS2322: ")
                    .expect("one TS2322 listener diagnostic");
                assert_eq!(identity, "src/App.vue(6,26)");
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ListenerRoot {
    Native,
    NoTemplate,
    Unresolved,
    Resolved,
}

fn prepare_pattern_listener_project(
    project: &Path,
    shared_helpers: bool,
    declared_listener: bool,
    root: ListenerRoot,
    valid_listener: bool,
    fallthrough_attributes: Option<bool>,
    vue_target: Option<serde_json::Value>,
) {
    prepare_mapper_project(project, shared_helpers);
    let source_root = project.join("src");
    std::fs::create_dir_all(&source_root).unwrap();
    std::fs::write(
        source_root.join("augment.d.ts"),
        r#"export {};

type Decorate<T extends Record<string, any>> = {
  [K in keyof T as K extends string ? `$${K}` : never]: T[K];
};

declare module "vue" {
  interface ComponentCustomProperties extends Decorate<Record<string, unknown>> {}
}
"#,
    )
    .unwrap();
    let emits = if declared_listener {
        "click: [event: MouseEvent]"
    } else {
        "change: []"
    };
    let (root_import, template) = match root {
        ListenerRoot::Native => (
            "",
            "\n<template>\n  <div class=\"my-item\"><slot></slot></div>\n</template>\n",
        ),
        ListenerRoot::NoTemplate => ("", ""),
        ListenerRoot::Unresolved => ("", "\n<template>\n  <UnresolvedRoot />\n</template>\n"),
        ListenerRoot::Resolved => {
            std::fs::write(
                source_root.join("BaseRoot.vue"),
                "<script setup lang=\"ts\"></script>\n<template><div /></template>\n",
            )
            .unwrap();
            (
                "import BaseRoot from \"./BaseRoot.vue\";\n",
                "\n<template>\n  <BaseRoot />\n</template>\n",
            )
        }
    };
    std::fs::write(
        source_root.join("MyItem.vue"),
        cstr!(
            "<script setup lang=\"ts\">\n{root_import}defineEmits<{{ {emits} }}>();\n</script>\n{template}"
        ),
    )
    .unwrap();
    let (listener, body) = if valid_listener {
        ("MouseEvent", "event.button")
    } else {
        ("string", "event.length")
    };
    std::fs::write(
        source_root.join("App.vue"),
        cstr!(
            r#"<script setup lang="ts">
import MyItem from "./MyItem.vue";
const onClick = (event: {listener}) => {body};
</script>
<template>
  <MyItem role="button" @click="onClick" />
</template>
"#
        ),
    )
    .unwrap();
    let mut config = json!({
        "compilerOptions": {
            "target": "ESNext",
            "module": "preserve",
            "moduleResolution": "bundler",
            "strict": true,
            "noEmit": true,
            "skipLibCheck": true,
        },
        "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
        "include": ["src"],
    });
    match (fallthrough_attributes, vue_target) {
        (None, None) => {}
        (Some(enabled), None) => {
            config["vueCompilerOptions"] = json!({ "fallthroughAttributes": enabled });
        }
        (None, Some(target)) => {
            config["vueCompilerOptions"] = json!({ "target": target });
        }
        (Some(enabled), Some(target)) => {
            config["vueCompilerOptions"] =
                json!({ "fallthroughAttributes": enabled, "target": target });
        }
    }
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
}
