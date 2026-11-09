//! #11657: an authored check TypeScript anchors at the `(` wrapping it stays
//! reported; the unmapped ignores cover only generated-only scaffolding.
//! Since #11675 a listener check lands on the authored value's `(`, not on
//! the attribute's `=`.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/listener-anchor-11657/Child.vue.txt"
));

/// Every template wrapper form the generator emits, one authored error each.
const FORMS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/listener-anchor-11657/Forms.vue.txt"
));

/// `(file, template)`. Every wrong value is an authored error.
const CASES: [(&str, &str); 4] = [
    (
        "Handler",
        r#"  <Child foo-bar="ok" @save-item="(value: number) => undefined" />"#,
    ),
    (
        "HandlerAndProp",
        r#"  <Child :foo-bar="1" @save-item="(value: number) => undefined" />"#,
    ),
    (
        "Expression",
        r#"  <Child foo-bar="ok" @save-item="(value: number) => value" />"#,
    ),
    // Negative control: a correct handler and prop report nothing.
    (
        "Valid",
        r#"  <Child foo-bar="ok" @save-item="(value: string) => undefined" />"#,
    ),
];

#[test]
fn checks_anchored_at_wrapping_parentheses_stay_reported() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-listener-anchor-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    std::fs::write(project.join("src/Child.vue"), CHILD).unwrap();
    std::fs::write(project.join("src/Forms.vue"), FORMS).unwrap();
    for (name, template) in CASES {
        std::fs::write(
            project.join(cstr!("src/{name}.vue").as_str()),
            cstr!("<script setup lang=\"ts\">\nimport Child from \"./Child.vue\"\n</script>\n<template>\n{template}\n</template>\n").as_str(),
        )
        .unwrap();
    }
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler",
                "types": [], "lib": ["ES2022", "DOM"]
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = check_project(&tsgo, project, "tsconfig.json");
    let mut lines = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("src/"))
        // Anchor and code only: the two hosts may flatten messages differently.
        .map(|line| {
            line.split_once(": ").map_or(line, |(at, rest)| {
                &line[..at.len() + 2 + rest.find(':').unwrap_or(rest.len())]
            })
        })
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "src/Expression.vue(5,24): error TS2322",
            "src/Forms.vue(10,47): error TS2345",
            "src/Forms.vue(11,54): error TS2345",
            "src/Forms.vue(12,11): error TS2322",
            "src/Forms.vue(13,4): error TS2345",
            "src/Forms.vue(14,32): error TS2322",
            "src/Forms.vue(15,21): error TS2345",
            "src/Forms.vue(16,26): error TS2345",
            "src/Forms.vue(17,24): error TS2345",
            "src/Forms.vue(18,30): error TS2345",
            "src/Forms.vue(19,26): error TS2345",
            "src/Forms.vue(20,56): error TS2345",
            "src/Forms.vue(21,26): error TS2345",
            "src/Forms.vue(22,19): error TS2345",
            "src/Forms.vue(23,31): error TS2345",
            "src/Forms.vue(24,27): error TS2345",
            "src/Forms.vue(25,51): error TS2322",
            "src/Forms.vue(9,35): error TS2322",
            "src/Handler.vue(5,35): error TS2322",
            "src/HandlerAndProp.vue(5,11): error TS2322",
            "src/HandlerAndProp.vue(5,35): error TS2322",
        ],
        "{}",
        output_text(&output)
    );
}
