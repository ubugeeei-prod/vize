//! Top-level Vapor compilation entry points.
//!
//! Wires together parsing, the core transform lane, Vapor IR lowering, and
//! code generation behind the public `compile_vapor*` functions.

mod entry;

use crate::lower as vapor_lower;
use crate::s3::{self, VaporS3BridgeOptions, VaporS3BridgeStatus};
use vize_atelier_core::{
    CompilerError, Namespace,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode, TransformOptions},
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_carton::{Allocator, String};

pub use entry::{
    compile_vapor, compile_vapor_with_custom_elements_and_template_syntax,
    compile_vapor_with_custom_elements_template_syntax_and_diagnostics,
    compile_vapor_with_custom_elements_template_syntax_and_experimental_options,
    compile_vapor_with_custom_elements_template_syntax_diagnostics_and_experimental_options,
    compile_vapor_with_diagnostics, compile_vapor_with_experimental_options,
    compile_vapor_with_template_syntax, compile_vapor_with_template_syntax_and_diagnostics,
    compile_vapor_with_template_syntax_and_experimental_options,
};
#[allow(deprecated)]
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

/// Vapor compilation result
#[derive(Debug)]
pub struct VaporCompileResult {
    /// Generated code
    pub code: String,
    /// Template strings for static parts
    pub templates: Vec<String>,
    /// Source Map v3 JSON for the generated render code.
    pub map: Option<String>,
    /// Error messages during compilation
    pub error_messages: Vec<String>,
}

fn compile_vapor_inner<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    vize_carton::ensure_sufficient_stack(|| {
        compile_vapor_inner_with_stack(
            allocator,
            source,
            options,
            template_syntax,
            custom_elements,
            experimental_options,
        )
    })
}

fn compile_vapor_inner_with_stack<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    // Parse
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        experimental_in_tag_comments: options.experimental_in_tag_comments,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        ..ParserOptions::default()
    };
    let (mut root, errors) = parse_with_options_custom_elements_and_template_syntax(
        allocator,
        source,
        parser_opts,
        custom_elements.clone(),
        template_syntax,
    );
    let parser_diagnostics = errors.to_vec();

    let fatal: std::vec::Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
    if !fatal.is_empty() {
        return (
            VaporCompileResult {
                code: String::default(),
                templates: Vec::new(),
                map: None,
                error_messages: fatal.iter().map(|e| e.message.clone()).collect(),
            },
            parser_diagnostics,
        );
    }

    let s3_bridge_status = s3::lower_source_for_vapor(
        allocator,
        source,
        VaporS3BridgeOptions {
            ssr: options.ssr,
            custom_renderer: options.custom_renderer,
            experimental_in_tag_comments: options.experimental_in_tag_comments,
            experimental_patterned_template: options.experimental_patterned_template,
            template_syntax,
            has_custom_elements: !custom_elements.is_empty(),
        },
    );

    // Transform to Vapor IR
    let binding_metadata = options.binding_metadata.clone();
    let transform_opts = TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        ssr: options.ssr,
        binding_metadata: binding_metadata.clone(),
        inline: options.inline,
        vapor: true,
        custom_renderer: options.custom_renderer,
        experimental_patterned_template: options.experimental_patterned_template,
        ..Default::default()
    };
    transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
        allocator,
        &mut root,
        transform_opts,
        None,
        custom_elements,
        template_syntax.is_quirks(),
        None,
    );

    // Lower to Vapor IR
    let (ir, mut transform_diagnostics) =
        vapor_lower::transform_to_ir_with_diagnostics(allocator, &root, source);
    if let VaporS3BridgeStatus::Rejected(diagnostics) = s3_bridge_status {
        transform_diagnostics.extend(diagnostics);
    }

    // Generate Vapor code
    let result = crate::generate::generate_vapor_with_options_and_experimentals(
        &ir,
        binding_metadata.as_ref(),
        crate::generate::VaporGenerateOptions::default(),
        crate::generate::VaporGenerateExperimentalOptions {
            component_name: experimental_options.component_name.as_deref(),
            self_component: experimental_options.self_component,
            source_map: experimental_options.source_map,
            source_map_filename: experimental_options.source_map_filename.as_deref(),
        },
    );

    (
        VaporCompileResult {
            code: result.code,
            templates: result.templates,
            map: result.map,
            error_messages: transform_diagnostics,
        },
        parser_diagnostics,
    )
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
