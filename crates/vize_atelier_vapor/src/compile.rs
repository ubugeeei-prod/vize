//! Top-level Vapor compilation entry points.
//!
//! Wires together parsing, the core transform lane, Vapor IR lowering, and
//! code generation behind the public `compile_vapor*` functions.

use crate::generate::generate_vapor;
use crate::lower as vapor_lower;
use vize_atelier_core::{
    CompilerError, Namespace,
    lane::{transform, transform_with_template_syntax_quirks},
    options::{ParserOptions, TemplateSyntaxMode, TransformOptions},
    parser::parse_with_options_and_template_syntax,
};
use vize_carton::{Bump, String};

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
}

/// Vapor compilation result
#[derive(Debug)]
pub struct VaporCompileResult {
    /// Generated code
    pub code: String,
    /// Template strings for static parts
    pub templates: Vec<String>,
    /// Error messages during compilation
    pub error_messages: Vec<String>,
}

/// Compile a Vue template to Vapor mode
pub fn compile_vapor<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
) -> VaporCompileResult {
    compile_vapor_inner(allocator, source, options, TemplateSyntaxMode::Standard).0
}

/// Compile a Vue template to Vapor mode with Vue parser quirk compatibility.
#[deprecated(note = "use compile_vapor_with_template_syntax instead")]
pub fn compile_vapor_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
) -> VaporCompileResult {
    compile_vapor_inner(allocator, source, options, TemplateSyntaxMode::Quirks).0
}

/// Compile a Vue template to Vapor mode with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_vapor_with_template_syntax<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> VaporCompileResult {
    compile_vapor_inner(allocator, source, options, template_syntax).0
}

/// Compile a Vue template to Vapor mode and return parser diagnostics.
#[doc(hidden)]
pub fn compile_vapor_with_diagnostics<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(allocator, source, options, TemplateSyntaxMode::Standard)
}

/// Compile a Vue template to Vapor mode with Vue parser quirks and return parser diagnostics.
#[doc(hidden)]
#[deprecated(note = "use compile_vapor_with_template_syntax_and_diagnostics instead")]
pub fn compile_vapor_with_vue_parser_quirks_and_diagnostics<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(allocator, source, options, TemplateSyntaxMode::Quirks)
}

/// Compile a Vue template to Vapor mode with template syntax mode and return parser diagnostics.
#[doc(hidden)]
pub fn compile_vapor_with_template_syntax_and_diagnostics<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(allocator, source, options, template_syntax)
}

fn compile_vapor_inner<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    // Parse
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        ..ParserOptions::default()
    };
    let (mut root, errors) =
        parse_with_options_and_template_syntax(allocator, source, parser_opts, template_syntax);
    let parser_diagnostics = errors.to_vec();

    let fatal: std::vec::Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
    if !fatal.is_empty() {
        return (
            VaporCompileResult {
                code: String::default(),
                templates: Vec::new(),
                error_messages: fatal.iter().map(|e| e.message.clone()).collect(),
            },
            parser_diagnostics,
        );
    }

    // Transform to Vapor IR
    let binding_metadata = options.binding_metadata.clone();
    let transform_opts = TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        ssr: options.ssr,
        binding_metadata: binding_metadata.clone(),
        inline: options.inline,
        vapor: true,
        custom_renderer: options.custom_renderer,
        ..Default::default()
    };
    if template_syntax.is_quirks() {
        transform_with_template_syntax_quirks(allocator, &mut root, transform_opts, None);
    } else {
        transform(allocator, &mut root, transform_opts, None);
    }

    // Lower to Vapor IR
    let (ir, transform_diagnostics) =
        vapor_lower::transform_to_ir_with_diagnostics(allocator, &root);

    // Generate Vapor code
    let result = generate_vapor(&ir, binding_metadata.as_ref());

    (
        VaporCompileResult {
            code: result.code,
            templates: result.templates,
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
