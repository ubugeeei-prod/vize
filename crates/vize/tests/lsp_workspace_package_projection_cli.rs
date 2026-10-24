//! Shared workspace-package projections retain native types in one LSP process.
#![cfg(test)]
use crate::support::lsp_vue_project as project;

use project::Fixture;
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

const APP: &str = r#"<script setup lang="ts">
import { value, Widget } from 'widget-package';
value.toPrecision();
</script>
<template><Widget :count="1" /></template>
"#;
const INDEX: &str =
    "export { value } from './value';\nexport { default as Widget } from './Root.vue';\n";
const NUMBER: &str = "export const value = 123 as number;\n";
const TEXT: &str = "export const value = 'hello' as string;\n";
const ROOT: &str = r#"<script setup lang="ts">
import Leaf from './Leaf.vue';
defineProps<{ count: number }>();
</script>
<template><Leaf :count="count" /></template>
"#;
const LEAF: &str = r#"<script setup lang="ts">
defineProps<{ count: number }>();
</script>
<template><div>{{ count.toPrecision() }}</div></template>
"#;
const CONFIG: &str = r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["src/**/*.vue"]}"#;

#[test]
fn shared_workspace_package_native_queries_and_freshness_share_one_server() {
    let mut fixture = Fixture::new_with_vue_component_project(
        APP,
        "src/App.vue",
        &[
            (
                "package.json",
                r#"{"private":true,"workspaces":["packages/*"]}"#,
            ),
            ("tsconfig.json", CONFIG),
            (
                "packages/widgets/package.json",
                r#"{"name":"widget-package","type":"module","exports":"./src/index.ts"}"#,
            ),
            ("packages/widgets/src/index.ts", INDEX),
            ("packages/widgets/src/value.ts", NUMBER),
            ("packages/widgets/src/Root.vue", ROOT),
            ("packages/widgets/src/Leaf.vue", LEAF),
        ],
    );
    link_package(fixture.project_root());
    let cold = Instant::now();
    assert_eq!(fixture.open(APP), json!([]));
    let number_hover = assert_number_queries(&mut fixture, APP);
    let cold_elapsed = cold.elapsed();

    let warm = Instant::now();
    for index in 0..12 {
        let name = vize_l0::cstr!("src/Consumer{index}.vue");
        let uri = fixture.write_file(&name, APP);
        assert_eq!(fixture.open_file(&uri, APP), json!([]));
        let hover =
            fixture.request_file_with("textDocument/hover", &uri, APP, "toPrecision()", json!({}));
        assert_eq!(hover, number_hover, "complete shared-consumer hover");
    }
    let warm_elapsed = warm.elapsed();

    let wrong_number = APP.replace("toPrecision()", "toUpperCase()");
    assert_native_missing_method(
        &fixture.change(&wrong_number, 2),
        &wrong_number,
        "number",
        "toUpperCase",
    );
    assert_eq!(fixture.change(APP, 3), json!([]));

    // The same session must invalidate the shared package closure on a disk
    // edit, rather than returning a cached numeric projection to every host.
    fixture.write_file("packages/widgets/src/value.ts", TEXT);
    let string_source = APP.replace("toPrecision()", "toUpperCase()");
    assert_eq!(fixture.change(&string_source, 4), json!([]));
    let string_hover = fixture.request("textDocument/hover", &string_source, "toUpperCase()");
    assert_method(&string_hover, "toUpperCase");
    let wrong_string = vize_l0::cstr!("{APP}<!-- changed package -->\n");
    assert_native_missing_method(
        &fixture.change(&wrong_string, 5),
        &wrong_string,
        "string",
        "toPrecision",
    );

    // Replacing a manifest target after deleting its old implementation must
    // retire its type and pick up the newly exported numeric source.
    fixture.write_file("packages/widgets/src/replacement.ts", NUMBER);
    std::fs::remove_file(fixture.project_root().join("packages/widgets/src/value.ts")).unwrap();
    std::fs::remove_file(fixture.project_root().join("packages/widgets/src/index.ts")).unwrap();
    fixture.write_file(
        "packages/widgets/src/next-entry.ts",
        "export { value } from './replacement';\nexport { default as Widget } from './Root.vue';\n",
    );
    fixture.write_file(
        "packages/widgets/package.json",
        r#"{"name":"widget-package","type":"module","exports":"./src/next-entry.ts"}"#,
    );
    let restored = vize_l0::cstr!("{APP}<!-- replacement module -->\n");
    assert_eq!(fixture.change(&restored, 6), json!([]));
    assert_eq!(assert_number_queries(&mut fixture, &restored), number_hover);

    // Config changes remain authoritative while the package projection is
    // warm. A paths target switches the same authored bare package spelling.
    fixture.write_file("packages/widgets/src/text-entry.ts", "export { default as Widget } from './Root.vue';\nexport const value = 'changed config' as string;\n");
    fixture.write_file("tsconfig.json", r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true,"paths":{"widget-package":["./packages/widgets/src/text-entry.ts"]}},"include":["src/**/*.vue"]}"#);
    let configured = vize_l0::cstr!("{string_source}<!-- changed config -->\n");
    assert_eq!(fixture.change(&configured, 7), json!([]));
    assert_eq!(
        fixture.request("textDocument/hover", &configured, "toUpperCase()"),
        string_hover,
        "complete string hover survives config-driven projection replacement"
    );
    fixture.write_file("tsconfig.json", CONFIG);
    let final_source = vize_l0::cstr!("{APP}<!-- restored config -->\n");
    assert_eq!(fixture.change(&final_source, 8), json!([]));
    assert_eq!(
        assert_number_queries(&mut fixture, &final_source),
        number_hover
    );
    eprintln!(
        "workspace package projection: cold={cold_elapsed:?}, 12 shared consumers={warm_elapsed:?}"
    );
    fixture.shutdown();
}

fn assert_number_queries(fixture: &mut Fixture, source: &str) -> Value {
    let hover = fixture.request("textDocument/hover", source, "toPrecision()");
    assert_method(&hover, "toPrecision");
    let completion = fixture.request_with(
        "textDocument/completion",
        source,
        "recision()",
        json!({"context":{"triggerKind":1}}),
    );
    let items = completion
        .as_array()
        .or_else(|| completion["items"].as_array())
        .expect("native member completions");
    assert!(
        items.iter().any(|item| item["label"] == "toPrecision"),
        "{completion:#}"
    );
    assert!(
        !items.iter().any(|item| item["label"] == "toUpperCase"),
        "{completion:#}"
    );
    hover
}

fn assert_method(hover: &Value, name: &str) {
    let markdown = hover["contents"]["value"]
        .as_str()
        .expect("native Markdown hover");
    let (signature, _) = markdown
        .strip_prefix("```typescript\n")
        .and_then(|text| text.split_once("\n```"))
        .expect("complete native TypeScript signature fence");
    // Signatures follow the pinned TypeScript lib.es5 declarations. Preserve
    // the complete native documentation in the per-type session oracle above.
    let expected = match name {
        "toPrecision" => "(method) Number.toPrecision(precision?: number): string",
        "toUpperCase" => "(method) String.toUpperCase(): string",
        _ => panic!("unowned method signature: {name}"),
    };
    assert_eq!(signature, expected, "{hover:#}");
    assert_eq!(
        hover,
        &json!({
            "contents": {"kind": "markdown", "value": markdown},
            "range": method_range(name)
        })
    );
}

fn method_range(method: &str) -> Value {
    // Every source variant preserves the authored `value.` prefix on line 3.
    json!({
        "start": {"line": 2, "character": 6},
        "end": {"line": 2, "character": 6 + method.len()}
    })
}

fn assert_native_missing_method(diagnostics: &Value, source: &str, ty: &str, method: &str) {
    assert_eq!(
        project::position(source, method),
        method_range(method)["start"]
    );
    assert_eq!(
        diagnostics,
        &json!([{
            "code": 2339,
            "message": vize_l0::cstr!("Property '{method}' does not exist on type '{ty}'."),
            "range": method_range(method),
            "severity": 1,
            "source": "vize/types"
        }])
    );
}

fn link_package(root: &Path) {
    let target = root.join("packages/widgets");
    let link = root.join("node_modules/widget-package");
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(target, link).unwrap();
}
