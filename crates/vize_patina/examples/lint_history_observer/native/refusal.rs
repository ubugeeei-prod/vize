//! Retain the complete actual driver refusal without an invented Debug wrapper.

use vize_patina::native::{sfc::NativeSfcLintRefusal, template::NativeTemplateLintRefusal};

pub(super) enum QueryRefusal {
    Template(NativeTemplateLintRefusal),
    Sfc(NativeSfcLintRefusal),
}

impl QueryRefusal {
    pub(super) fn detail(&self) -> std::string::String {
        match self {
            Self::Template(error) => format!("{error:?}"),
            Self::Sfc(error) => format!("{error:?}"),
        }
    }

    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::Template(error) => super::kind(error),
            Self::Sfc(error) => match error {
                NativeSfcLintRefusal::Source(_) => "Source",
                NativeSfcLintRefusal::UnsupportedVueVersion { .. } => "UnsupportedVueVersion",
                NativeSfcLintRefusal::UnsupportedVaporMode { .. } => "UnsupportedVaporMode",
                NativeSfcLintRefusal::UnsupportedTypeAwareMode => "UnsupportedTypeAwareMode",
                NativeSfcLintRefusal::UnprovidedRule { .. } => "UnprovidedRule",
                NativeSfcLintRefusal::UnprovidedInvocationPolicy { .. } => {
                    "UnprovidedInvocationPolicy"
                }
                NativeSfcLintRefusal::Descriptor { .. } => "Descriptor",
                NativeSfcLintRefusal::UnsupportedEnvelope { .. } => "UnsupportedEnvelope",
                NativeSfcLintRefusal::SourceMismatch => "SourceMismatch",
                NativeSfcLintRefusal::Parse(_) => "Parse",
                NativeSfcLintRefusal::Template(_) => "Template",
                NativeSfcLintRefusal::Embed(_) => "Embed",
                NativeSfcLintRefusal::ProgramSyntax { .. } => "ProgramSyntax",
                NativeSfcLintRefusal::ScriptComment { .. } => "ScriptComment",
                NativeSfcLintRefusal::Program(_) => "Program",
                NativeSfcLintRefusal::File(_) => "File",
                NativeSfcLintRefusal::FileIssues { .. } => "FileIssues",
                NativeSfcLintRefusal::Setup(_) => "Setup",
            },
        }
    }
}
