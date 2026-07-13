//! Full SFC compilation over an Atlas-owned parse-only template artifact.

use vize_relief::CodegenOptions;
use vize_relief::ReliefArtifact;
use vize_relief::TemplateSyntaxMode;

use crate::{
    compile_template::{
        TemplateBlockCompileContext, TemplateBlockCompileResult, VaporTemplateCompileContext,
        compile_template_block as compile_template_block_direct,
        compile_template_block_from_syntax,
        compile_template_block_vapor as compile_template_block_vapor_direct,
        compile_template_block_vapor_from_syntax,
    },
    types::{
        BindingMetadata, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
        SfcTemplateBlock, TemplateCompileOptions,
    },
};

/// Compile a descriptor while reusing its already parsed template syntax.
///
/// `None` is valid only for a descriptor without a template. This explicit
/// presence check prevents a stale or mismatched artifact graph from silently
/// falling back to the legacy parser.
pub fn compile_sfc_with_shared_syntax(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    syntax: Option<&ReliefArtifact>,
) -> Result<SfcCompileResult, SfcError> {
    if descriptor.template.is_some() != syntax.is_some() {
        return Err(SfcError {
            message: "SFC descriptor and Relief syntax disagree about template presence".into(),
            code: Some("INCONSISTENT_TEMPLATE_ARTIFACTS".into()),
            loc: descriptor
                .template
                .as_ref()
                .map(|template| template.loc.clone()),
        });
    }
    super::compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        syntax,
        CodegenOptions::default(),
    )
}

pub(super) fn compile_template_block(
    template: &SfcTemplateBlock,
    options: &TemplateCompileOptions,
    context: TemplateBlockCompileContext<'_>,
    template_syntax: TemplateSyntaxMode,
    syntax: Option<&ReliefArtifact>,
    codegen_options: &CodegenOptions,
) -> Result<TemplateBlockCompileResult, SfcError> {
    match syntax {
        Some(syntax) => compile_template_block_from_syntax(
            template,
            options,
            context,
            template_syntax,
            syntax,
            codegen_options,
        ),
        None => compile_template_block_direct(
            template,
            options,
            context,
            template_syntax,
            codegen_options,
        ),
    }
}

pub(super) fn compile_template_block_vapor(
    template: &SfcTemplateBlock,
    context: VaporTemplateCompileContext<'_>,
    bindings: Option<&BindingMetadata>,
    syntax: Option<&ReliefArtifact>,
) -> Result<TemplateBlockCompileResult, SfcError> {
    match syntax {
        Some(syntax) => {
            compile_template_block_vapor_from_syntax(template, syntax, context, bindings)
        }
        None => compile_template_block_vapor_direct(template, context, bindings),
    }
}
