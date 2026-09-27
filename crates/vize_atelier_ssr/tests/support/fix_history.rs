//! Exact public SSR output payload for the authored fix-history cases.

use serde_json::{Value, json};
use vize_atelier_core::CompilerError;
use vize_atelier_ssr::{
    SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions, compile_ssr,
};
use vize_l0::Allocator;

pub const CASES: &[(&str, &str)] = &[
    (
        "select-model",
        include_str!("../fixtures/fix-history/select-model.input.txt"),
    ),
    (
        "model-before-listener",
        include_str!("../fixtures/fix-history/model-before-listener.input.txt"),
    ),
    (
        "listener-before-model",
        include_str!("../fixtures/fix-history/listener-before-model.input.txt"),
    ),
    (
        "event-modifiers",
        include_str!("../fixtures/fix-history/event-modifiers.input.txt"),
    ),
    (
        "event-spread-boundary",
        include_str!("../fixtures/fix-history/event-spread-boundary.input.txt"),
    ),
    (
        "event-dynamic-key-boundary",
        include_str!("../fixtures/fix-history/event-dynamic-key-boundary.input.txt"),
    ),
    (
        "keyed-slot-conditional",
        include_str!("../fixtures/fix-history/keyed-slot-conditional.input.txt"),
    ),
    (
        "keyed-slot-single-element",
        include_str!("../fixtures/fix-history/keyed-slot-single-element.input.txt"),
    ),
    (
        "keyed-slot-fallback",
        include_str!("../fixtures/fix-history/keyed-slot-fallback.input.txt"),
    ),
];

pub fn options() -> Result<Value, Box<dyn std::error::Error>> {
    let options = SsrCompilerOptions::default();
    if options.dialect != vize_l0::config::VueVersion::V3
        || options.binding_metadata.is_some()
        || options.croquis.is_some()
    {
        return Err(
            "SSR fixture defaults gained dialect, binding metadata or Croquis context".into(),
        );
    }
    let SsrCompilerExperimentalOptions {
        component_name,
        self_component,
        source_map,
        source_map_filename,
    } = SsrCompilerExperimentalOptions::default();
    Ok(json!({
        "entrypoint": "compile_ssr",
        "publicOptions": serde_json::to_value(&options)?,
        "dialect": "V3",
        "bindingMetadata": null,
        "croquis": null,
        "templateSyntax": "Standard",
        "customElements": "default",
        "rawTemplateSlotted": true,
        "experimental": {
            "componentName": component_name, "selfComponent": self_component,
            "sourceMap": source_map, "sourceMapFilename": source_map_filename,
        },
    }))
}

pub fn observe(source: &str) -> Result<Value, serde_json::Error> {
    let allocator = Allocator::new();
    let (_, diagnostics, result) = compile_ssr(&allocator, source);
    // Exhaustive public-field patterns make added result/diagnostic fields
    // require an explicit extension to the fixture contract.
    let SsrCodegenResult {
        code,
        preamble,
        map,
    } = result;
    let diagnostics = diagnostics
        .into_iter()
        .map(|error| {
            let CompilerError { code, message, loc } = error;
            json!({ "code": code as u16, "message": message, "loc": loc })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "code": code.as_str(),
        "preamble": preamble.as_str(),
        "map": map.as_ref().map(|value| value.as_str()),
        "diagnostics": diagnostics,
    }))
}
