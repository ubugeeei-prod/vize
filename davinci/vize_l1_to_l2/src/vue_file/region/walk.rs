//! Private live-borrow bridge; normal end is not native construction authority.

use super::VueL2IrRegion;
use super::native::VueNative;
use crate::native::ConstructionFactory;
use crate::vue_file::{NativeTemplateProfile, NativeTemplateVisibility};
use vize_l0::Span;
use vize_l2::ir::{TemplateIssue, TemplateWalkRegion};
use vize_l2::template::L2TemplateError;

pub(crate) struct NativeVueWalk<'g, 'f, 'a> {
    lower: TemplateWalkRegion<'g, 'f, 'a, NativeTemplateVisibility>,
    profile: NativeTemplateProfile,
}

impl<'f, 'a> VueL2IrRegion<'f, 'a> {
    pub(crate) fn begin_native_walk(
        &mut self,
        span: Span,
    ) -> Result<NativeVueWalk<'_, 'f, 'a>, L2TemplateError> {
        let profile = self.native_profile();
        Ok(NativeVueWalk {
            lower: self.inner.walk(span)?,
            profile,
        })
    }
}
impl<'g, 'f, 'a> NativeVueWalk<'g, 'f, 'a> {
    pub(crate) fn profile(&self) -> NativeTemplateProfile {
        self.profile
    }
    pub(crate) fn lower(
        &mut self,
    ) -> &mut TemplateWalkRegion<'g, 'f, 'a, NativeTemplateVisibility> {
        &mut self.lower
    }
    pub(crate) fn factory(&mut self) -> impl ConstructionFactory<'a> + '_ {
        VueNative::new(self.lower())
    }
    pub(crate) fn complete(self) -> Result<(), TemplateIssue> {
        self.lower.complete()
    }
}
