//! Source-admitted script receipts select a private template factory policy.

use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Allocator, Span};
use vize_l2::artifact::L2TemplateError;
use vize_l2::file::{BindingRef, L2Ir, RejectedL2Ir, ScriptUnitId};
use vize_l2::lang::js::{L2IrProducer, ProgramInput, ProgramScope};

mod ordinary;
mod policy;
mod profile;
mod records;
pub use records::{
    VueDeclaration, VueL2IrIssue, VueL2IrIssueKind, VueScriptReceipt, VueScriptRole,
};
mod region;
pub(crate) use policy::Visibility as NativeTemplateVisibility;
use policy::{PolicyObserver, Visibility};
pub(crate) use profile::NativeTemplateProfile;
pub use region::VueL2IrRegion;

/// An opaque producer derives scope/exposure from its own actual unit receipts.
pub struct VueL2IrProducer<'a> {
    producer: L2IrProducer<'a>,
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    declarations: Vec<VueDeclaration>,
    issues: Vec<VueL2IrIssue>,
    template_started: bool,
    source: &'a str,
}

impl<'a> VueL2IrProducer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Result<Self, L2TemplateError> {
        Ok(Self {
            producer: L2IrProducer::new(allocator, source)?,
            ordinary: None,
            setup: None,
            declarations: Vec::new(),
            issues: Vec::new(),
            template_started: false,
            source,
        })
    }
    pub fn ordinary(&mut self, input: ProgramInput<'_, 'a>) -> Result<ScriptUnitId, VueL2IrIssue> {
        self.program(input, VueScriptRole::Ordinary)
    }
    pub fn setup(&mut self, input: ProgramInput<'_, 'a>) -> Result<ScriptUnitId, VueL2IrIssue> {
        self.program(input, VueScriptRole::Setup)
    }
    fn program(
        &mut self,
        input: ProgramInput<'_, 'a>,
        role: VueScriptRole,
    ) -> Result<ScriptUnitId, VueL2IrIssue> {
        let span = input.span();
        let duplicate = match role {
            VueScriptRole::Ordinary => self.ordinary.is_some(),
            VueScriptRole::Setup => self.setup.is_some(),
        };
        let misuse = if self.template_started {
            Some(VueL2IrIssueKind::ScriptAfterTemplate)
        } else if duplicate {
            Some(VueL2IrIssueKind::DuplicateRole)
        } else if role == VueScriptRole::Ordinary && self.setup.is_some() {
            Some(VueL2IrIssueKind::OrdinaryAfterSetup)
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
                return Err(self.reject(error.span, VueL2IrIssueKind::InvalidProgram(error.kind)));
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
    pub fn template_region(&mut self) -> Result<VueL2IrRegion<'_, 'a>, VueL2IrIssue> {
        let source = self.source;
        let span = Span::new(0, source.len() as u32);
        if self.template_started {
            return Err(self.reject(span, VueL2IrIssueKind::DuplicateTemplate));
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
        let staged = ordinary::staged_options(
            self.ordinary,
            self.setup,
            self.producer.ordinary_empty_script(),
        );
        if self.issues.iter().any(|issue| Some(*issue) != staged)
            || !self.producer.issues().is_empty()
            || self.producer.interrupted_programs().next().is_some()
        {
            return Err(self.reject(span, VueL2IrIssueKind::PreviousIssues));
        }
        self.template_started = true;
        let policy = Visibility::new(self.ordinary, self.setup);
        let selection = if self.setup.is_some() {
            vize_l2::file::TemplateScope::LastUnit
        } else {
            vize_l2::file::TemplateScope::Root
        };
        match self.producer.template_region(selection, policy) {
            Ok(region) => Ok(VueL2IrRegion::new(region, profile)),
            Err(kind) => {
                let issue = VueL2IrIssue {
                    span,
                    unit: None,
                    scope: None,
                    kind: VueL2IrIssueKind::InvalidProgram(kind),
                };
                self.issues.push(issue);
                Err(issue)
            }
        }
    }
    fn reject(&mut self, span: Span, kind: VueL2IrIssueKind) -> VueL2IrIssue {
        let issue = VueL2IrIssue {
            span,
            unit: None,
            scope: None,
            kind,
        };
        self.issues.push(issue);
        issue
    }
    pub fn finish(mut self) -> Result<VueL2Ir<'a>, RejectedVueL2Ir<'a>> {
        if let Err(issue) = NativeTemplateProfile::checked(self.ordinary, self.setup)
            && !self.issues.contains(&issue)
        {
            self.issues.push(issue);
        }
        let file = self.producer.finish();
        if let Ok(file) = &file
            && let Some(staged) =
                ordinary::staged_options(self.ordinary, self.setup, file.ordinary_empty_script())
            && let Some(index) = self.issues.iter().position(|issue| *issue == staged)
        {
            self.issues.remove(index);
        }
        match file {
            Ok(file) if file.is_complete() && self.issues.is_empty() => Ok(VueL2Ir {
                l2ir: file,
                ordinary: self.ordinary,
                setup: self.setup,
                declarations: self.declarations,
            }),
            file => Err(RejectedVueL2Ir {
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
pub struct VueL2Ir<'a> {
    l2ir: L2Ir<'a>,
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    declarations: Vec<VueDeclaration>,
}
impl core::fmt::Debug for VueL2Ir<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("VueL2Ir")
            .field("ordinary", &self.ordinary)
            .field("setup", &self.setup)
            .finish_non_exhaustive()
    }
}
impl<'a> VueL2Ir<'a> {
    #[must_use]
    pub fn l2ir(&self) -> &L2Ir<'a> {
        &self.l2ir
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
        if !core::ptr::eq(binding.l2ir(), &self.l2ir)
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
    file: &'f VueL2Ir<'a>,
    binding: BindingRef<'f, 'a>,
    declaration: &'f VueDeclaration,
}
impl<'f, 'a> VueExposure<'f, 'a> {
    #[must_use]
    pub fn file(&self) -> &'f VueL2Ir<'a> {
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
pub struct RejectedVueL2Ir<'a> {
    file: Box<Result<L2Ir<'a>, RejectedL2Ir<'a>>>,
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    declarations: Vec<VueDeclaration>,
    issues: Vec<VueL2IrIssue>,
}
impl core::fmt::Debug for RejectedVueL2Ir<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RejectedVueL2Ir")
            .field("ordinary", &self.ordinary)
            .field("setup", &self.setup)
            .field("issues", &self.issues)
            .finish_non_exhaustive()
    }
}
impl<'a> RejectedVueL2Ir<'a> {
    #[must_use]
    pub fn file(&self) -> Option<&L2Ir<'a>> {
        self.file.as_ref().as_ref().ok()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedL2Ir<'a>> {
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
    pub fn issues(&self) -> &[VueL2IrIssue] {
        &self.issues
    }
}

#[cfg(feature = "lexical-arrow-initializers")]
impl<'a> VueL2IrProducer<'a> {
    pub(crate) fn for_lexical_initializers(
        allocator: &'a Allocator,
        source: &'a str,
    ) -> Result<Self, L2TemplateError> {
        let mut producer = Self::new(allocator, source)?;
        producer.producer.enable_lexical_arrow_initializers();
        Ok(producer)
    }
}

#[cfg(feature = "lexical-arrow-initializers")]
impl VueL2IrProducer<'_> {
    pub(crate) fn reserve_template_storage(&mut self, roots: usize, expressions: usize) {
        self.producer.reserve_template_storage(roots, expressions);
    }
}
