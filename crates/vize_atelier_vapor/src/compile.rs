//! Top-level Vapor compilation entry points.
//!
//! Wires together parsing, the core transform lane, Vapor IR lowering, and
//! code generation behind the public `compile_vapor*` functions.

#[cfg(feature = "davinci-benchmark")]
#[doc(hidden)]
pub mod benchmark;
mod entry;
pub(crate) mod native;
mod result;

pub use result::VaporCompileResult;

use crate::generate::spans::VaporSourceSpans;
use crate::l3::{self, VaporL3BridgeOptions, VaporL3BridgeStatus};
use crate::lower as vapor_lower;
use vize_atelier_core::{
    CompilerError, Namespace,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode, TransformOptions},
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_carton::cstr;
use vize_carton::{Allocator, String};
use vize_l0::{
    dump::capture::{CaptureOutcome, CaptureSink, NoCapture},
    level::Level,
};

pub use entry::{
    compile_vapor, compile_vapor_with_custom_elements_and_template_syntax,
    compile_vapor_with_custom_elements_template_syntax_and_diagnostics,
    compile_vapor_with_custom_elements_template_syntax_and_experimental_options,
    compile_vapor_with_custom_elements_template_syntax_diagnostics_and_experimental_options,
    compile_vapor_with_diagnostics, compile_vapor_with_experimental_options,
    compile_vapor_with_sfc_context, compile_vapor_with_sfc_context_and_capture,
    compile_vapor_with_template_syntax, compile_vapor_with_template_syntax_and_diagnostics,
    compile_vapor_with_template_syntax_and_experimental_options,
};
#[expect(deprecated, reason = "re-exported until their removal")]
pub use entry::{
    compile_vapor_with_vue_parser_quirks, compile_vapor_with_vue_parser_quirks_and_diagnostics,
};

/// Vapor compiler options
#[derive(Debug, Clone, Default)]
pub struct VaporCompilerOptions {
    /// Whether to prefix identifiers
    pub prefix_identifiers: bool,
    /// Whether in SSR mode
    pub ssr: bool,
    /// Binding metadata
    pub binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
    /// Whether to inline
    pub inline: bool,
    /// Whether the template targets a custom renderer instead of the DOM.
    pub custom_renderer: bool,
    /// Enable experimental Vue in-tag comments (`// ...`) inside opening tags.
    pub experimental_in_tag_comments: bool,
    /// Enable experimental `v-match` / `v-when` patterned template desugaring.
    pub experimental_patterned_template: bool,
    /// Davinci A/B and baseline instrumentation: always select the retained
    /// (pre-L3) lane. Not a user option; production callers leave it unset.
    #[doc(hidden)]
    pub davinci_retained_lane: bool,
}

/// Experimental Vapor compiler options kept separate from
/// [`VaporCompilerOptions`] so existing Rust struct literals remain
/// source-compatible.
#[derive(Debug, Clone, Default)]
pub struct VaporCompilerExperimentalOptions {
    /// Current SFC component name for self-reference resolution.
    pub component_name: Option<String>,
    /// Treat the reserved `<Self>` tag as a reference to the current SFC.
    pub self_component: bool,
    /// Generate a Source Map v3 document for Vapor render code.
    pub source_map: bool,
    /// Filename recorded in the Source Map v3 `file` and `sources` fields.
    pub source_map_filename: Option<String>,
}

mod inner;
use inner::{compile_vapor_inner, compile_vapor_inner_scoped, compile_vapor_inner_scoped_captured};

/// The legacy parser's configuration for a Vapor compile.
pub(crate) fn parser_options(options: &VaporCompilerOptions) -> ParserOptions {
    ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        experimental_in_tag_comments: options.experimental_in_tag_comments,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        ..ParserOptions::default()
    }
}

fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }

    if let Some(parent_tag) = parent {
        if vize_carton::is_svg_tag(parent_tag) && tag != "foreignObject" {
            return Namespace::Svg;
        }
        if vize_carton::is_math_ml_tag(parent_tag)
            && tag != "annotation-xml"
            && tag != "foreignObject"
        {
            return Namespace::MathMl;
        }
    }

    Namespace::Html
}
