//! SFC type definitions.
//!
//! Zero-copy design using borrowed strings for maximum parsing performance.

#[doc(inline)]
pub use vize_croquis::sfc::{
    BindingMetadata, BindingType, BlockLocation, PadOption, SfcCustomBlock, SfcDescriptor,
    SfcError, SfcParseOptions, SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock,
};

use serde::{Deserialize, Serialize};
use vize_carton::{FxHashMap, String};

#[derive(Debug, Clone, Default)]
pub(crate) struct CssModuleMapping {
    pub name: String,
    pub exports: FxHashMap<String, String>,
}

pub(crate) fn css_modules_object_literal(
    css_modules: &[CssModuleMapping],
    base_indent: &str,
) -> String {
    let mut modules = css_modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));

    let mut out = String::from("{\n");
    for (module_index, module) in modules.iter().enumerate() {
        out.push_str(base_indent);
        out.push_str("  ");
        out.push_str(&json_string(&module.name));
        out.push_str(": ");

        if module.exports.is_empty() {
            out.push_str("{}");
        } else {
            let mut exports = module.exports.iter().collect::<Vec<_>>();
            exports.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));

            out.push_str("{\n");
            for (export_index, (original, compiled)) in exports.iter().enumerate() {
                out.push_str(base_indent);
                out.push_str("    ");
                out.push_str(&json_string(original));
                out.push_str(": ");
                out.push_str(&json_string(compiled));
                if export_index + 1 < exports.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(base_indent);
            out.push_str("  }");
        }

        if module_index + 1 < modules.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(base_indent);
    out.push('}');
    out
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value)
        .map(|value| String::from(value.as_str()))
        .unwrap_or_else(|_| {
            let mut escaped = String::with_capacity(value.len() + 2);
            escaped.push('"');
            escaped.push_str(value);
            escaped.push('"');
            escaped
        })
}

/// SFC compilation options
#[derive(Debug, Clone, Default)]
pub struct SfcCompileOptions {
    /// SFC parse options
    pub parse: SfcParseOptions,

    /// Script compile options
    pub script: ScriptCompileOptions,

    /// Template compile options
    pub template: TemplateCompileOptions,

    /// Style compile options
    pub style: StyleCompileOptions,

    /// Whether to compile the SFC in Vapor mode
    pub vapor: bool,

    /// External scope ID (8-char hex, without "data-v-" prefix).
    /// When provided, this scope ID is used instead of generating one from the filename.
    /// This ensures consistency with the JS-side scope ID generation (SHA-256).
    pub scope_id: Option<String>,
}

/// Script compile options
#[derive(Debug, Clone, Default)]
pub struct ScriptCompileOptions {
    /// ID for scoped CSS
    pub id: Option<String>,

    /// Whether to emit standalone output with an inline template.
    pub inline_template: bool,

    /// Whether to use TypeScript
    pub is_ts: bool,

    /// Reactive transform
    pub reactive_props_destructure: bool,

    /// Props destructure
    pub props_destructure: PropsDestructure,

    /// Define model options
    pub define_model: bool,
}

/// Props destructure mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PropsDestructure {
    /// Disabled (error)
    #[default]
    False,
    /// Enabled
    True,
    /// Error on use
    Error,
}

/// Template compile options
#[derive(Debug, Clone, Default)]
pub struct TemplateCompileOptions {
    /// ID for scoped CSS
    pub id: Option<String>,

    /// Whether SSR mode
    pub ssr: bool,

    /// SSR CSS vars
    pub ssr_css_vars: Option<String>,

    /// Scoped
    pub scoped: bool,

    /// Is prod mode
    pub is_prod: bool,

    /// Whether TypeScript mode
    pub is_ts: bool,

    /// Whether the template targets a custom renderer instead of the DOM.
    pub custom_renderer: bool,

    /// Vue dialect resolved once per file from `vue.version`. Defaults to
    /// [`VueVersion::V3`] and is threaded into the DOM/SSR compiler options so
    /// it reaches the parser/transform layer. Legacy lines are opt-in behind
    /// the `legacy` cargo feature; this PR only plumbs the signal.
    pub dialect: vize_carton::config::VueVersion,

    /// Compiler options
    pub compiler_options: Option<vize_atelier_dom::DomCompilerOptions>,
}

/// Style compile options
#[derive(Debug, Clone, Default)]
pub struct StyleCompileOptions {
    /// ID for scoped CSS
    pub id: String,

    /// Whether scoped
    pub scoped: bool,

    /// Whether trim
    pub trim: bool,

    /// Source map
    pub source_map: bool,

    /// Preprocessor language
    pub preprocessor_lang: Option<String>,

    /// Custom data attributes to add
    pub data_attrs: Vec<String>,
}

/// SFC compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcCompileResult {
    /// Compiled JavaScript code
    pub code: String,

    /// Compiled CSS (from all style blocks)
    pub css: Option<String>,

    /// Source map
    pub map: Option<serde_json::Value>,

    /// Errors
    pub errors: Vec<SfcError>,

    /// Warnings
    pub warnings: Vec<SfcError>,

    /// Binding metadata
    pub bindings: Option<BindingMetadata>,

    /// Compile-time macro artifacts extracted from script blocks.
    #[serde(default)]
    pub macro_artifacts: Vec<SfcMacroArtifact>,
}

/// Compile-time macro artifact extracted from an SFC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcMacroArtifact {
    /// Stable artifact kind.
    pub kind: String,

    /// Macro call name.
    pub name: String,

    /// Full macro call source.
    pub source: String,

    /// Extracted macro payload source.
    pub content: String,

    /// Ready-to-load virtual module code, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_code: Option<String>,

    /// Absolute start offset in the original SFC source.
    pub start: usize,

    /// Absolute end offset in the original SFC source.
    pub end: usize,
}
