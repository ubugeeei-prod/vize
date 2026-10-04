//! Opt-in original-SFC custody for whole primitive-only TypeScript setup rules.
//! This grants no native template File, DOM/ref association or default route.
//! Offered builtin capabilities remain bounded to this whole setup proof.

use vize_l0::{SourceFrameError, Span, String, config::VueVersion};
use vize_l1::{
    container::{ContainerError, vue::DescriptorIssue},
    embed::{SourceError, syntax::EmbedHole},
    markup::ComponentSourceError,
};
use vize_l2::{
    artifact::ArtifactError,
    file::FileIssue,
    lang::js::{ProgramInputError, SetupIssue},
};

mod context;
mod driver;
mod envelope;
mod owner;
#[cfg(test)]
mod tests;

pub use context::NativeSfcLintContext;
pub use owner::{NativeSfcLintOwner, NativeSfcSetup};

/// Real original evidence; an error never returns pending product findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeSfcLintRefusal {
    Source(SourceFrameError),
    UnsupportedVueVersion {
        requested: VueVersion,
    },
    UnsupportedVaporMode {
        requested: bool,
    },
    UnsupportedTypeAwareMode,
    UnprovidedRule {
        rule: String,
    },
    UnprovidedInvocationPolicy {
        rule: String,
    },
    Descriptor {
        issues: Vec<DescriptorIssue>,
        errors: Vec<ContainerError>,
    },
    UnsupportedEnvelope {
        span: Span,
    },
    SourceMismatch,
    Parse(ComponentSourceError),
    Template(super::template::NativeTemplateLintRefusal),
    Embed(SourceError),
    ProgramSyntax {
        span: Span,
        hole: Option<EmbedHole>,
    },
    ScriptComment {
        span: Span,
    },
    Program(ProgramInputError),
    File(ArtifactError),
    FileIssues {
        issues: Vec<FileIssue>,
        interruptions: Vec<FileIssue>,
    },
    Setup(SetupIssue),
}

/// Capability offered by the real configured ScriptRule instance. Its input
/// borrows the complete File, original Program and Descriptor of the same host;
/// primitive setup eligibility never certifies the pending template output.
pub trait NativeSfcSetupRule: Send + Sync {
    fn run_on_setup<'a>(
        &self,
        context: &mut NativeSfcLintContext<'_, 'a>,
        setup: &NativeSfcSetup<'_, 'a>,
    ) -> Result<(), NativeSfcLintRefusal>;
}
