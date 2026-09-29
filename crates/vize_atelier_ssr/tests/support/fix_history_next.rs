//! Exact public SSR output payload for five further authored fixes.

use serde_json::{Value, json};
use vize_atelier_core::CompilerError;
use vize_atelier_ssr::{
    SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions, compile_ssr,
};
use vize_l0::Allocator;

pub const CASES: &[(&str, &str)] = &[
    (
        "textarea-model",
        include_str!("../fixtures/fix-history-next/textarea-model.input.txt"),
    ),
    (
        "slot-fallback-vnode",
        include_str!("../fixtures/fix-history-next/slot-fallback-vnode.input.txt"),
    ),
    (
        "component-slot-props",
        include_str!("../fixtures/fix-history-next/component-slot-props.input.txt"),
    ),
    (
        "named-scoped-slot",
        include_str!("../fixtures/fix-history-next/named-scoped-slot.input.txt"),
    ),
    (
        "component-lone-spread",
        include_str!("../fixtures/fix-history-next/component-lone-spread.input.txt"),
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
