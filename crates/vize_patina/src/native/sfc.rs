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
mod template;
#[cfg(test)]
mod tests;

pub use context::NativeSfcLintContext;
pub use owner::{NativeSfcLintOwner, NativeSfcSetup};
pub use template::{NativeSfcTemplateLintContext, NativeSfcTemplateOwner, NativeSfcTemplateRule};

/// Real original evidence; an error never returns pending product findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeSfcLintRefusal {
    /// The supplied source cannot establish a valid source frame.
    Source(SourceFrameError),
    /// The configured Vue version is outside the native SFC contract.
    UnsupportedVueVersion {
        /// Vue version requested by the linter configuration.
        requested: VueVersion,
    },
    /// The configured Vapor mode is outside the native SFC contract.
    UnsupportedVaporMode {
        /// Vapor mode requested by the linter configuration.
        requested: bool,
    },
    /// Type-aware linting is not offered by this native SFC capability.
    UnsupportedTypeAwareMode,
    /// An enabled configured rule does not offer a native SFC implementation.
    UnprovidedRule {
        /// Stable rule code that lacks the required capability.
        rule: String,
    },
    /// A rule offers native execution but not its configured invocation policy.
    UnprovidedInvocationPolicy {
        /// Stable rule code whose invocation policy cannot be preserved.
        rule: String,
    },
    /// Descriptor parsing produced issues or container errors.
    Descriptor {
        /// Recoverable descriptor issues retained from the original parse.
        issues: Vec<DescriptorIssue>,
        /// Descriptor container errors retained from the original parse.
        errors: Vec<ContainerError>,
    },
    /// The SFC contains blocks outside the admitted native envelope.
    UnsupportedEnvelope {
        /// Original unsupported block range.
        span: Span,
    },
    /// One retained owner or source range does not match the original SFC.
    SourceMismatch,
    /// The original selected template could not be parsed.
    Parse(ComponentSourceError),
    /// The original template failed native lint admission.
    Template(super::template::NativeTemplateLintRefusal),
    /// The setup block could not be projected into authored source.
    Embed(SourceError),
    /// The setup program contains unsupported or incomplete syntax.
    ProgramSyntax {
        /// Original setup block range.
        span: Span,
        /// Producer-recorded syntax hole, when available.
        hole: Option<EmbedHole>,
    },
    /// A setup comment requires suppression policy unavailable to this capability.
    ScriptComment {
        /// Original comment range.
        span: Span,
    },
    /// The setup program does not match the checked file producer input.
    Program(ProgramInputError),
    /// The script-only semantic file could not be produced.
    File(ArtifactError),
    /// The semantic file retained unresolved issues or interrupted programs.
    FileIssues {
        /// File issues retained by the producer.
        issues: Vec<FileIssue>,
        /// Issues attached to interrupted program production.
        interruptions: Vec<FileIssue>,
    },
    /// The original setup block cannot form a checked Vue setup view.
    Setup(SetupIssue),
}

/// Capability offered by the real configured ScriptRule instance. Its input
/// borrows the complete File, original Program and Descriptor of the same host;
/// primitive setup eligibility never certifies the pending template output.
pub trait NativeSfcSetupRule: Send + Sync {
    /// Runs the configured rule against the authenticated setup capability.
    fn run_on_setup<'a>(
        &self,
        context: &mut NativeSfcLintContext<'_, 'a>,
        setup: &NativeSfcSetup<'_, 'a>,
    ) -> Result<(), NativeSfcLintRefusal>;
}
