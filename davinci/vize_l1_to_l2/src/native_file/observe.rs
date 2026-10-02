use super::{NativeSfcIssue, NativeSfcIssueKind, NativeTemplateObservation};
use crate::vue_file::{RejectedVueFile, VueFile, VueScriptReceipt};
use alloc::vec::Vec;
use vize_l0::SourceBlock;
use vize_l1::container::vue::{DescriptorObservation, ScriptRole, ScriptView};
use vize_l1::embed::{Lang, syntax::NativeSyntax};
use vize_l2::file::ScriptUnitId;

/// Original selected role, original syntax and the actually minted unit, if any.
#[derive(Debug)]
pub struct NativeScriptObservation<'a> {
    pub(super) container_index: usize,
    pub(super) block: SourceBlock<'a>,
    pub(super) role: ScriptRole,
    pub(super) lang: Lang,
    pub(super) syntax: Option<NativeSyntax<'a>>,
    pub(super) unit: Option<ScriptUnitId>,
}
impl<'a> NativeScriptObservation<'a> {
    #[must_use]
    pub fn container_index(&self) -> usize {
        self.container_index
    }
    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    #[must_use]
    pub fn role(&self) -> ScriptRole {
        self.role
    }
    #[must_use]
    pub fn lang(&self) -> Lang {
        self.lang
    }
    #[must_use]
    pub fn syntax(&self) -> Option<&NativeSyntax<'a>> {
        self.syntax.as_ref()
    }
    #[must_use]
    pub fn unit(&self) -> Option<ScriptUnitId> {
        self.unit
    }
    fn matches(
        &self,
        selected: ScriptView<'_, 'a>,
        receipt: &VueScriptReceipt,
        file: &VueFile<'a>,
    ) -> bool {
        let Some(syntax) = &self.syntax else {
            return false;
        };
        let profile = receipt.profile();
        self.container_index == selected.container_index()
            && self.role == selected.role()
            && self.lang == selected.lang()
            && self.block.span() == selected.block().span()
            && core::ptr::eq(self.block.source(), selected.block().source())
            && core::ptr::eq(self.block.root_source(), selected.source())
            && syntax.admitted_program().is_some()
            && syntax.grammar().lang == self.lang
            && syntax.source_type().is_module()
            && !syntax.source_type().is_jsx()
            && syntax.source_type().is_typescript() == profile.typescript
            && syntax.source().span() == self.block.span()
            && core::ptr::eq(syntax.source().text(), self.block.source())
            && self.unit == Some(receipt.unit())
            && usize::try_from(receipt.unit().index()) == Ok(self.container_index)
            && receipt.span() == self.block.span()
            && profile.module
            && !profile.jsx
            && profile.typescript == (self.lang == Lang::Ts)
            && file.file().units().iter().any(|unit| {
                unit.id == receipt.unit()
                    && unit.scope == receipt.scope()
                    && unit.span == receipt.span()
                    && unit.profile == profile
            })
    }
}

/// Every original descriptor/script/template and partial File owner survives.
/// Fields are private; diagnostic factories cannot assemble this observation.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSfcObservation;
/// fn change(owner: &mut NativeSfcObservation<'_>) {
///     owner.scripts.clear();
/// }
/// ```
pub struct NativeSfcObservation<'a> {
    descriptor: DescriptorObservation<'a>,
    scripts: Vec<NativeScriptObservation<'a>>,
    template: Option<NativeTemplateObservation<'a>>,
    file: Option<Result<VueFile<'a>, RejectedVueFile<'a>>>,
    issues: Vec<NativeSfcIssue>,
    admitted: bool,
}
impl core::fmt::Debug for NativeSfcObservation<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeSfcObservation")
            .field("descriptor", &self.descriptor)
            .field("scripts", &self.scripts)
            .field("template", &self.template)
            .field("issues", &self.issues)
            .field("admitted", &self.admitted)
            .finish_non_exhaustive()
    }
}
impl<'a> NativeSfcObservation<'a> {
    pub(super) fn new(
        descriptor: DescriptorObservation<'a>,
        scripts: Vec<NativeScriptObservation<'a>>,
        template: Option<NativeTemplateObservation<'a>>,
        file: Option<Result<VueFile<'a>, RejectedVueFile<'a>>>,
        mut issues: Vec<NativeSfcIssue>,
    ) -> Self {
        let membership = match (descriptor.admitted(), file.as_ref()) {
            (Ok(view), Some(Ok(file))) => {
                let roles = [
                    (view.ordinary(), file.ordinary()),
                    (view.setup(), file.setup()),
                ];
                core::ptr::eq(file.file().artifact().source(), descriptor.source())
                    && file.file().is_complete()
                    && scripts.len() == roles.iter().filter(|(view, _)| view.is_some()).count()
                    && file.file().units().len() == scripts.len()
                    && roles
                        .into_iter()
                        .all(|(selected, receipt)| match (selected, receipt) {
                            (Some(selected), Some(receipt)) => scripts
                                .iter()
                                .any(|script| script.matches(selected, receipt, file)),
                            (None, None) => true,
                            _ => false,
                        })
                    && match (view.template(), template.as_ref()) {
                        (Some(selected), Some(template)) => {
                            template.matches(selected, view.template_lang())
                        }
                        (None, None) => true,
                        _ => false,
                    }
            }
            _ => false,
        };
        if !membership && issues.is_empty() {
            issues.push(NativeSfcIssue {
                container_index: None,
                span: descriptor.root().map(|root| root.whole_block().span()),
                kind: NativeSfcIssueKind::Membership,
            });
        }
        let admitted = membership && issues.is_empty();
        Self {
            descriptor,
            scripts,
            template,
            file,
            issues,
            admitted,
        }
    }
    #[must_use]
    pub fn descriptor(&self) -> &DescriptorObservation<'a> {
        &self.descriptor
    }
    #[must_use]
    pub fn scripts(&self) -> &[NativeScriptObservation<'a>] {
        &self.scripts
    }
    #[must_use]
    pub fn template(&self) -> Option<&NativeTemplateObservation<'a>> {
        self.template.as_ref()
    }
    #[must_use]
    pub fn file(&self) -> Option<&VueFile<'a>> {
        self.file.as_ref()?.as_ref().ok()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedVueFile<'a>> {
        self.file.as_ref()?.as_ref().err()
    }
    #[must_use]
    pub fn issues(&self) -> &[NativeSfcIssue] {
        &self.issues
    }
    /// Admission only for the documented first family, not an emitted product.
    #[must_use]
    pub fn admitted(&self) -> Option<NativeSfc<'_, 'a>> {
        if !self.admitted {
            return None;
        }
        Some(NativeSfc {
            observation: self,
            file: self.file()?,
        })
    }
}

/// Private-constructor view of this exact source-admitted assembly.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::{NativeSfc, NativeSfcObservation};
/// fn forge<'o, 'a>(observation: &'o NativeSfcObservation<'a>) {
///     if let Some(file) = observation.file() {
///         let _ = NativeSfc { observation, file };
///     }
/// }
/// ```
pub struct NativeSfc<'o, 'a> {
    observation: &'o NativeSfcObservation<'a>,
    file: &'o VueFile<'a>,
}
impl<'o, 'a> NativeSfc<'o, 'a> {
    #[must_use]
    pub fn observation(&self) -> &'o NativeSfcObservation<'a> {
        self.observation
    }
    #[must_use]
    pub fn file(&self) -> &'o VueFile<'a> {
        self.file
    }
}
