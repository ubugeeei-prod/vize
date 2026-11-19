//! Opaque public diagnostic facade over the private authoritative recorder.
use super::policy::Visibility;
use super::profile::NativeTemplateProfile;
use vize_l0::{Span, id::NodeId};
use vize_l2::artifact::{ArtifactError, ComponentBody, ComponentFactory};
use vize_l2::expr::JsExpr;
use vize_l2::file::TemplateRegion;
use vize_l2::op::{Attribute, Namespace};
use vize_l2::provenance::ProvenanceRecord;
mod native;
mod walk;

/// Its ordinary diagnostic factory grants no native construction receipt.
///
/// ```compile_fail
/// use vize_l1::embed::Lang;
/// use vize_l1_to_l2::{native::NativeComponent, vue_file::VueFileRegion};
/// fn override_lang<'a>(component: NativeComponent<'a>, region: &mut VueFileRegion<'_, 'a>) {
///     let _ = component.construct_vue_file_in(region, Lang::Ts);
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1_to_l2::vue_file::NativeTemplateProfile;
/// ```
pub struct VueFileRegion<'f, 'a> {
    inner: TemplateRegion<'f, 'a, Visibility>,
    profile: NativeTemplateProfile,
}
impl<'f, 'a> VueFileRegion<'f, 'a> {
    pub(super) fn new(
        inner: TemplateRegion<'f, 'a, Visibility>,
        profile: NativeTemplateProfile,
    ) -> Self {
        Self { inner, profile }
    }
    pub(crate) fn native_profile(&self) -> NativeTemplateProfile {
        self.profile
    }
}
impl<'a> ComponentFactory<'a> for VueFileRegion<'_, 'a> {
    fn source(&self) -> &'a str {
        self.inner.source()
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.inner.text(text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.inner.comment(text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.interpolation(expression, span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.bind(name, name_span, expression, span)
    }
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.element(tag, namespace, attributes, span, body)
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.component(name, attributes, span, body)
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.inner.record(record)
    }
}
