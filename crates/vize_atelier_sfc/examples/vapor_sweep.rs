//! Local discovery tool: compile a JSON corpus through the public Vapor and SFC
//! entrypoints so the outputs can be compared with the official compilers.
#![allow(clippy::all, clippy::pedantic, clippy::restriction)]

use serde_json::{Value, json};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: vapor_sweep <cases.json>")?;
    let cases: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let mut out = Vec::with_capacity(cases.len());
    for case in &cases {
        let id = case["id"].as_str().unwrap_or_default();
        let source = case["source"].as_str().unwrap_or_default();
        let kind = case["kind"].as_str().unwrap_or("template");
        let result = std::panic::catch_unwind(|| match kind {
            "sfc" => sfc(source, case),
            _ => template(source, case),
        })
        .unwrap_or_else(|_| json!({ "panic": true }));
        out.push(json!({ "id": id, "result": result }));
    }
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

fn template(source: &str, case: &Value) -> Value {
    let allocator = Allocator::new();
    let compiled = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: case["prefix"].as_bool().unwrap_or(true),
            ..VaporCompilerOptions::default()
        },
    );
    json!({
        "code": compiled.code.to_string(),
        "errors": compiled.error_messages.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
    })
}

fn sfc(source: &str, case: &Value) -> Value {
    let ssr = case["ssr"].as_bool().unwrap_or(false);
    let is_ts = source.contains("lang=\"ts\"");
    let id = "data-v-sweep";
    let descriptor = match parse_sfc(
        source,
        SfcParseOptions {
            filename: "App.vue".into(),
            ..SfcParseOptions::default()
        },
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => return json!({ "parseError": format!("{error:?}") }),
    };
    let options = SfcCompileOptions {
        script: ScriptCompileOptions {
            id: Some(id.into()),
            inline_template: case["inline"].as_bool().unwrap_or(true),
            is_ts,
            ..ScriptCompileOptions::default()
        },
        template: TemplateCompileOptions {
            id: Some(id.into()),
            ssr,
            is_prod: case["prod"].as_bool().unwrap_or(false),
            is_ts,
            ..TemplateCompileOptions::default()
        },
        vapor: case["vapor"].as_bool().unwrap_or(false),
        ..SfcCompileOptions::default()
    };
    match compile_sfc(&descriptor, options) {
        Ok(result) => json!({
            "code": result.code,
            "errors": result.errors.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>(),
            "warnings": result.warnings.iter().map(|e| e.message.to_string()).collect::<Vec<_>>(),
        }),
        Err(error) => json!({ "compileError": format!("{error:?}") }),
    }
}
