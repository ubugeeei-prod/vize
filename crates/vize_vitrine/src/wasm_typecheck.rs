//! WASM bindings for type checking.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use wasm_bindgen::prelude::*;

use crate::typecheck::{type_check_sfc, TypeCheckOptions};

/// Helper function to serialize values to JsValue with maps as objects
fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ============================================================================
// Canon (Type Checker) WASM bindings
// ============================================================================

/// Perform type checking on a Vue SFC
///
/// This performs AST-based type analysis without requiring a TypeScript compiler.
/// For full type checking, use the CLI with Corsa integration.
#[wasm_bindgen(js_name = "typeCheck")]
pub fn type_check_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let filename: String = js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "anonymous.vue".to_string());

    let strict = js_sys::Reflect::get(&options, &JsValue::from_str("strict"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let include_virtual_ts = js_sys::Reflect::get(&options, &JsValue::from_str("includeVirtualTs"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let check_props = js_sys::Reflect::get(&options, &JsValue::from_str("checkProps"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let check_emits = js_sys::Reflect::get(&options, &JsValue::from_str("checkEmits"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let check_template_bindings =
        js_sys::Reflect::get(&options, &JsValue::from_str("checkTemplateBindings"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

    let mut opts = TypeCheckOptions::new(filename);
    opts.strict = strict;
    opts.include_virtual_ts = include_virtual_ts;
    opts.check_props = check_props;
    opts.check_emits = check_emits;
    opts.check_template_bindings = check_template_bindings;

    let result = type_check_sfc(source, &opts);

    // Convert to JSON-friendly format
    let output = serde_json::json!({
        "diagnostics": result.diagnostics.iter().map(|d| {
            serde_json::json!({
                "severity": match d.severity {
                    crate::typecheck::TypeSeverity::Error => "error",
                    crate::typecheck::TypeSeverity::Warning => "warning",
                    crate::typecheck::TypeSeverity::Info => "info",
                    crate::typecheck::TypeSeverity::Hint => "hint",
                },
                "message": d.message,
                "start": d.start,
                "end": d.end,
                "code": d.code,
                "help": d.help,
                "related": d.related.iter().map(|r| {
                    serde_json::json!({
                        "message": r.message,
                        "start": r.start,
                        "end": r.end,
                        "filename": r.filename,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "virtualTs": result.virtual_ts,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "analysisTimeMs": result.analysis_time_ms,
    });

    to_js_value(&output)
}

/// Get type checking capabilities info
#[wasm_bindgen(js_name = "getTypeCheckCapabilities")]
pub fn get_type_check_capabilities_wasm() -> Result<JsValue, JsValue> {
    let capabilities = serde_json::json!({
        "mode": "ast-based",
        "description": "AST-based type analysis (no TypeScript compiler required)",
        "checks": [
            {
                "name": "untyped-props",
                "description": "Detects props without type definitions",
                "severity": "warning",
            },
            {
                "name": "untyped-emits",
                "description": "Detects emits without type definitions",
                "severity": "warning",
            },
            {
                "name": "undefined-binding",
                "description": "Detects undefined template bindings",
                "severity": "error",
            },
        ],
        "notes": [
            "For full TypeScript type checking, use the CLI with Corsa integration",
            "AST-based analysis catches common issues without external dependencies",
        ],
    });

    to_js_value(&capabilities)
}
