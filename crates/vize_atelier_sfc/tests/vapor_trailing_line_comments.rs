//! #7894: whole Vapor SFCs with authored trailing line comments.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    reason = "whole public regression results cross an independent runtime oracle"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::CodegenOptions;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, TemplateCompileOptions,
    compile_sfc_for_adapter, parse_sfc,
};
use vize_carton::String;

const REPORTED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-trailing-line-comments/Reported.vue.txt"
);
const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-trailing-line-comments/cases.json"
);

#[test]
fn whole_sfc_import_metadata_keeps_vue_values_external_refs_and_type_only_boundaries() {
    let source = r#"<script setup lang="ts">
import { reactive as makeState, ref, type Component } from "vue";
import type { Ref } from "vue";
import { value } from "./external.js";
</script><template><span></span></template>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    for (vapor, vue_binding) in [(false, "setup-maybe-ref"), (true, "setup-const")] {
        let result = compile_sfc_for_adapter(
            &descriptor,
            SfcCompileOptions {
                vapor,
                ..Default::default()
            },
            Default::default(),
            Default::default(),
            CodegenOptions::default(),
            SfcScriptOutputMode::InlineTemplate,
        )
        .unwrap();
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        let bindings = result.bindings.unwrap();
        assert_eq!(
            serde_json::to_value(&bindings.bindings).unwrap(),
            json!({"makeState": vue_binding, "ref": vue_binding, "value": "setup-maybe-ref"})
        );
        assert!(bindings.props_aliases.is_empty());
        assert!(bindings.is_script_setup);
    }
}

#[test]
fn complete_vapor_modules_match_official_rendered_behavior_with_line_comments() {
    let mut fixtures = vec![json!({"name": "reported", "source": REPORTED})];
    fixtures.extend(serde_json::from_str::<Vec<Value>>(CASES).unwrap());
    assert_eq!(fixtures.len(), 16);
    for production in [false, true] {
        let mut observations = Vec::new();
        for fixture in &fixtures {
            let source = fixture.get("source").and_then(Value::as_str).unwrap();
            let parse = SfcParseOptions {
                filename: "Reported.vue".into(),
                ..Default::default()
            };
            let descriptor = parse_sfc(source, parse.clone()).unwrap();
            let result = compile_sfc_for_adapter(
                &descriptor,
                SfcCompileOptions {
                    parse,
                    vapor: true,
                    template: TemplateCompileOptions {
                        is_prod: production,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Default::default(),
                Default::default(),
                CodegenOptions::default(),
                SfcScriptOutputMode::InlineTemplate,
            )
            .unwrap();
            assert!(result.errors.is_empty(), "{:?}", result.errors);
            assert!(result.warnings.is_empty(), "{:?}", result.warnings);
            assert!(result.css.is_none());
            assert!(result.map.is_none());
            assert!(result.bindings.is_some());
            assert!(result.macro_artifacts.is_empty());
            observations.push(json!({
                "name": fixture.get("name").unwrap(),
                "source": source,
                "current": result,
            }));
        }
        let mut child = Command::new("node")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/vapor-trailing-line-comments.mjs"),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                serde_json::to_string(&json!({
                    "production": production,
                    "cases": observations,
                }))
                .unwrap()
                .as_bytes(),
            )
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "production={production}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
