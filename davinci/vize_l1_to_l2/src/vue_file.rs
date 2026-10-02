//! Source-admitted script receipts select a private template factory policy.

use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Allocator, Span};
use vize_l2::artifact::ArtifactError;
use vize_l2::file::{BindingRef, FileArtifact, RejectedFile, ScriptUnitId};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

mod policy;
mod profile;
mod records;
pub use records::{
    VueDeclaration, VueFileIssue, VueFileIssueKind, VueScriptReceipt, VueScriptRole,
};
mod region;
pub(crate) use policy::Visibility as NativeTemplateVisibility;
use policy::{PolicyObserver, Visibility};
pub(crate) use profile::NativeTemplateProfile;
pub use region::VueFileRegion;

/// An opaque producer derives scope/exposure from its own actual unit receipts.
pub struct VueFileProducer<'a> {
    producer: FileProducer<'a>,
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    declarations: Vec<VueDeclaration>,
    issues: Vec<VueFileIssue>,
    template_started: bool,
    source: &'a str,
}

impl<'a> VueFileProducer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Result<Self, ArtifactError> {
        Ok(Self {
            producer: FileProducer::new(allocator, source)?,
            ordinary: None,
            setup: None,
            declarations: Vec::new(),
            issues: Vec::new(),
            template_started: false,
            source,
        })
    }
    pub fn ordinary(&mut self, input: ProgramInput<'_, 'a>) -> Result<ScriptUnitId, VueFileIssue> {
        self.program(input, VueScriptRole::Ordinary)
    }
    pub fn setup(&mut self, input: ProgramInput<'_, 'a>) -> Result<ScriptUnitId, VueFileIssue> {
        self.program(input, VueScriptRole::Setup)
    }
    fn program(
        &mut self,
        input: ProgramInput<'_, 'a>,
        role: VueScriptRole,
    ) -> Result<ScriptUnitId, VueFileIssue> {
        let span = input.span();
        let duplicate = match role {
            VueScriptRole::Ordinary => self.ordinary.is_some(),
            VueScriptRole::Setup => self.setup.is_some(),
        };
        let misuse = if self.template_started {
            Some(VueFileIssueKind::ScriptAfterTemplate)
        } else if duplicate {
            Some(VueFileIssueKind::DuplicateRole)
        } else if role == VueScriptRole::Ordinary && self.setup.is_some() {
            Some(VueFileIssueKind::OrdinaryAfterSetup)
        } else {
            None
        };
        if let Some(kind) = misuse {
            return Err(self.reject(span, kind));
        }
        let mut observer = PolicyObserver {
            role,
            source_type: input.source_type(),
            receipt: None,
            declarations: &mut self.declarations,
            issues: &mut self.issues,
        };
        let (unit, _) = match self.producer.program_observed(
            input,
            if role == VueScriptRole::Setup {
                ProgramScope::Nested
            } else {
                ProgramScope::Module
            },
            &mut observer,
        ) {
            Ok(value) => value,
            Err(error) => {
                return Err(self.reject(error.span, VueFileIssueKind::InvalidProgram(error.kind)));
            }
        };
        let receipt = observer.receipt;
        match role {
            VueScriptRole::Ordinary => self.ordinary = receipt,
            VueScriptRole::Setup => self.setup = receipt,
        }
        Ok(unit)
    }
    /// No caller scope, unit, lookup or name map can confer file completeness.
    pub fn template_region(&mut self) -> Result<VueFileRegion<'_, 'a>, VueFileIssue> {
        let source = self.source;
        let span = Span::new(0, source.len() as u32);
        if self.template_started {
            return Err(self.reject(span, VueFileIssueKind::DuplicateTemplate));
        }
        let profile = match NativeTemplateProfile::checked(self.ordinary, self.setup) {
            Ok(profile) => profile,
            Err(issue) => {
                if !self.issues.contains(&issue) {
                    self.issues.push(issue);
                }
                return Err(issue);
            }
        };
        if !self.issues.is_empty()
            || !self.producer.issues().is_empty()
            || self.producer.interrupted_programs().next().is_some()
        {
            return Err(self.reject(span, VueFileIssueKind::PreviousIssues));
        }
        self.template_started = true;
        let policy = Visibility::new(self.ordinary, self.setup);
        let selection = if self.setup.is_some() {
            vize_l2::file::TemplateScope::LastUnit
        } else {
            vize_l2::file::TemplateScope::Root
        };
        match self.producer.template_region(selection, policy) {
            Ok(region) => Ok(VueFileRegion::new(region, profile)),
            Err(kind) => {
                let issue = VueFileIssue {
                    span,
                    unit: None,
                    scope: None,
                    kind: VueFileIssueKind::InvalidProgram(kind),
                };
                self.issues.push(issue);
                Err(issue)
            }
        }
    }
    fn reject(&mut self, span: Span, kind: VueFileIssueKind) -> VueFileIssue {
        let issue = VueFileIssue {
            span,
            unit: None,
            scope: None,
            kind,
        };
        self.issues.push(issue);
        issue
    }
    pub fn finish(mut self) -> Result<VueFile<'a>, RejectedVueFile<'a>> {
        if let Err(issue) = NativeTemplateProfile::checked(self.ordinary, self.setup)
            && !self.issues.contains(&issue)
        {
            self.issues.push(issue);
        }
        let file = self.producer.finish();
        match file {
            Ok(file) if file.is_complete() && self.issues.is_empty() => Ok(VueFile {
                file,
                ordinary: self.ordinary,
                setup: self.setup,
                declarations: self.declarations,
            }),
            file => Err(RejectedVueFile {
                file: Box::new(file),
                ordinary: self.ordinary,
                setup: self.setup,
                declarations: self.declarations,
                issues: self.issues,
            }),
        }
    }
}

/// Completeness of the admitted Vue file family; descriptor membership remains
/// the native SFC assembler's checked obligation.
pub struct VueFile<'a> {
    file: FileArtifact<'a>,
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    declarations: Vec<VueDeclaration>,
}
impl core::fmt::Debug for VueFile<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("VueFile")
            .field("ordinary", &self.ordinary)
            .field("setup", &self.setup)
            .finish_non_exhaustive()
    }
}
impl<'a> VueFile<'a> {
    #[must_use]
    pub fn file(&self) -> &FileArtifact<'a> {
        &self.file
    }
    #[must_use]
    pub fn ordinary(&self) -> Option<&VueScriptReceipt> {
        self.ordinary.as_ref()
    }
    #[must_use]
    pub fn setup(&self) -> Option<&VueScriptReceipt> {
        self.setup.as_ref()
    }
    #[must_use]
    pub fn declarations(&self) -> &[VueDeclaration] {
        &self.declarations
    }
    #[must_use]
    pub fn exposure<'f>(&'f self, binding: BindingRef<'f, 'a>) -> Option<VueExposure<'f, 'a>> {
        if !core::ptr::eq(binding.file(), &self.file)
            || !Visibility::new(self.ordinary, self.setup).accepts(binding.declaration()?)
        {
            return None;
        }
        let declaration = self
            .declarations
            .iter()
            .find(|declaration| declaration.binding == binding.id())?;
        Some(VueExposure {
            file: self,
            binding,
            declaration,
        })
    }
}

pub struct VueExposure<'f, 'a> {
    file: &'f VueFile<'a>,
    binding: BindingRef<'f, 'a>,
    declaration: &'f VueDeclaration,
}
impl<'f, 'a> VueExposure<'f, 'a> {
    #[must_use]
    pub fn file(&self) -> &'f VueFile<'a> {
        self.file
    }
    #[must_use]
    pub fn binding(&self) -> BindingRef<'f, 'a> {
        self.binding
    }
    #[must_use]
    pub fn role(&self) -> VueScriptRole {
        self.declaration.role
    }
}

/// Both canonical and semantic partial owners survive failure.
pub struct RejectedVueFile<'a> {
    file: Box<Result<FileArtifact<'a>, RejectedFile<'a>>>,
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    declarations: Vec<VueDeclaration>,
    issues: Vec<VueFileIssue>,
}
impl core::fmt::Debug for RejectedVueFile<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RejectedVueFile")
            .field("ordinary", &self.ordinary)
            .field("setup", &self.setup)
            .field("issues", &self.issues)
            .finish_non_exhaustive()
    }
}
impl<'a> RejectedVueFile<'a> {
    #[must_use]
    pub fn file(&self) -> Option<&FileArtifact<'a>> {
        self.file.as_ref().as_ref().ok()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedFile<'a>> {
        self.file.as_ref().as_ref().err()
    }
    #[must_use]
    pub fn ordinary(&self) -> Option<&VueScriptReceipt> {
        self.ordinary.as_ref()
    }
    #[must_use]
    pub fn setup(&self) -> Option<&VueScriptReceipt> {
        self.setup.as_ref()
    }
    #[must_use]
    pub fn declarations(&self) -> &[VueDeclaration] {
        &self.declarations
    }
    #[must_use]
    pub fn issues(&self) -> &[VueFileIssue] {
        &self.issues
    }
}
