//! Legacy template-AST compiler entry points.

use vize_armature::parse_with_options_and_template_syntax;
use vize_atelier_core::lane::{transform as do_transform, transform_with_template_syntax_quirks};
use vize_carton::{Bump, String, profile};
use vize_relief::{
    CompilerError, Namespace, ParserOptions, RootNode, TemplateSyntaxMode, TransformOptions,
};

use crate::{SsrCodegenContext, SsrCodegenResult, SsrCompilerOptions};

/// Compile a Vue template for SSR with default options.
pub fn compile_ssr<'a>(
    allocator: &'a Bump,
    source: &'a str,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_with_options(allocator, source, SsrCompilerOptions::default())
}

/// Compile a Vue template for SSR with custom options.
pub fn compile_ssr_with_options<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, TemplateSyntaxMode::Standard)
}

/// Compile a Vue template for SSR with Vue parser quirk compatibility.
#[deprecated(note = "use compile_ssr_with_template_syntax instead")]
pub fn compile_ssr_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, TemplateSyntaxMode::Quirks)
}

/// Compile a Vue template for SSR with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_ssr_with_template_syntax<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, template_syntax)
}

/// Transform and emit SSR output from an already parsed Relief root.
#[doc(hidden)]
pub fn compile_ssr_root_with_template_syntax<'a>(
    allocator: &'a Bump,
    root: RootNode<'a>,
    parse_errors: Vec<CompilerError>,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_root(allocator, root, parse_errors, options, template_syntax)
}

fn compile_ssr_inner<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        experimental_in_tag_comments: options.experimental_in_tag_comments,
        dialect: options.dialect,
        ..ParserOptions::default()
    };

    let (root, errors) = profile!(
        "atelier.ssr.template.parse",
        parse_with_options_and_template_syntax(allocator, source, parser_opts, template_syntax)
    );
    compile_ssr_root(allocator, root, errors.to_vec(), options, template_syntax)
}

fn compile_ssr_root<'a>(
    allocator: &'a Bump,
    mut root: RootNode<'a>,
    mut errors: Vec<CompilerError>,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    let codegen_options = options.clone();
    if errors.iter().any(|error| !error.is_recoverable()) {
        let codegen_result = SsrCodegenResult {
            code: String::default(),
            preamble: String::default(),
        };
        return (root, errors, codegen_result);
    }

    let transform_opts = TransformOptions {
        prefix_identifiers: true,
        hoist_static: false,
        cache_handlers: false,
        scope_id: codegen_options.scope_id.clone(),
        ssr: true,
        is_ts: codegen_options.is_ts,
        inline: codegen_options.inline,
        custom_renderer: codegen_options.custom_renderer,
        experimental_patterned_template: codegen_options.experimental_patterned_template,
        binding_metadata: codegen_options.binding_metadata.clone(),
        dialect: codegen_options.dialect,
        ..Default::default()
    };
    let analysis = options.croquis.map(|croquis| &*allocator.alloc(*croquis));
    let transform_errors = profile!(
        "atelier.ssr.template.transform",
        if template_syntax.is_quirks() {
            transform_with_template_syntax_quirks(allocator, &mut root, transform_opts, analysis)
        } else {
            do_transform(allocator, &mut root, transform_opts, analysis)
        }
    );
    errors.extend(transform_errors.errors);

    let codegen_ctx = SsrCodegenContext::new(allocator, &codegen_options);
    let codegen_result = profile!("atelier.ssr.template.codegen", codegen_ctx.generate(&root));
    (root, errors, codegen_result)
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
