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
    compile_vapor_with_sfc_context, compile_vapor_with_template_syntax,
    compile_vapor_with_template_syntax_and_diagnostics,
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
    /// Davinci A/B and baseline instrumentation: always select the retained
    /// (pre-S3) lane. Not a user option; production callers leave it unset.
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
    compile_vapor_inner_scoped(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        None,
    )
}

fn compile_vapor_inner_scoped<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
    scope_id: Option<&str>,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    vize_carton::ensure_sufficient_stack(|| {
        compile_vapor_inner_with_stack(
            allocator,
            source,
            options,
            template_syntax,
            custom_elements,
            experimental_options,
            scope_id,
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
    scope_id: Option<&str>,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    // The native lane parses the source through S1 itself. It admits only
    // sources the legacy parser reports nothing for (S1 keeps the tokenizer's
    // codes and S2 refuses every recovery rule; `s3/tests/parser_agreement.rs`
    // pins this over the fixture corpus and its malformed variants), so an
    // admitted source never builds the legacy tree it would discard.
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
            // Without prefixing, the retained lane keeps expression text as
            // authored and binding metadata only steers the shared generator.
            prefixed_binding_metadata: options.binding_metadata.is_some()
                && options.prefix_identifiers,
            retained_lane: options.davinci_retained_lane,
            inline: options.inline,
        },
    );
    if let VaporS3BridgeStatus::Accepted(artifact) = s3_bridge_status {
        debug_assert!(
            parse_with_options_custom_elements_and_template_syntax(
                allocator,
                source,
                parser_options(&options),
                custom_elements,
                template_syntax,
            )
            .1
            .is_empty(),
            "the native Vapor lane admitted a source the legacy parser diagnoses"
        );
        s3::record_accepted();
        let ir = artifact.into_ir(allocator, source, scope_id);
        return (
            generate(&ir, &options, &experimental_options, Vec::new()),
            std::vec::Vec::new(),
        );
    }

    let (mut root, errors) = parse_with_options_custom_elements_and_template_syntax(
        allocator,
        source,
        parser_options(&options),
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

    // A diagnosed source keeps the legacy lane whatever the bridge concluded.
    let s3_bridge_status = if parser_diagnostics.is_empty() {
        s3_bridge_status
    } else {
        VaporS3BridgeStatus::Legacy(s3::LegacyReason::SurfaceSemantics)
    };
    s3::record_selection(&s3_bridge_status);
    match s3_bridge_status {
        VaporS3BridgeStatus::Accepted(_) => {
            unreachable!("admitted sources return before the legacy parse")
        }
        VaporS3BridgeStatus::Rejected(error_messages) => {
            return (
                VaporCompileResult {
                    code: String::default(),
                    templates: Vec::new(),
                    map: None,
                    error_messages,
                },
                parser_diagnostics,
            );
        }
        VaporS3BridgeStatus::Legacy(_) => {}
    }

    // The explicitly selected legacy route retains its complete transforms.
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
    let transform_errors =
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            &mut root,
            transform_opts,
            None,
            custom_elements,
            template_syntax.is_quirks(),
            None,
        );
    let fatal: Vec<_> = transform_errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .collect();
    if !fatal.is_empty() {
        let mut diagnostics = parser_diagnostics;
        diagnostics.extend(transform_errors.iter().cloned());
        return (
            VaporCompileResult {
                code: String::default(),
                templates: Vec::new(),
                map: None,
                error_messages: fatal.iter().map(|error| error.message.clone()).collect(),
            },
            diagnostics,
        );
    }

    // Lower to Vapor IR
    let (ir, transform_diagnostics) =
        vapor_lower::transform_to_ir_with_scope_id(allocator, &root, source, scope_id);
    (
        generate(&ir, &options, &experimental_options, transform_diagnostics),
        parser_diagnostics,
    )
}

fn generate(
    ir: &crate::ir::RootIRNode<'_>,
    options: &VaporCompilerOptions,
    experimental_options: &VaporCompilerExperimentalOptions,
    error_messages: Vec<String>,
) -> VaporCompileResult {
    let result = crate::generate::generate_vapor_with_options_and_experimentals(
        ir,
        options.binding_metadata.as_ref(),
        crate::generate::VaporGenerateOptions::default(),
        crate::generate::VaporGenerateExperimentalOptions {
            component_name: experimental_options.component_name.as_deref(),
            self_component: experimental_options.self_component,
            source_map: experimental_options.source_map,
            source_map_filename: experimental_options.source_map_filename.as_deref(),
        },
    );

    VaporCompileResult {
        code: result.code,
        templates: result.templates,
        map: result.map,
        error_messages,
    }
}

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
