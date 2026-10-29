//! Public build acceptance for legacy DOM `v-for` alias expressions (#10811).
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "whole compiler output")]

use std::{fs, path::Path, process::Command};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

const SOURCE: &str = r#"<script>export default {}</script>
<template><!-- @vize:todo x --><div v-for="outer in rows"><p v-for="{ label = fallback, [key]: tag } in outer.items">{{ label + tag + outside }}</p></div></template>
"#;
const EXPECTED_MODULE: &str = r#"const _sfc_main = {}
import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock, Fragment as _Fragment, renderList as _renderList } from "vue"

function _sfc_render(_ctx, _cache) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.rows, (outer) => {
    return (_openBlock(), _createElementBlock("div", null, [
      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(outer.items, ({ label = _ctx.fallback, [_ctx.key]: tag }) => {
        return (_openBlock(), _createElementBlock("p", null, _toDisplayString(label + tag + _ctx.outside), 1 /* TEXT */))
      }), 256 /* UNKEYED_FRAGMENT */))
    ]))
  }), 256 /* UNKEYED_FRAGMENT */))
}
_sfc_main.render = _sfc_render
export default _sfc_main"#;

fn compiler_code(source: &str, name: &str) -> String {
    let parse = SfcParseOptions {
        filename: name.into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).unwrap();
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.id = Some(name.into());
    options.style.id = name.into();
    let result = compile_sfc(&descriptor, options).unwrap();
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().into()
}

#[test]
fn source_built_cli_keeps_legacy_aliases_scoped_and_rewrites_free_reads() {
    let expected = compiler_code(SOURCE, "App.vue");
    assert_eq!(expected, EXPECTED_MODULE);

    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("App.vue"), SOURCE).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--no-config",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            "js",
            "App.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "{:?}", output.stderr);
    assert_eq!(output.stdout, b"");
    assert_eq!(
        normalize_time(output.stderr),
        "Built: App.vue -> out/App.js\n\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n"
    );
    assert_eq!(
        fs::read(project.path().join("out/App.js")).unwrap(),
        expected.as_bytes()
    );
    assert_eq!(
        fs::read(project.path().join("App.vue")).unwrap(),
        SOURCE.as_bytes()
    );
}

fn normalize_time(bytes: Vec<u8>) -> String {
    let text = String::from_utf8(bytes).unwrap();
    let (head, time) = text.rsplit_once(" compiled in ").unwrap();
    let (time, tail) = time.split_once('s').unwrap();
    let elapsed = time.parse::<f64>().unwrap();
    assert!(elapsed.is_finite() && elapsed >= 0.0);
    format!("{head} compiled in <TIME>s{tail}")
}
