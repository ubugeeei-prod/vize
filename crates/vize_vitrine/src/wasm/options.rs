//! Parsing of compiler and CSS options from JavaScript option objects.

use wasm_bindgen::prelude::*;

use crate::CompilerOptions;
use vize_atelier_sfc::{CssCompileOptions, CssTargets};

pub(crate) struct ParsedCompilerOptions {
    pub(crate) options: CompilerOptions,
    pub(crate) binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
}

pub(crate) fn parse_compiler_options(options: &JsValue) -> ParsedCompilerOptions {
    let get_string = |key: &str| {
        js_sys::Reflect::get(options, &JsValue::from_str(key))
            .ok()
            .and_then(|value| value.as_string())
    };

    let get_bool = |key: &str| {
        js_sys::Reflect::get(options, &JsValue::from_str(key))
            .ok()
            .and_then(|value| value.as_bool())
    };

    let binding_metadata = js_sys::Reflect::get(options, &JsValue::from_str("bindingMetadata"))
        .ok()
        .and_then(|value| {
            if value.is_null() || value.is_undefined() {
                return None;
            }
            let json = js_sys::JSON::stringify(&value).ok()?.as_string()?;
            serde_json::from_str(&json).ok()
        });

    ParsedCompilerOptions {
        options: CompilerOptions {
            mode: get_string("mode"),
            prefix_identifiers: get_bool("prefixIdentifiers"),
            hoist_static: get_bool("hoistStatic"),
            cache_handlers: get_bool("cacheHandlers"),
            scope_id: get_string("scopeId"),
            ssr: get_bool("ssr"),
            source_map: get_bool("sourceMap"),
            filename: get_string("filename"),
            output_mode: get_string("outputMode"),
            is_ts: get_bool("isTs"),
            custom_renderer: get_bool("customRenderer"),
            template_syntax: get_string("templateSyntax"),
            runtime_module_name: get_string("runtimeModuleName"),
            runtime_global_name: get_string("runtimeGlobalName"),
            script_ext: get_string("scriptExt"),
        },
        binding_metadata,
    }
}

/// Parse CSS options from JsValue
pub(crate) fn parse_css_options(options: JsValue) -> CssCompileOptions {
    let scope_id = js_sys::Reflect::get(&options, &JsValue::from_str("scopeId"))
        .ok()
        .and_then(|v| v.as_string())
        .map(Into::into);

    let scoped = js_sys::Reflect::get(&options, &JsValue::from_str("scoped"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let minify = js_sys::Reflect::get(&options, &JsValue::from_str("minify"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let source_map = js_sys::Reflect::get(&options, &JsValue::from_str("sourceMap"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let css_modules = js_sys::Reflect::get(&options, &JsValue::from_str("cssModules"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let custom_media = js_sys::Reflect::get(&options, &JsValue::from_str("customMedia"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let filename = js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|v| v.as_string())
        .map(Into::into);

    // Parse targets
    let targets = js_sys::Reflect::get(&options, &JsValue::from_str("targets"))
        .ok()
        .and_then(|v| {
            if v.is_undefined() || v.is_null() {
                return None;
            }
            Some(CssTargets {
                chrome: js_sys::Reflect::get(&v, &JsValue::from_str("chrome"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
                firefox: js_sys::Reflect::get(&v, &JsValue::from_str("firefox"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
                safari: js_sys::Reflect::get(&v, &JsValue::from_str("safari"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
                edge: js_sys::Reflect::get(&v, &JsValue::from_str("edge"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
                ios: js_sys::Reflect::get(&v, &JsValue::from_str("ios"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
                android: js_sys::Reflect::get(&v, &JsValue::from_str("android"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
            })
        });

    CssCompileOptions {
        scope_id,
        scoped,
        minify,
        source_map,
        targets,
        filename,
        custom_media,
        css_modules,
    }
}
