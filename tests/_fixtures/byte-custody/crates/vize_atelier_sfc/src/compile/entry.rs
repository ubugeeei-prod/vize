mod pug;
mod root_match;
use crate::types::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
};
use root_match::compile_sfc_inner;
pub use root_match::prepare_root_patterned_template;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_l0::dump::capture::{CaptureClock, StageCapture};

/// Compile against the immutable project selected by the host. Omitting a
/// project retains ordinary nearest-config discovery for filesystem inputs.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "adapter compile inputs and project authority"
)]
pub fn compile_sfc_for_adapter_with_project(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
    nuxt_page_meta: bool,
    project: Option<&crate::script::SfcTypeProject>,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        nuxt_page_meta,
        project,
        None,
    )
}

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
        SfcCompileExperimentalOptions::default(),
        false,
        None,
        None,
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
        SfcCompileExperimentalOptions::default(),
        false,
        None,
        None,
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
        SfcCompileExperimentalOptions::default(),
        false,
        None,
        None,
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

/// Compile with declarative custom-element patterns, adapter codegen defaults,
/// and opt-in experimental template codegen context.
#[doc(hidden)]
pub fn compile_sfc_with_custom_elements_template_syntax_codegen_and_experimental_options(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    experimental_options: SfcCompileExperimentalOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        SfcScriptOutputMode::InlineTemplate,
        experimental_options,
        false,
        None,
        None,
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
    compile_sfc_for_adapter_with_experimental_options(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        SfcCompileExperimentalOptions::default(),
    )
}

/// Compile an SFC with an explicit adapter-facing script output mode and
/// opt-in experimental template codegen context.
#[doc(hidden)]
pub fn compile_sfc_for_adapter_with_experimental_options(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        false,
        None,
        None,
    )
}

/// Compile once and return the stages observed by the backend that produced
/// the SFC module. The ordinary entry points keep their existing result shape.
#[doc(hidden)]
pub fn compile_sfc_for_adapter_with_stage_capture(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
) -> Result<(SfcCompileResult, StageCapture), SfcError> {
    compile_sfc_for_adapter_with_project_stage_capture_and_clock(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        None,
        None,
    )
}

/// Capture the adapter result using a monotonic clock supplied by its host.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "adapter inputs with host timing authority"
)]
pub fn compile_sfc_for_adapter_with_stage_capture_and_clock(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
    clock: CaptureClock,
) -> Result<(SfcCompileResult, StageCapture), SfcError> {
    compile_sfc_for_adapter_with_project_stage_capture_and_clock(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        None,
        Some(clock),
    )
}

/// Capture the same stages while compiling against a host-selected project.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "adapter inputs with project authority"
)]
pub fn compile_sfc_for_adapter_with_project_and_stage_capture(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
    project: Option<&crate::script::SfcTypeProject>,
) -> Result<(SfcCompileResult, StageCapture), SfcError> {
    compile_sfc_for_adapter_with_project_stage_capture_and_clock(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        project,
        None,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "one internal capture constructor for project and clock authority"
)]
fn compile_sfc_for_adapter_with_project_stage_capture_and_clock(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
    project: Option<&crate::script::SfcTypeProject>,
    clock: Option<CaptureClock>,
) -> Result<(SfcCompileResult, StageCapture), SfcError> {
    let mut capture = match clock {
        Some(clock) => StageCapture::new_with_clock("sfc", clock),
        None => StageCapture::new("sfc"),
    };
    let result = compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        false,
        project,
        Some(&mut capture),
    )?;
    Ok((result, capture))
}

/// Compile with Nuxt's page-meta extraction enabled by its integration.
/// Ordinary SFC entry points preserve these runtime calls and imports.
#[doc(hidden)]
pub fn compile_sfc_for_adapter_with_nuxt_page_meta(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
        true,
        None,
        None,
    )
}
