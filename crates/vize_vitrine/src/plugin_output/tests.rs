#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "plugin wire contract fixtures use standard strings"
)]
use super::*;
use crate::CompileResult;
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_experimental_options,
};
use vize_l0::Allocator;

mod cache;
mod maps;

fn native(mapped: bool) -> CompileResult {
    let allocator = Allocator::new();
    let source = "<button title=\"日本語🎨\" @click=\"save\">{{ label }}</button>";
    let compiled = compile_vapor_with_experimental_options(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        VaporCompilerExperimentalOptions {
            source_map: mapped,
            source_map_filename: Some("src/Example.vue".into()),
            ..Default::default()
        },
    );
    assert!(compiled.error_messages.is_empty());
    CompileResult {
        code: compiled.code.to_string(),
        preamble: String::new(),
        ast: serde_json::json!({}),
        map: compiled.map.map(|map| serde_json::from_str(&map).unwrap()),
        helpers: vec![],
        templates: Some(compiled.templates.iter().map(ToString::to_string).collect()),
    }
}

fn spec(name: &'static str, family: &'static str) -> PluginSpec<'static> {
    PluginSpec {
        name,
        version: "1.0.0",
        fingerprint: "plugin-code",
        family,
        inputs: Some(&[]),
    }
}

fn simple(code: &str) -> CompileResult {
    CompileResult {
        code: code.to_owned(),
        preamble: String::new(),
        ast: serde_json::json!({}),
        map: None,
        helpers: vec![],
        templates: None,
    }
}

#[test]
fn real_native_formatter_preserves_every_other_compile_field() {
    let compiled = native(true);
    let at = compiled.code.find('\n').unwrap();
    let response = serde_json::json!([{ "start": at, "end": at + 1, "text": "\n\n" }]).to_string();
    let (result, cost) = run(
        &compiled,
        &spec("real-formatter", "formatter"),
        "config",
        false,
        true,
        None,
        |batch| {
            let batch: serde_json::Value = serde_json::from_str(&batch).unwrap();
            assert_eq!(batch["schema"], 1);
            assert_eq!(batch["offsetEncoding"], "utf8");
            assert_eq!(batch["compiled"], serde_json::to_value(&compiled).unwrap());
            Ok(response.clone())
        },
    )
    .unwrap();
    assert_eq!(result.code.len(), compiled.code.len() + 1);
    assert_eq!(result.preamble, compiled.preamble);
    assert_eq!(result.helpers, compiled.helpers);
    assert_eq!(result.templates, compiled.templates);
    assert_eq!(result.ast, compiled.ast);
    assert_eq!(cost.operations, 1);
    assert!(!cost.cached && cost.batch_bytes > 0 && cost.elapsed_ns > 0.0 && cost.js_ns > 0.0);
}

#[test]
fn formatter_rejects_asi_literals_comments_and_non_whitespace() {
    for (code, start, end, text) in [
        ("function f(){return value}", 19, 20, "\n"),
        ("const text = `a b`;", 15, 16, "\n"),
        ("const text = 'a b';", 15, 16, "  "),
        ("/* legal notice */const a=1;", 2, 3, "\n"),
        ("const a=1;", 6, 7, "b"),
    ] {
        let response =
            serde_json::json!([{ "start": start, "end": end, "text": text }]).to_string();
        assert!(
            rewrite::apply(&simple(code), "formatter", &response).is_err(),
            "{code}"
        );
    }
}

#[test]
fn formatter_function_output_and_no_map_lane_are_supported() {
    let code = "const Vue = {};\nreturn function render(_ctx){ with(_ctx){ return value } }";
    let (result, _) = rewrite::apply(
        &simple(code),
        "formatter",
        "[{\"start\":16,\"end\":16,\"text\":\"  \"}]",
    )
    .unwrap();
    assert!(result.map.is_none());
    assert!(result.code.contains("\n  return"));
    let native = native(false);
    let (result, _) = rewrite::apply(
        &native,
        "output",
        "[{\"placement\":\"prepend\",\"comment\":\"license\"}]",
    )
    .unwrap();
    assert!(result.map.is_none());
}

#[test]
fn malformed_and_overlapping_edits_fail_closed() {
    for response in [
        "{}",
        "[{\"start\":1000,\"end\":1001,\"text\":\" \"}]",
        "[{\"start\":1,\"end\":0,\"text\":\" \"}]",
        "[{\"start\":0,\"end\":1,\"text\":\" \"},{\"start\":0,\"end\":0,\"text\":\" \"}]",
        "[{\"start\":0,\"end\":0,\"text\":\" \",\"code\":\"malicious\"}]",
    ] {
        assert!(rewrite::apply(&simple("  const x=1;"), "formatter", response).is_err());
    }
    assert!(
        rewrite::apply(
            &simple("const 日本語=1;"),
            "formatter",
            "[{\"start\":7,\"end\":7,\"text\":\" \"}]"
        )
        .is_err()
    );
}

#[test]
fn output_cannot_replace_javascript_or_supply_forged_maps() {
    for response in [
        "[{\"placement\":\"prepend\",\"comment\":\"*/ alert(1); /*\"}]",
        "[{\"placement\":\"replace\",\"comment\":\"license\"}]",
        "[{\"placement\":\"prepend\",\"comment\":\"license\",\"map\":{}}]",
        "[{\"placement\":\"prepend\",\"comment\":\"# sourceMappingURL=stale.map\"}]",
        "[{\"placement\":\"prepend\",\"comment\":\"# sourceURL=stale.js\"}]",
        "[{\"placement\":\"prepend\",\"comment\":\"#__PURE__\"}]",
        "[{\"placement\":\"prepend\",\"comment\":\"@__NO_SIDE_EFFECTS__\"}]",
    ] {
        assert!(rewrite::apply(&native(true), "output", response).is_err());
    }
}
