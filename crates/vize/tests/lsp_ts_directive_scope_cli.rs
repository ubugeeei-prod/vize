#![cfg(test)]
//! Authored suppression comments cannot hide a different import or statement.
use crate::support::lsp_vue_project as support;
use serde_json::{Value, json};
use support::Fixture;

const APP: &str = r#"<script setup lang="ts">
// @ts-expect-error third-party editor has no declarations
import Monaco from './missing-monaco.ts'
import Child from './Child.vue'
const store = new Map()
void Monaco
void store
</script>
<template><Child /></template>
"#;
const CHILD: &str = "<template><span /></template>\n";

#[test]
fn consumed_import_directive_preserves_deleted_sfc_diagnostic_and_recovery() {
    let mut fixture = Fixture::new_with_vue_component_project(
        APP,
        "App.vue",
        &[
            ("Child.vue", CHILD),
            (
                "env.d.ts",
                "declare module '*.vue' { import type { DefineComponent } from 'vue'; const component: DefineComponent; export default component }\n",
            ),
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.vue","*.d.ts"]}"#,
            ),
        ],
    );
    assert_eq!(
        fixture.open(APP),
        json!([]),
        "existing SFC and consumed directive"
    );

    // The deleted dependency was never opened. The unchanged importer must
    // report its missing SFC despite the ambient wildcard and the consumed
    // suppression on the preceding import, as in Varlet's playground.
    std::fs::remove_file(fixture.project_root().join("Child.vue")).unwrap();
    let deleted = fixture.change(APP, 2);
    assert_missing_child(&deleted);

    fixture.write_file("Child.vue", CHILD);
    assert_eq!(fixture.change(APP, 3), json!([]), "restored dependency");
    fixture.shutdown();
}

#[test]
fn multiline_call_suppression_keeps_an_independent_following_error() {
    let source = r#"<script setup lang="ts">
function acceptNumber(value: number) { return value }
// @ts-ignore intentional argument error
acceptNumber(
  'suppressed',
)
const visible: number = 'independent'
void visible
</script>
<template><span /></template>
"#;
    let mut fixture = Fixture::new_with_vue(source);
    let diagnostics = fixture.open(source);
    let diagnostics = diagnostics.as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0]["code"], 2322);
    assert_eq!(
        diagnostics[0]["range"]["start"],
        support::position(source, "visible:")
    );
    assert_eq!(
        fixture.change(&source.replace("'independent'", "42"), 2),
        json!([]),
        "repair leaves the intentional multiline suppression effective"
    );
    fixture.shutdown();
}

#[test]
fn import_literal_parentheses_cannot_extend_the_suppression() {
    let source = r#"<script setup lang="ts">
// @ts-expect-error intentionally unavailable module
import Ignored from './missing(parenthesis.ts'
const visible: number = 'independent)'
void Ignored
void visible
</script>
<template><span /></template>
"#;
    let mut fixture = Fixture::new_with_vue(source);
    let diagnostics = fixture.open(source);
    let diagnostics = diagnostics.as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0]["code"], 2322);
    assert_eq!(
        diagnostics[0]["range"]["start"],
        support::position(source, "visible:")
    );
    fixture.shutdown();
}

fn assert_missing_child(diagnostics: &Value) {
    let diagnostics = diagnostics.as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["code"], 2307);
    assert_eq!(
        diagnostic["message"],
        "Cannot find module './Child.vue' or its corresponding type declarations."
    );
    assert_eq!(diagnostic["range"]["start"]["line"], 3);
}
