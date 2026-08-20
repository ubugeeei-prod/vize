use super::compile_sfc_inner;
use crate::types::{SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

/// Script/template assembly selected by adapter-facing compiler entrypoints.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfcScriptOutputMode {
    /// Preserve the historical Rust API behavior and inline the render body.
    InlineTemplate,
    /// Emit a render function alongside the setup-state return object.
    SeparateTemplate,
}

impl SfcScriptOutputMode {
    pub(super) const fn separates_template(self) -> bool {
        matches!(self, Self::SeparateTemplate)
    }
}

/// Compile an SFC descriptor into JavaScript and CSS.
pub fn compile_sfc(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::InlineTemplate,
    )
}

/// Compile an SFC descriptor with Vue parser quirk compatibility.
#[deprecated(note = "use compile_sfc_with_template_syntax instead")]
pub fn compile_sfc_with_vue_parser_quirks(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        TemplateSyntaxMode::Quirks,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::InlineTemplate,
    )
}

/// Compile an SFC descriptor with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_sfc_with_template_syntax(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::InlineTemplate,
    )
}

/// Compile an SFC with adapter-provided codegen defaults.
#[doc(hidden)]
pub fn compile_sfc_with_template_syntax_and_codegen_options(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    codegen_options: CodegenOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_with_custom_elements_template_syntax_and_codegen_options(
        descriptor,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        codegen_options,
    )
}

/// Compile with declarative custom-element patterns and adapter codegen defaults.
#[doc(hidden)]
pub fn compile_sfc_with_custom_elements_template_syntax_and_codegen_options(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_for_adapter(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        SfcScriptOutputMode::InlineTemplate,
    )
}

/// Compile an SFC with an explicit adapter-facing script output mode.
#[doc(hidden)]
pub fn compile_sfc_for_adapter(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
    )
}
