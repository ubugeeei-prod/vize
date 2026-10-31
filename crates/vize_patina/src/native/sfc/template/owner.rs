//! Retain the real Descriptor and its original selected content block together.

use super::super::{NativeSfcLintRefusal as Refusal, envelope};
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    markup::{NativeTemplateComponent, SurfaceParseOptions},
};

/// A normal original-SFC owner; callers cannot supply raw content spans or
/// replace it with a same-buffer bare template or setup owner.
///
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcTemplateOwner;
/// let owner = NativeSfcTemplateOwner {};
/// ```
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcTemplateOwner;
/// fn cloneable<T: Clone>() {}
/// cloneable::<NativeSfcTemplateOwner<'static>>();
/// ```
pub struct NativeSfcTemplateOwner<'a> {
    descriptor: DescriptorObservation<'a>,
    template: NativeTemplateComponent<'a>,
}

impl<'a> NativeSfcTemplateOwner<'a> {
    pub(super) fn parse_in(allocator: &'a Allocator, source: &'a str) -> Result<Self, Refusal> {
        SourceRoot::new(source).map_err(Refusal::Source)?;
        let descriptor = Vue.observe_descriptor(
            allocator,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        envelope::check_template_only(&descriptor)?;
        let admitted = descriptor.admitted().map_err(|_| Refusal::SourceMismatch)?;
        let original = admitted.template().ok_or(Refusal::SourceMismatch)?;
        let template = NativeTemplateComponent::parse_in(allocator, admitted)
            .map_err(Refusal::Parse)?
            .ok_or(Refusal::SourceMismatch)?;
        if original.container_index() != template.template_index()
            || original.block().span() != template.component().block().span()
            || !core::ptr::eq(
                original.block().source(),
                template.component().block().source(),
            )
            || !core::ptr::eq(source, template.component().block().root_source())
        {
            return Err(Refusal::SourceMismatch);
        }
        crate::native::admission::selected_component(&template).map_err(Refusal::Template)?;
        Ok(Self {
            descriptor,
            template,
        })
    }

    #[must_use]
    pub fn source(&self) -> &'a str {
        self.descriptor.source()
    }

    #[must_use]
    pub fn descriptor(&self) -> &DescriptorObservation<'a> {
        &self.descriptor
    }

    #[must_use]
    pub fn template(&self) -> &NativeTemplateComponent<'a> {
        &self.template
    }
}
