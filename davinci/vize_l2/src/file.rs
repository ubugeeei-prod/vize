//! File-local semantic facts sealed with their actual canonical artifact.
//!
//! Bare indices identify rows only inside this owner. Construction belongs to
//! checked language producers, never a caller-supplied declaration/name list.

use crate::artifact::L2Template;
use crate::resolution::{BindingId, ResolutionTable};
use alloc::boxed::Box;
use vize_l0::id::NodeId;

pub(crate) mod build;
pub(crate) mod region;
pub use build::L2IrBuilder;
pub use region::{
    TemplateBody, TemplateChildRegion, TemplatePolicy, TemplateRegion, TemplateScope,
    TemplateWalkRegion,
};
mod attribute_value;
mod conditional;
mod model;
pub use attribute_value::{
    L2IrAttributeValue, NativeL2IrAttributeValue, NativeL2IrAttributeValueState,
};
pub use conditional::{
    NativeConditionalElseInput, NativeL2IrConditional, NativeL2IrConditionalState,
};
pub use model::{NativeL2IrModel, NativeL2IrModelKind, NativeL2IrModelState};
mod for_head;
mod handler;
mod interpolation;
mod text_value;
pub use text_value::{
    L2IrTextValue, NativeL2IrTextValue, NativeL2IrTextValueState, NativeTextValuePolicyError,
};
mod module_source;
pub use for_head::{L2IrForHead, L2IrTemplateDeclaration, RejectedL2IrFor, TemplateDeclaration};
pub use handler::{L2IrHandler, RejectedL2IrHandler};
pub use interpolation::{
    NativeL2IrInterpolation, NativeL2IrInterpolationFailure, NativeL2IrInterpolationState,
};
pub(crate) use module_source::{DYNAMIC_IMPORT, EMPTY_SOURCE_EXPORT, SourceSite};
mod records;
mod template;
pub(crate) use records::ProgramOrigin;
pub mod vue;
pub use records::{
    Declaration, DeclarationKind, Export, Import, InitializerKind, L2IrIssue, L2IrIssueKind,
    Namespace, Reference, ReferenceTarget, Scope, ScopeId, ScriptProfile, ScriptUnit, ScriptUnitId,
    TemplateIssue,
};

/// Immutable file facts and the exact canonical tree built with them.
///
/// Arbitrary declarations cannot establish file ownership:
/// ```compile_fail,E0451
/// use vize_l2::{artifact::L2Template, file::L2Ir};
/// fn forge<'a>(artifact: L2Template<'a>, actual: L2Ir<'a>) {
///     let _ = L2Ir { artifact, ..actual };
/// }
/// ```
pub struct L2Ir<'a> {
    artifact: L2Template<'a>,
    /// Read by `crate::query` scans; construction stays with the producers.
    pub(crate) facts: build::L2Script<'a>,
}

impl<'a> L2Ir<'a> {
    #[must_use]
    pub fn artifact(&self) -> &L2Template<'a> {
        &self.artifact
    }

    #[must_use]
    pub fn units(&self) -> &[ScriptUnit] {
        &self.facts.units
    }

    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.facts.scopes
    }

    #[must_use]
    pub fn binding(&self, id: BindingId) -> Option<BindingRef<'_, 'a>> {
        if self.facts.declarations.get(id.index() as usize).is_none()
            && self.facts.template_declaration(id).is_none()
        {
            return None;
        }
        Some(BindingRef { l2ir: self, id })
    }

    /// Script rows followed by authentic original-template rows in mint order.
    pub fn bindings(&self) -> impl Iterator<Item = BindingRef<'_, 'a>> {
        self.facts
            .declarations
            .iter()
            .map(|declaration| BindingRef {
                l2ir: self,
                id: declaration.id,
            })
            .chain(self.template_declarations().map(|declaration| BindingRef {
                l2ir: self,
                id: declaration.declaration().id(),
            }))
    }

    #[must_use]
    pub fn references(&self) -> &[Reference] {
        &self.facts.references
    }

    #[must_use]
    pub fn exports(&self) -> &[Export] {
        &self.facts.exports
    }

    #[must_use]
    pub fn imports(&self) -> &[Import] {
        &self.facts.imports
    }

    /// Same completed File's original bounded ordinary script family.
    #[must_use]
    pub fn ordinary_empty_script(&self) -> Option<crate::lang::js::OrdinaryEmptyScript<'_>> {
        if !self.is_complete() {
            return None;
        }
        crate::lang::js::file::ordinary::OrdinaryEmptyScript::from_facts(&self.facts)
    }

    pub(crate) fn setup_annotations(
        &self,
    ) -> &[crate::lang::js::file::setup::SetupAnnotationRecord] {
        self.facts
            .setup_annotations
            .as_deref()
            .map_or(&[], |storage| storage.rows())
    }

    /// Borrowed name lookup inside an actual file scope; no context fallback.
    #[must_use]
    pub fn lookup(
        &self,
        scope: ScopeId,
        name: &str,
        namespace: Namespace,
    ) -> Option<BindingRef<'_, 'a>> {
        self.binding(self.facts.lookup(scope, name, namespace)?)
    }

    #[must_use]
    pub fn issues(&self) -> &[L2IrIssue] {
        &self.facts.issues
    }

    pub fn interrupted_programs(&self) -> impl Iterator<Item = L2IrIssue> + '_ {
        self.facts.units.iter().filter_map(ScriptUnit::interruption)
    }

    /// Interruption of the actual surrounding template walk, separate from syntax issues.
    #[must_use]
    pub fn template_interruption(&self) -> Option<TemplateIssue> {
        self.facts.template_walk.interruption()
    }

    /// Completeness of the admitted script family, not of Vue/product output.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.facts.issues.is_empty()
            && self.facts.template_issues.is_empty()
            && self.facts.template_walk.is_complete()
            && self.facts.units.iter().all(ScriptUnit::walk_completed)
    }

    #[must_use]
    pub fn template_issues(&self) -> &[TemplateIssue] {
        &self.facts.template_issues
    }

    #[must_use]
    pub fn expression(&self, node: NodeId) -> Option<L2IrResolution<'_, 'a>> {
        self.facts.expressions.get(node)?;
        Some(L2IrResolution { l2ir: self, node })
    }

    pub(crate) fn from_producer(artifact: L2Template<'a>, facts: build::L2Script<'a>) -> Self {
        Self { artifact, facts }
    }
}

/// A query handle retains the actual file; matching numeric IDs are insufficient.
///
/// ```compile_fail
/// use vize_l2::{file::{BindingRef, L2Ir}, resolution::BindingId};
/// fn forge(l2ir: &L2Ir<'_>) {
///     let _ = BindingRef { l2ir, id: BindingId::new(0) };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct BindingRef<'f, 'a> {
    l2ir: &'f L2Ir<'a>,
    id: BindingId,
}

impl<'f, 'a> BindingRef<'f, 'a> {
    #[must_use]
    pub fn l2ir(self) -> &'f L2Ir<'a> {
        self.l2ir
    }

    #[must_use]
    pub fn id(self) -> BindingId {
        self.id
    }

    /// Original script declaration only; template aliases retain their whole
    /// original parameter through `template_declaration` instead.
    #[must_use]
    pub fn declaration(self) -> Option<&'f Declaration> {
        self.l2ir.facts.declarations.get(self.id.index() as usize)
    }

    #[must_use]
    pub fn same_owner(self, other: BindingRef<'_, '_>) -> bool {
        core::ptr::eq(self.l2ir, other.l2ir)
    }
}

/// A complete expression entry remains associated with its actual factory node.
///
/// ```compile_fail
/// use vize_l0::id::NodeId;
/// use vize_l2::file::{L2Ir, L2IrResolution};
/// fn forge(l2ir: &L2Ir<'_>) {
///     let _ = L2IrResolution { l2ir, node: NodeId::FIRST };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct L2IrResolution<'f, 'a> {
    l2ir: &'f L2Ir<'a>,
    node: NodeId,
}

impl<'f, 'a> L2IrResolution<'f, 'a> {
    #[must_use]
    pub fn l2ir(self) -> &'f L2Ir<'a> {
        self.l2ir
    }

    #[must_use]
    pub fn node(self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn table(self) -> Option<&'f ResolutionTable<'a>> {
        Some(&self.l2ir.facts.expressions.get(self.node)?.table)
    }

    /// The lexical scope captured by the actual expression factory.
    #[must_use]
    pub fn scope(self) -> Option<ScopeId> {
        Some(self.l2ir.facts.expressions.get(self.node)?.scope)
    }

    #[must_use]
    pub fn binding(self, id: BindingId) -> Option<BindingRef<'f, 'a>> {
        self.l2ir.binding(id)
    }

    #[must_use]
    pub fn accepts(self, binding: BindingRef<'_, '_>) -> bool {
        core::ptr::eq(self.l2ir, binding.l2ir)
    }
}

/// A rejected canonical construction retains its private semantic facts.
pub struct RejectedL2Ir<'a> {
    pub(crate) artifact: crate::artifact::RejectedL2Template<'a>,
    pub(crate) facts: Box<build::L2Script<'a>>,
}

impl core::fmt::Debug for RejectedL2Ir<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RejectedL2Ir")
            .field("artifact", &self.artifact)
            .field("issues", &self.facts.issues)
            .field("template_issues", &self.facts.template_issues)
            .finish_non_exhaustive()
    }
}

impl<'a> RejectedL2Ir<'a> {
    #[must_use]
    pub fn template_interruption(&self) -> Option<TemplateIssue> {
        self.facts.template_walk.interruption()
    }
    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.facts.scopes
    }
    #[must_use]
    pub fn artifact(&self) -> &crate::artifact::RejectedL2Template<'a> {
        &self.artifact
    }
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.artifact.parts.source
    }
    #[must_use]
    pub fn units(&self) -> &[ScriptUnit] {
        &self.facts.units
    }
    #[must_use]
    pub fn declarations(&self) -> &[Declaration] {
        &self.facts.declarations
    }
    #[must_use]
    pub fn issues(&self) -> &[L2IrIssue] {
        &self.facts.issues
    }
    pub fn interrupted_programs(&self) -> impl Iterator<Item = L2IrIssue> + '_ {
        self.facts.units.iter().filter_map(ScriptUnit::interruption)
    }
    #[must_use]
    pub fn template_issues(&self) -> &[TemplateIssue] {
        &self.facts.template_issues
    }
}

#[cfg(feature = "ir-views")]
impl<'a> crate::L2Ir<'a> {
    /// The original input buffer that gives this file's spans their coordinates.
    #[inline]
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.artifact.source()
    }

    /// The canonical component region borrowed by L2 analyses and L3 lowering.
    #[inline]
    #[must_use]
    pub fn root_region(&self) -> &crate::op::Region<'a> {
        self.artifact.root()
    }
}
