//! Qualified actual setup rows for consumers of the normally owned receipt.

use super::NativeSelectedSetup;
use crate::file::{
    BindingRef, DeclarationKind, Namespace,
    vue::{ExposureIssue, ExposureIssueKind as Kind},
};
use crate::lang::js::SetupAnnotation;
use vize_l0::SourceBlock;

impl<'owner, 'arena> NativeSelectedSetup<'owner, 'arena> {
    #[must_use]
    pub fn source(&self) -> SourceBlock<'arena> {
        self.selected().block()
    }

    /// The argument must be an actual same-File direct setup Value declaration.
    /// Equal local IDs, names or spans do not authorize a foreign row.
    pub fn binding(
        &self,
        binding: BindingRef<'owner, 'arena>,
    ) -> Result<BindingRef<'owner, 'arena>, ExposureIssue> {
        let reject = |kind| ExposureIssue {
            kind,
            span: self.source().span(),
        };
        if !core::ptr::eq(binding.file(), self.file()) {
            return Err(reject(Kind::ForeignBinding));
        }
        let declaration = binding
            .declaration()
            .ok_or_else(|| reject(Kind::DirectDeclaration))?;
        if declaration.unit != self.unit()
            || declaration.scope != self.scope()
            || !declaration.is_direct_program()
        {
            return Err(reject(Kind::DirectDeclaration));
        }
        if declaration.namespace != Namespace::Value {
            return Err(reject(Kind::Namespace));
        }
        if !matches!(
            declaration.kind,
            DeclarationKind::Const | DeclarationKind::Let | DeclarationKind::Var
        ) {
            return Err(reject(Kind::DeclarationKind));
        }
        if crate::file::vue::reserved(declaration.name.as_str()) {
            return Err(reject(Kind::ReservedName));
        }
        Ok(binding)
    }
    /// Original declaration order from this actual unit and root scope only.
    pub fn bindings(&self) -> impl Iterator<Item = BindingRef<'owner, 'arena>> + '_ {
        self.file()
            .bindings()
            .filter(|binding| self.binding(*binding).is_ok())
    }
    /// Original source order, using only annotation rows from the sole walk.
    pub fn type_annotations(&self) -> impl Iterator<Item = SetupAnnotation<'owner, 'arena>> + '_ {
        self.file().setup_annotations_for_unit(self.unit())
    }
}
