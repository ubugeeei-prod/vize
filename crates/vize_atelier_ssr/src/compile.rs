use crate::{SsrCodegenContext, SsrCodegenResult, SsrCompilerOptions};
use vize_atelier_core::{
    CompilerError, Namespace, RootNode,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{CustomElementMatcher, TemplateSyntaxMode},
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_s0::{Allocator, String, profile};

/// Compile a Vue template for SSR with default options
pub fn compile_ssr<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_with_options(allocator, source, SsrCompilerOptions::default())
}

/// Compile a Vue template for SSR with custom options
pub fn compile_ssr_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
    )
}

/// Compile a Vue template for SSR with Vue parser quirk compatibility.
#[deprecated(note = "use compile_ssr_with_template_syntax instead")]
pub fn compile_ssr_with_vue_parser_quirks<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        CustomElementMatcher::default(),
    )
}

/// Compile a Vue template for SSR with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_ssr_with_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        template_syntax,
        CustomElementMatcher::default(),
    )
}

/// Compile SSR with declarative custom-element patterns.
#[doc(hidden)]
pub fn compile_ssr_with_custom_elements_and_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, template_syntax, custom_elements)
}

fn compile_ssr_inner<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    let codegen_options = options.clone();
    let parser_opts = crate::stage_options::parser_options(&options);

    let (mut root, errors) = profile!(
        "atelier.ssr.template.parse",
        parse_with_options_custom_elements_and_template_syntax(
            allocator,
            source,
            parser_opts,
            custom_elements.clone(),
            template_syntax,
        )
    );
    if errors.iter().any(|e| !e.is_recoverable()) {
        return (
            root,
            errors.to_vec(),
            SsrCodegenResult {
                code: String::default(),
                preamble: String::default(),
            },
        );
    }

    let transform_opts = crate::stage_options::transform_options(&codegen_options);
    let transform_errors = profile!(
        "atelier.ssr.template.transform",
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            &mut root,
            transform_opts,
            options.croquis.map(|c| allocator.alloc_owned(*c)),
            custom_elements,
            template_syntax.is_quirks(),
            None,
        )
    );

    let mut errors = errors.to_vec();
    errors.extend(transform_errors);
    let codegen_ctx = SsrCodegenContext::new(allocator, &codegen_options, source);
    let codegen_result = profile!("atelier.ssr.template.codegen", codegen_ctx.generate(&root));

    (root, errors, codegen_result)
}

pub(crate) fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_s0::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_s0::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }
    if let Some(parent_tag) = parent {
        if vize_s0::is_svg_tag(parent_tag) && tag != "foreignObject" {
            return Namespace::Svg;
        }
        if vize_s0::is_math_ml_tag(parent_tag) && tag != "annotation-xml" && tag != "foreignObject"
        {
            return Namespace::MathMl;
        }
    }
    Namespace::Html
}
