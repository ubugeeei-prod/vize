//! Compiler options.

use vize_s0::String;
use vize_s0::config::VueVersion;

mod bindings;
mod custom_elements;

pub use bindings::{BindingMetadata, BindingType};
pub use custom_elements::CustomElementMatcher;

/// Parse mode for the tokenizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// Platform-agnostic mode
    #[default]
    Base,
    /// HTML mode with special handling for certain tags
    Html,
    /// SFC mode for parsing .vue files
    Sfc,
}

/// Text mode for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextMode {
    /// Normal text parsing (default)
    #[default]
    Data,
    /// RCDATA (e.g., textarea, title)
    RcData,
    /// Raw text (e.g., script, style)
    RawText,
    /// CDATA section
    CData,
    /// Attribute value
    AttributeValue,
}

/// Template syntax compatibility mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum TemplateSyntaxMode {
    /// Accept common recoverable template syntax issues with warnings and rewrite them.
    #[default]
    Standard,
    /// Report recoverable template syntax issues as fatal errors.
    Strict,
    /// Preserve template syntax compatibility quirks without additional warnings.
    Quirks,
}

impl TemplateSyntaxMode {
    /// Whether template syntax quirks should be enabled.
    #[must_use]
    pub fn is_quirks(self) -> bool {
        matches!(self, Self::Quirks)
    }
}

/// Parser options
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Parse mode
    pub mode: ParseMode,
    /// Whether to trim whitespace
    pub whitespace: WhitespaceStrategy,
    /// Custom delimiters for interpolation (default: ["{{", "}}"])
    pub delimiters: (String, String),
    /// Whether in pre tag
    pub is_pre_tag: fn(&str) -> bool,
    /// Whether is a native tag
    pub is_native_tag: Option<fn(&str) -> bool>,
    /// Whether is a custom element
    pub is_custom_element: Option<fn(&str) -> bool>,
    /// Whether the template targets a custom renderer instead of the DOM.
    ///
    /// When enabled, lowercase non-HTML tags default to renderer-native
    /// elements instead of Vue component resolution.
    pub custom_renderer: bool,
    pub is_void_tag: fn(&str) -> bool,
    pub get_namespace: fn(&str, Option<&str>) -> crate::Namespace,
    /// Error handler
    pub on_error: Option<fn(crate::CompilerError)>,
    /// Warning handler
    pub on_warn: Option<fn(crate::CompilerError)>,
    pub comments: bool,
    pub experimental_in_tag_comments: bool,
    /// Vue dialect the source is written in, resolved once per file from
    /// `vue.version`. Defaults to [`VueVersion::V3`]; any legacy line is opt-in
    /// behind the `legacy` cargo feature. Parsing/tokenizing only consults this
    /// at cold decision points, and the Vue 3 default keeps the hot path
    /// byte-identical (see `vize_armature::legacy`). This PR threads the signal;
    /// it does not yet branch any behavior on it.
    pub dialect: VueVersion,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            mode: ParseMode::Base,
            whitespace: WhitespaceStrategy::Condense,
            delimiters: (String::from("{{"), String::from("}}")),
            is_pre_tag: |_| false,
            is_native_tag: None,
            is_custom_element: None,
            custom_renderer: false,
            is_void_tag: vize_s0::is_void_tag,
            get_namespace: |_, _| crate::Namespace::Html,
            on_error: None,
            on_warn: None,
            comments: true,
            experimental_in_tag_comments: false,
            dialect: VueVersion::V3,
        }
    }
}

/// Whitespace handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhitespaceStrategy {
    /// Condense whitespace (default)
    #[default]
    Condense,
    /// Preserve all whitespace
    Preserve,
}

/// Transform options
#[derive(Debug, Clone)]
pub struct TransformOptions {
    /// Filename for error messages
    pub filename: String,
    /// Whether to prefix identifiers
    pub prefix_identifiers: bool,
    /// Whether to hoist static nodes
    pub hoist_static: bool,
    /// Whether to cache handlers
    pub cache_handlers: bool,
    /// Scope ID for scoped CSS
    pub scope_id: Option<String>,
    /// Whether in SSR mode
    pub ssr: bool,
    /// Whether SSR optimize is enabled
    pub ssr_css_vars: Option<String>,
    /// Binding metadata from script setup
    pub binding_metadata: Option<BindingMetadata>,
    /// Inline mode
    pub inline: bool,
    /// Whether is TypeScript
    pub is_ts: bool,
    /// Whether in Vapor mode (skip v-model expansion)
    pub vapor: bool,
    pub custom_renderer: bool,
    pub experimental_patterned_template: bool,
    /// Vue dialect the source is written in, resolved once per file from
    /// `vue.version`. Defaults to [`VueVersion::V3`]; any legacy line is opt-in
    /// behind the `legacy` cargo feature. Transforms only consult this at cold
    /// decision points, and the Vue 3 default keeps the hot path byte-identical
    /// (see `vize_armature::legacy`). This PR threads the signal; it does not
    /// yet branch any behavior on it.
    pub dialect: VueVersion,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            filename: String::from("template.vue"),
            prefix_identifiers: false,
            hoist_static: false,
            cache_handlers: false,
            scope_id: None,
            ssr: false,
            ssr_css_vars: None,
            binding_metadata: None,
            inline: false,
            is_ts: false,
            vapor: false,
            custom_renderer: false,
            experimental_patterned_template: false,
            dialect: VueVersion::V3,
        }
    }
}

/// Codegen options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Output mode
    pub mode: CodegenMode,
    /// Whether to prefix identifiers
    pub prefix_identifiers: bool,
    /// Whether to generate source map
    pub source_map: bool,
    /// Filename for source map
    pub filename: String,
    /// Current SFC component name for self-reference resolution
    pub component_name: Option<String>,
    /// Scope ID for scoped CSS
    pub scope_id: Option<String>,
    /// Whether in SSR mode
    pub ssr: bool,
    /// Whether SSR optimize is enabled
    pub optimize_imports: bool,
    /// Runtime module name
    pub runtime_module_name: String,
    /// Runtime global name
    pub runtime_global_name: String,
    /// Whether is TypeScript
    pub is_ts: bool,
    /// Inline mode
    pub inline: bool,
    /// Binding metadata from script setup
    pub binding_metadata: Option<BindingMetadata>,
    /// Whether to cache inline event handlers
    pub cache_handlers: bool,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            mode: CodegenMode::Function,
            prefix_identifiers: false,
            source_map: false,
            filename: String::from("template.vue"),
            component_name: None,
            scope_id: None,
            ssr: false,
            optimize_imports: false,
            runtime_module_name: String::from("vue"),
            runtime_global_name: String::from("Vue"),
            is_ts: false,
            inline: false,
            binding_metadata: None,
            cache_handlers: false,
        }
    }
}

/// Codegen output mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodegenMode {
    /// Generate a function (default)
    #[default]
    Function,
    /// Generate an ES module
    Module,
}

/// Combined compiler options
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub parser: ParserOptions,
    pub transform: TransformOptions,
    pub codegen: CodegenOptions,
}

#[cfg(test)]
mod tests {
    use super::{
        CodegenMode, CodegenOptions, ParseMode, ParserOptions, TransformOptions, VueVersion,
        WhitespaceStrategy,
    };

    #[test]
    fn parser_options_default() {
        let opts = ParserOptions::default();
        assert_eq!(opts.mode, ParseMode::Base);
        assert_eq!(opts.whitespace, WhitespaceStrategy::Condense);
        assert_eq!(opts.delimiters.0.as_str(), "{{");
        assert_eq!(opts.delimiters.1.as_str(), "}}");
        assert!(opts.comments);
        assert!(opts.is_native_tag.is_none());
        assert!(opts.is_custom_element.is_none());
        assert!(opts.on_error.is_none());
        assert!(opts.on_warn.is_none());
        assert_eq!(opts.dialect, VueVersion::V3);
    }

    #[test]
    fn transform_options_default() {
        let opts = TransformOptions::default();
        assert!(!opts.prefix_identifiers);
        assert!(!opts.hoist_static);
        assert!(!opts.cache_handlers);
        assert!(!opts.ssr);
        assert!(!opts.is_ts);
        assert!(!opts.inline);
        assert!(opts.scope_id.is_none());
        assert!(opts.ssr_css_vars.is_none());
        assert!(opts.binding_metadata.is_none());
        assert_eq!(opts.dialect, VueVersion::V3);
    }

    #[test]
    fn codegen_options_default() {
        let opts = CodegenOptions::default();
        assert_eq!(opts.mode, CodegenMode::Function);
        assert_eq!(opts.runtime_module_name.as_str(), "vue");
        assert_eq!(opts.runtime_global_name.as_str(), "Vue");
        assert!(!opts.prefix_identifiers);
        assert!(!opts.source_map);
        assert!(!opts.ssr);
        assert!(!opts.is_ts);
        assert!(!opts.inline);
        assert!(opts.scope_id.is_none());
        assert!(opts.binding_metadata.is_none());
    }

    #[test]
    fn codegen_mode_serde() {
        let json_fn = serde_json::to_string(&CodegenMode::Function).unwrap();
        assert_eq!(json_fn, "\"function\"");
        let json_mod = serde_json::to_string(&CodegenMode::Module).unwrap();
        assert_eq!(json_mod, "\"module\"");

        let deserialized: CodegenMode = serde_json::from_str("\"function\"").unwrap();
        assert_eq!(deserialized, CodegenMode::Function);
        let deserialized: CodegenMode = serde_json::from_str("\"module\"").unwrap();
        assert_eq!(deserialized, CodegenMode::Module);
    }
}
