//! WASM bindings for Vue compiler.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod analyze;
mod cross_file;
#[cfg(feature = "glyph")]
mod format;
mod lint;
mod musea;

// Re-export type checking bindings from separate module
#[path = "../wasm_typecheck.rs"]
mod wasm_typecheck;

// Re-export all WASM bindings
pub use analyze::*;
pub use cross_file::*;
#[cfg(feature = "glyph")]
pub use format::*;
pub use lint::*;
pub use musea::*;
pub use wasm_typecheck::*;

use serde::Serialize;
use std::collections::BTreeMap;
use vize_carton::Bump;
use wasm_bindgen::prelude::*;

use crate::{CompileResult, CompilerOptions};
use vize_atelier_core::options::CodegenMode;
use vize_atelier_core::parser::parse;
use vize_atelier_dom::{compile_template_with_options, DomCompilerOptions};
use vize_atelier_sfc::{
    compile_sfc as sfc_compile, parse_sfc, CssCompileOptions, CssTargets, ScriptCompileOptions,
    SfcCompileOptions, SfcDescriptor, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
};
use vize_atelier_ssr::compile_ssr as ssr_compile;
use vize_atelier_vapor::{compile_vapor as vapor_compile, VaporCompilerOptions};

/// Helper function to serialize values to JsValue with maps as objects
pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

fn to_json_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let json = serde_json::to_string(value).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

struct ParsedCompilerOptions {
    options: CompilerOptions,
    binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
}

fn parse_compiler_options(options: &JsValue) -> ParsedCompilerOptions {
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
            script_ext: get_string("scriptExt"),
        },
        binding_metadata,
    }
}

/// Convert a UTF-8 byte offset to a JavaScript string offset (UTF-16 code units).
/// OXC and the SFC parser report byte offsets, while Monaco/JS consumers expect
/// UTF-16 offsets.
pub(crate) fn utf8_byte_to_utf16_offset(content: &str, byte_offset: u32) -> u32 {
    let byte_offset = byte_offset as usize;
    if byte_offset >= content.len() {
        return content.encode_utf16().count() as u32;
    }
    content[..byte_offset].encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
    use super::utf8_byte_to_utf16_offset;

    #[test]
    fn test_utf8_byte_to_utf16_offset_handles_multibyte_and_astral_chars() {
        let source = "aあ😀b";

        let hiragana_start = source.find('あ').expect("hiragana should exist") as u32;
        let emoji_start = source.find('😀').expect("emoji should exist") as u32;
        let latin_b_start = source.find('b').expect("latin b should exist") as u32;

        assert_eq!(utf8_byte_to_utf16_offset(source, 0), 0);
        assert_eq!(utf8_byte_to_utf16_offset(source, hiragana_start), 1);
        assert_eq!(utf8_byte_to_utf16_offset(source, emoji_start), 2);
        assert_eq!(utf8_byte_to_utf16_offset(source, latin_b_start), 4);
        assert_eq!(utf8_byte_to_utf16_offset(source, source.len() as u32), 5);
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

/// SFC compile result for WASM
#[derive(Serialize)]
pub struct SfcWasmResult {
    pub descriptor: SfcDescriptorWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<CompileResult>,
    pub script: SfcScriptResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bindingMetadata")]
    pub binding_metadata: Option<serde_json::Value>,
}

/// Script compilation result
#[derive(Serialize)]
pub struct SfcScriptResult {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bindings: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcDescriptorWasm {
    pub filename: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<SfcTemplateBlockWasm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<SfcScriptBlockWasm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_setup: Option<SfcScriptBlockWasm>,
    pub styles: Vec<SfcStyleBlockWasm>,
    pub custom_blocks: Vec<SfcCustomBlockWasm>,
    pub css_vars: Vec<String>,
    pub slotted: bool,
    pub should_force_reload: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcTemplateBlockWasm {
    pub content: String,
    pub loc: SfcBlockLocationWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcScriptBlockWasm {
    pub content: String,
    pub loc: SfcBlockLocationWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub setup: bool,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcStyleBlockWasm {
    pub content: String,
    pub loc: SfcBlockLocationWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub scoped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcCustomBlockWasm {
    pub r#type: String,
    pub content: String,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcBlockLocationWasm {
    pub start: usize,
    pub end: usize,
    pub tag_start: usize,
    pub tag_end: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

fn attrs_to_wasm(
    attrs: &vize_carton::FxHashMap<std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>>,
) -> BTreeMap<String, String> {
    attrs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn loc_to_wasm(loc: &vize_atelier_sfc::BlockLocation) -> SfcBlockLocationWasm {
    SfcBlockLocationWasm {
        start: loc.start,
        end: loc.end,
        tag_start: loc.tag_start,
        tag_end: loc.tag_end,
        start_line: loc.start_line,
        start_column: loc.start_column,
        end_line: loc.end_line,
        end_column: loc.end_column,
    }
}

fn template_block_to_wasm(block: &vize_atelier_sfc::SfcTemplateBlock<'_>) -> SfcTemplateBlockWasm {
    SfcTemplateBlockWasm {
        content: block.content.to_string(),
        loc: loc_to_wasm(&block.loc),
        lang: block.lang.as_ref().map(|value| value.to_string()),
        src: block.src.as_ref().map(|value| value.to_string()),
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn script_block_to_wasm(block: &vize_atelier_sfc::SfcScriptBlock<'_>) -> SfcScriptBlockWasm {
    SfcScriptBlockWasm {
        content: block.content.to_string(),
        loc: loc_to_wasm(&block.loc),
        lang: block.lang.as_ref().map(|value| value.to_string()),
        src: block.src.as_ref().map(|value| value.to_string()),
        setup: block.setup,
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn style_block_to_wasm(block: &vize_atelier_sfc::SfcStyleBlock<'_>) -> SfcStyleBlockWasm {
    SfcStyleBlockWasm {
        content: block.content.to_string(),
        loc: loc_to_wasm(&block.loc),
        lang: block.lang.as_ref().map(|value| value.to_string()),
        src: block.src.as_ref().map(|value| value.to_string()),
        scoped: block.scoped,
        module: block.module.as_ref().map(|value| value.to_string()),
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn custom_block_to_wasm(block: &vize_atelier_sfc::SfcCustomBlock<'_>) -> SfcCustomBlockWasm {
    SfcCustomBlockWasm {
        r#type: block.block_type.to_string(),
        content: block.content.to_string(),
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn descriptor_to_wasm(descriptor: &SfcDescriptor<'_>) -> SfcDescriptorWasm {
    SfcDescriptorWasm {
        filename: descriptor.filename.to_string(),
        source: descriptor.source.to_string(),
        template: descriptor.template.as_ref().map(template_block_to_wasm),
        script: descriptor.script.as_ref().map(script_block_to_wasm),
        script_setup: descriptor.script_setup.as_ref().map(script_block_to_wasm),
        styles: descriptor.styles.iter().map(style_block_to_wasm).collect(),
        custom_blocks: descriptor
            .custom_blocks
            .iter()
            .map(custom_block_to_wasm)
            .collect(),
        css_vars: descriptor
            .css_vars
            .iter()
            .map(|value| value.to_string())
            .collect(),
        slotted: descriptor.slotted,
        should_force_reload: descriptor.should_force_reload,
    }
}

/// WASM Compiler instance
#[wasm_bindgen]
pub struct Compiler;

#[wasm_bindgen]
impl Compiler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Compiler
    }

    /// Compile template to VDom render function
    #[wasm_bindgen]
    pub fn compile(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);

        match compile_internal(template, &parsed.options, false, parsed.binding_metadata) {
            Ok(result) => to_json_js_value(&result),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Compile template to Vapor mode
    #[wasm_bindgen(js_name = "compileVapor")]
    pub fn compile_vapor(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);

        match compile_internal(template, &parsed.options, true, None) {
            Ok(result) => to_json_js_value(&result),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Parse template to AST
    #[wasm_bindgen]
    pub fn parse(&self, template: &str, _options: JsValue) -> Result<JsValue, JsValue> {
        let allocator = Bump::new();

        let (root, errors) = parse(&allocator, template);

        if !errors.is_empty() {
            return Err(JsValue::from_str(&format!("Parse errors: {:?}", errors)));
        }

        let ast = build_ast_json(&root);
        to_js_value(&ast)
    }

    /// Parse SFC (.vue file)
    #[wasm_bindgen(js_name = "parseSfc")]
    pub fn parse_sfc_method(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let filename: vize_carton::CompactString =
            js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "anonymous.vue".to_string())
                .into();

        let opts = SfcParseOptions {
            filename,
            ..Default::default()
        };

        match parse_sfc(source, opts) {
            Ok(descriptor) => to_json_js_value(&descriptor_to_wasm(&descriptor)),
            Err(e) => Err(JsValue::from_str(&e.message)),
        }
    }

    /// Compile CSS with LightningCSS
    #[wasm_bindgen(js_name = "compileCss")]
    pub fn compile_css_method(&self, css: &str, options: JsValue) -> Result<JsValue, JsValue> {
        use vize_atelier_sfc::compile_css;
        let opts = parse_css_options(options);
        let result = compile_css(css, &opts);
        to_js_value(&result)
    }

    /// Compile SFC template block
    #[wasm_bindgen(js_name = "compileSfc")]
    pub fn compile_sfc(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);
        let opts = parsed.options;

        let filename: vize_carton::CompactString = opts
            .filename
            .clone()
            .unwrap_or_else(|| "anonymous.vue".to_string())
            .into();

        let parse_opts = SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        };

        // Parse SFC
        let descriptor = match parse_sfc(source, parse_opts) {
            Ok(d) => d,
            Err(e) => return Err(JsValue::from_str(&e.message)),
        };

        // Detect vapor mode from script setup attrs or options
        let has_vapor_attr = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.attrs.contains_key("vapor"))
            .unwrap_or(false)
            || descriptor
                .script
                .as_ref()
                .map(|s| s.attrs.contains_key("vapor"))
                .unwrap_or(false);
        let use_vapor = has_vapor_attr || opts.output_mode.as_deref() == Some("vapor");

        // Detect TypeScript from script lang attribute (for source detection)
        let source_is_ts = descriptor
            .script_setup
            .as_ref()
            .and_then(|s| s.lang.as_ref())
            .map(|l| l == "ts" || l == "tsx")
            .unwrap_or(false)
            || descriptor
                .script
                .as_ref()
                .and_then(|s| s.lang.as_ref())
                .map(|l| l == "ts" || l == "tsx")
                .unwrap_or(false);

        // Determine output format: preserve TypeScript or downcompile to JavaScript
        // script_ext option: "preserve" keeps TypeScript, "downcompile" (default) transpiles to JS
        let output_is_ts = opts
            .script_ext
            .as_deref()
            .map(|ext| ext == "preserve")
            .unwrap_or(false); // Default to downcompile (transpile to JS)

        // Update opts with source detection for backwards compatibility
        let mut opts = opts;
        if source_is_ts {
            opts.is_ts = Some(true);
        }

        // Compile template if present
        let template_result = if let Some(template) = &descriptor.template {
            match compile_internal(&template.content, &opts, use_vapor, None) {
                Ok(r) => Some(r),
                Err(e) => return Err(JsValue::from_str(&e)),
            }
        } else {
            None
        };

        // Full SFC compilation using sfc_compile
        // Use output_is_ts to control whether TypeScript is preserved or transpiled
        let sfc_opts = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                is_ts: output_is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                scoped: descriptor.styles.iter().any(|s| s.scoped),
                ssr: opts.ssr.unwrap_or(false),
                is_ts: output_is_ts,
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename,
                scoped: descriptor.styles.iter().any(|s| s.scoped),
                ..Default::default()
            },
            vapor: use_vapor,
            scope_id: None,
        };

        // Compile the full SFC
        let sfc_result = match sfc_compile(&descriptor, sfc_opts) {
            Ok(r) => r,
            Err(e) => return Err(JsValue::from_str(&e.message)),
        };

        // Build result with compiled script code
        // Convert descriptor to owned for serialization
        let binding_metadata = sfc_result
            .bindings
            .as_ref()
            .and_then(|b| serde_json::to_value(&b.bindings).ok());

        let result = SfcWasmResult {
            descriptor: descriptor_to_wasm(&descriptor),
            template: template_result,
            script: SfcScriptResult {
                code: sfc_result.code.into(),
                bindings: sfc_result
                    .bindings
                    .map(|b| serde_json::to_value(&b).unwrap_or_default()),
            },
            css: sfc_result.css.map(Into::into),
            errors: sfc_result
                .errors
                .into_iter()
                .map(|e| e.message.into())
                .collect(),
            warnings: sfc_result
                .warnings
                .into_iter()
                .map(|e| e.message.into())
                .collect(),
            binding_metadata,
        };

        to_json_js_value(&result)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal compile function
fn compile_internal(
    template: &str,
    opts: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
) -> Result<CompileResult, String> {
    let allocator = Bump::new();

    // SSR mode - use dedicated SSR compiler
    if opts.ssr.unwrap_or(false) && !vapor && binding_metadata.is_none() {
        let (root, errors, result) = ssr_compile(&allocator, template);

        if !errors.is_empty() {
            return Err(format!("SSR compile errors: {:?}", errors));
        }

        // Collect helpers
        let helpers: Vec<String> = root.helpers.iter().map(|h| h.name().to_string()).collect();

        // Build AST JSON
        let ast = build_ast_json(&root);

        return Ok(CompileResult {
            code: result.code.to_string(),
            preamble: result.preamble.to_string(),
            ast,
            map: None,
            helpers,
            templates: None,
        });
    }

    if vapor {
        // Use actual Vapor compiler
        let vapor_opts = VaporCompilerOptions {
            prefix_identifiers: opts.prefix_identifiers.unwrap_or(false),
            ssr: opts.ssr.unwrap_or(false),
            ..Default::default()
        };
        let result = vapor_compile(&allocator, template, vapor_opts);

        if !result.error_messages.is_empty() {
            return Err(result
                .error_messages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n"));
        }

        return Ok(CompileResult {
            code: result.code.to_string(),
            preamble: String::new(),
            ast: serde_json::json!({}),
            map: None,
            helpers: vec![],
            templates: Some(
                result
                    .templates
                    .into_iter()
                    .map(|t| t.to_string())
                    .collect(),
            ),
        });
    }

    // VDOM mode - use vize_atelier_dom which includes proper v-model transform
    let has_binding_metadata = binding_metadata.is_some();
    let dom_opts = DomCompilerOptions {
        mode: match opts.mode.as_deref() {
            Some("module") => CodegenMode::Module,
            _ => CodegenMode::Function,
        },
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(has_binding_metadata),
        hoist_static: opts.hoist_static.unwrap_or(has_binding_metadata),
        cache_handlers: opts.cache_handlers.unwrap_or(has_binding_metadata),
        scope_id: opts.scope_id.clone().map(|s| s.into()),
        ssr: opts.ssr.unwrap_or(false),
        source_map: opts.source_map.unwrap_or(false),
        is_ts: opts.is_ts.unwrap_or(false),
        binding_metadata,
        inline: has_binding_metadata,
        ..Default::default()
    };

    let (root, errors, result) = compile_template_with_options(&allocator, template, dom_opts);

    if !errors.is_empty() {
        return Err(format!("Compile errors: {:?}", errors));
    }

    // Collect helpers
    let helpers: Vec<String> = root.helpers.iter().map(|h| h.name().to_string()).collect();

    // Build AST JSON
    let ast = build_ast_json(&root);

    Ok(CompileResult {
        code: result.code.to_string(),
        preamble: result.preamble.to_string(),
        ast,
        map: None,
        helpers,
        templates: None,
    })
}

/// Build AST JSON from root node
fn build_ast_json(root: &vize_atelier_core::RootNode<'_>) -> serde_json::Value {
    use vize_atelier_core::TemplateChildNode;

    fn build_children(children: &[TemplateChildNode<'_>]) -> Vec<serde_json::Value> {
        children
            .iter()
            .map(|child| build_child_json(child))
            .collect()
    }

    fn build_child_json(child: &TemplateChildNode<'_>) -> serde_json::Value {
        match child {
            TemplateChildNode::Element(el) => {
                let props: Vec<serde_json::Value> = el
                    .props
                    .iter()
                    .map(|prop| match prop {
                        vize_atelier_core::PropNode::Attribute(attr) => serde_json::json!({
                            "type": "ATTRIBUTE",
                            "name": attr.name.as_str(),
                            "value": attr.value.as_ref().map(|v| v.content.as_str()),
                        }),
                        vize_atelier_core::PropNode::Directive(dir) => serde_json::json!({
                            "type": "DIRECTIVE",
                            "name": dir.name.as_str(),
                            "arg": dir.arg.as_ref().map(|a| match a {
                                vize_atelier_core::ExpressionNode::Simple(exp) => exp.content.as_str().to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "exp": dir.exp.as_ref().map(|e| match e {
                                vize_atelier_core::ExpressionNode::Simple(exp) => exp.content.as_str().to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "modifiers": dir.modifiers.iter().map(|m: &vize_atelier_core::SimpleExpressionNode| m.content.as_str()).collect::<Vec<_>>(),
                        }),
                    })
                    .collect();

                serde_json::json!({
                    "type": "ELEMENT",
                    "tag": el.tag.as_str(),
                    "tagType": format!("{:?}", el.tag_type),
                    "props": props,
                    "children": build_children(&el.children),
                    "isSelfClosing": el.is_self_closing,
                })
            }
            TemplateChildNode::Text(text) => serde_json::json!({
                "type": "TEXT",
                "content": text.content.as_str(),
            }),
            TemplateChildNode::Comment(comment) => serde_json::json!({
                "type": "COMMENT",
                "content": comment.content.as_str(),
            }),
            TemplateChildNode::Interpolation(interp) => serde_json::json!({
                "type": "INTERPOLATION",
                "content": match &interp.content {
                    vize_atelier_core::ExpressionNode::Simple(exp) => exp.content.as_str(),
                    _ => "<compound>",
                }
            }),
            _ => serde_json::json!({
                "type": "UNKNOWN"
            }),
        }
    }

    let children = build_children(&root.children);

    serde_json::json!({
        "type": "ROOT",
        "children": children,
        "helpers": root.helpers.iter().map(|h| h.name()).collect::<Vec<_>>(),
        "components": root.components.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "directives": root.directives.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
    })
}

/// Compile template to VDom (free function)
#[wasm_bindgen]
pub fn compile(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile(template, options)
}

/// Compile template to Vapor mode (free function)
#[wasm_bindgen(js_name = "compileVapor")]
pub fn compile_vapor_fn(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_vapor(template, options)
}

/// Parse template to AST (free function)
#[wasm_bindgen(js_name = "parseTemplate")]
pub fn parse_template(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse(template, options)
}

/// Parse SFC (free function)
#[wasm_bindgen(js_name = "parseSfc")]
pub fn parse_sfc_fn(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse_sfc_method(source, options)
}

/// Compile SFC (free function)
#[wasm_bindgen(js_name = "compileSfc")]
pub fn compile_sfc_fn(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_sfc(source, options)
}

/// Compile CSS (free function)
#[wasm_bindgen(js_name = "compileCss")]
pub fn compile_css_fn(css: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_css_method(css, options)
}
