use vize_atelier_core::options::{
    CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum S2EmitSelection {
    Allowed,
    RequireSections,
    /// The entry point declines S2 (compatibility entries, differential old side).
    Disabled,
    /// S2 was already attempted for this compile and refused it.
    Refused,
}

pub(super) struct DomCompilePipelineOptions {
    pub(super) custom_elements: CustomElementMatcher,
    pub(super) codegen_options: CodegenOptions,
    pub(super) codegen_experimental_options: CodegenExperimentalOptions,
    pub(super) s2_emit_selection: S2EmitSelection,
}

impl DomCompilePipelineOptions {
    pub(super) fn allow_s2(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options: CodegenExperimentalOptions::default(),
            s2_emit_selection: S2EmitSelection::Allowed,
        }
    }

    pub(super) fn allow_s2_with_experimental_options(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
        codegen_experimental_options: CodegenExperimentalOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options,
            s2_emit_selection: S2EmitSelection::Allowed,
        }
    }

    pub(super) fn require_sections(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options: CodegenExperimentalOptions::default(),
            s2_emit_selection: S2EmitSelection::RequireSections,
        }
    }

    pub(super) fn require_sections_with_experimental_options(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
        codegen_experimental_options: CodegenExperimentalOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options,
            s2_emit_selection: S2EmitSelection::RequireSections,
        }
    }

    /// The legacy lane, S2 declined — the differential lanes' old side.
    #[cfg(feature = "davinci-differential")]
    pub(super) fn deny_s2(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options: CodegenExperimentalOptions::default(),
            s2_emit_selection: S2EmitSelection::Disabled,
        }
    }

    /// The legacy lane after the SFC fast path's S2 attempt refused the
    /// template: sections still come from compatibility codegen, and the
    /// selection accounting names the refusal rather than the entry.
    pub(super) fn after_refusal_with_experimental_options(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
        codegen_experimental_options: CodegenExperimentalOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options,
            s2_emit_selection: S2EmitSelection::Refused,
        }
    }

    pub(super) fn require_sections_compat_with_experimental_options(
        custom_elements: CustomElementMatcher,
        codegen_options: CodegenOptions,
        codegen_experimental_options: CodegenExperimentalOptions,
    ) -> Self {
        Self {
            custom_elements,
            codegen_options,
            codegen_experimental_options,
            s2_emit_selection: S2EmitSelection::Disabled,
        }
    }
}
