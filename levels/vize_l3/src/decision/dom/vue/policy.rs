//! Private policies specialize the existing complete occurrence check.

use super::VueReadKind;
use vize_l2::{
    file::{BindingRef, DeclarationKind, vue::VueExposure},
    resolution::{Occurrence, Usage},
};

pub(in crate::decision) trait FileReads<'owner, 'arena> {
    const RECORD: bool;
    const NATIVE_SETUP: bool = false;
    const ORIGINAL_FOR: bool = false;
    fn classify(
        &self,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind>;
}

pub(in crate::decision) struct NoReads;

impl<'owner, 'arena> FileReads<'owner, 'arena> for NoReads {
    const RECORD: bool = false;
    fn classify(
        &self,
        _: &Occurrence<'arena>,
        _: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        None
    }
}

pub(super) struct ExposureReads<'view, 'owner, 'descriptor, 'program, 'arena>(
    pub(super) &'view VueExposure<'owner, 'descriptor, 'program, 'arena>,
);

impl<'owner, 'arena> FileReads<'owner, 'arena> for ExposureReads<'_, 'owner, '_, '_, 'arena> {
    const RECORD: bool = true;
    fn classify(
        &self,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        if occurrence.usage != Usage::Read || self.0.binding(binding).is_err() {
            return None;
        }
        let declaration = binding.declaration()?;
        match declaration.kind {
            DeclarationKind::Const if declaration.initializer.is_primitive() => {
                Some(VueReadKind::SetupConst)
            }
            DeclarationKind::Let | DeclarationKind::Var => Some(VueReadKind::SetupLet),
            _ => None,
        }
    }
}
