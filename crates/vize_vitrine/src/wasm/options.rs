//! Parsing of compiler and CSS options from JavaScript option objects.

use wasm_bindgen::prelude::*;

use crate::CompilerOptions;
use vize_atelier_sfc::{CssCompileOptions, CssTargets};

// This is the canonical public `@vizejs/wasm` compiler-option inventory. The
// declaration drift test reads the same entries and checks both directions:
// every entry is parsed below and every `CompilerOptions` property is listed.
macro_rules! define_compiler_option_inventory {
    ($($variant:ident => ($name:literal, $ts_type:literal),)+) => {
        #[derive(Clone, Copy)]
        enum CompilerOption {
            $($variant,)+
        }

        impl CompilerOption {
            #[inline]
            const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }
        }
    };
}

define_compiler_option_inventory! {
    Mode => ("mode", r#""module" | "function""#),
    PrefixIdentifiers => ("prefixIdentifiers", "boolean"),
    HoistStatic => ("hoistStatic", "boolean"),
    CacheHandlers => ("cacheHandlers", "boolean"),
    ScopeId => ("scopeId", "string"),
    Ssr => ("ssr", "boolean"),
    SourceMap => ("sourceMap", "boolean"),
    Filename => ("filename", "string"),
    OutputMode => ("outputMode", r#""vdom" | "vapor""#),
    IsTs => ("isTs", "boolean"),
    CustomRenderer => ("customRenderer", "boolean"),
    CustomElements => ("customElements", "string[]"),
    TemplateSyntax => ("templateSyntax", r#""standard" | "strict" | "quirks""#),
    ExperimentalInTagComments => ("experimentalInTagComments", "boolean"),
    ExperimentalPatternedTemplate => ("experimentalPatternedTemplate", "boolean"),
    RuntimeModuleName => ("runtimeModuleName", "string"),
    RuntimeGlobalName => ("runtimeGlobalName", "string"),
    ScriptExt => ("scriptExt", r#""preserve" | "downcompile""#),
    BindingMetadata => ("bindingMetadata", "BindingMetadata"),
    VueParserQuirks => ("vueParserQuirks", "boolean"),
}

pub(crate) struct ParsedCompilerOptions {
    pub(crate) options: CompilerOptions,
    pub(crate) binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
}

pub(super) fn resolve_template_syntax_compat(
    explicit: Option<String>,
    vue_parser_quirks: Option<bool>,
) -> Option<String> {
    explicit.or_else(|| {
        vue_parser_quirks
            .filter(|enabled| *enabled)
            .map(|_| "quirks".to_string())
    })
}

pub(crate) fn parse_compiler_options(options: &JsValue) -> ParsedCompilerOptions {
    let get_string = |option: CompilerOption| {
        js_sys::Reflect::get(options, &JsValue::from_str(option.name()))
            .ok()
            .and_then(|value| value.as_string())
    };

    let get_bool = |option: CompilerOption| {
        js_sys::Reflect::get(options, &JsValue::from_str(option.name()))
            .ok()
            .and_then(|value| value.as_bool())
    };
    let get_string_array = |option: CompilerOption| {
        let value = js_sys::Reflect::get(options, &JsValue::from_str(option.name())).ok()?;
        if value.is_null() || value.is_undefined() || !js_sys::Array::is_array(&value) {
            return None;
        }
        let array = js_sys::Array::from(&value);
        // Keep every string element and ignore the rest: a single non-string
        // entry must not silently discard the whole option. The `Vec` grows on
        // demand instead of reserving from the JS-controlled array length.
        let values: Vec<String> = array.iter().filter_map(|value| value.as_string()).collect();
        Some(values)
    };

    let binding_metadata = js_sys::Reflect::get(
        options,
        &JsValue::from_str(CompilerOption::BindingMetadata.name()),
    )
    .ok()
    .and_then(|value| {
        if value.is_null() || value.is_undefined() {
            return None;
        }
        let json = js_sys::JSON::stringify(&value).ok()?.as_string()?;
        serde_json::from_str(&json).ok()
    });

    let template_syntax = get_string(CompilerOption::TemplateSyntax);
    let vue_parser_quirks = if template_syntax.is_none() {
        get_bool(CompilerOption::VueParserQuirks)
    } else {
        None
    };
    let template_syntax = resolve_template_syntax_compat(template_syntax, vue_parser_quirks);

    ParsedCompilerOptions {
        options: CompilerOptions {
            mode: get_string(CompilerOption::Mode),
            prefix_identifiers: get_bool(CompilerOption::PrefixIdentifiers),
            hoist_static: get_bool(CompilerOption::HoistStatic),
            cache_handlers: get_bool(CompilerOption::CacheHandlers),
            scope_id: get_string(CompilerOption::ScopeId),
            ssr: get_bool(CompilerOption::Ssr),
            source_map: get_bool(CompilerOption::SourceMap),
            filename: get_string(CompilerOption::Filename),
            output_mode: get_string(CompilerOption::OutputMode),
            is_ts: get_bool(CompilerOption::IsTs),
            custom_renderer: get_bool(CompilerOption::CustomRenderer),
            custom_elements: get_string_array(CompilerOption::CustomElements),
            template_syntax,
            experimental_in_tag_comments: get_bool(CompilerOption::ExperimentalInTagComments),
            experimental_patterned_template: get_bool(
                CompilerOption::ExperimentalPatternedTemplate,
            ),
            // Reserved by the shared native type, but no WASM compiler stage
            // implements it. Keep it out of the public WASM option inventory.
            experimental_server_script: None,
            runtime_module_name: get_string(CompilerOption::RuntimeModuleName),
            runtime_global_name: get_string(CompilerOption::RuntimeGlobalName),
            script_ext: get_string(CompilerOption::ScriptExt),
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
