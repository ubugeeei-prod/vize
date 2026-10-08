//! Distinct original Vue 1 descriptor custody, without modern admission.

use super::{DescriptorIssue, DescriptorOptions, TemplateSelection};
use crate::container::{Container, ContainerError};
use crate::dialect::vue1::surface::ComponentParse;
use vize_l0::{SourceRoot, Vec};

mod observe;
mod view;
pub use view::Vue1TemplateView;
#[cfg(test)]
pub(crate) mod hooks;
#[cfg(test)]
mod tests;

/// One original splitter and, only for a supported envelope, one Component.
/// Body diagnostics and TextView admission stay distinct from this envelope.
/// Fields are private; captures and caller syntax cannot construct this owner.
///
/// ```compile_fail
/// use vize_l1::container::vue::Vue1DescriptorObservation;
/// fn clone(owner: Vue1DescriptorObservation<'_>) { let _ = owner.clone(); }
/// ```
/// Every constructor field is supplied with its real type; privacy rejects it:
/// ```compile_fail
/// use vize_l0::{Allocator, SourceRoot, Vec, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::{ContainerFormat, Vue,
///     vue::{DescriptorOptions, Vue1DescriptorObservation}}};
/// let arena = Allocator::default();
/// let source = "<template>{{ value }}</template>";
/// let _ = Vue1DescriptorObservation {
///     container: Vue.split(&arena, source), root: SourceRoot::new(source).ok(),
///     options: DescriptorOptions { version: VueVersion::V1,
///         dialect: VueDialect::Vue, template: SurfaceParseOptions::default() },
///     issues: Vec::new_in(&arena), selection: None, component: None,
/// };
/// ```
/// A capture is not an original historical owner:
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l1::container::{ContainerFormat, Vue, vue::Vue1DescriptorObservation};
/// let arena = Allocator::default();
/// let owner: Vue1DescriptorObservation<'_> = Vue.split(&arena, "<template/>").into();
/// ```
/// Neither the source nor allocator can be released while this owner lives:
/// ```compile_fail
/// use vize_l0::{Allocator, String, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}};
/// let arena = Allocator::default();
/// let owner = {
///     let source = String::from("<template>{{ value }}</template>");
///     Vue.observe_vue1_descriptor(&arena, &source, DescriptorOptions {
///         version: VueVersion::V1, dialect: VueDialect::Vue,
///         template: SurfaceParseOptions::default(),
///     })
/// };
/// let _ = owner.selected();
/// ```
/// ```compile_fail
/// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}};
/// let owner = {
///     let arena = Allocator::default();
///     Vue.observe_vue1_descriptor(&arena, "<template/>" , DescriptorOptions {
///         version: VueVersion::V1, dialect: VueDialect::Vue,
///         template: SurfaceParseOptions::default(),
///     })
/// };
/// let _ = owner.container();
/// ```
#[derive(Debug)]
pub struct Vue1DescriptorObservation<'a> {
    container: Container<'a>,
    root: Option<SourceRoot<'a>>,
    options: DescriptorOptions,
    issues: Vec<'a, DescriptorIssue>,
    selection: Option<TemplateSelection<'a>>,
    component: Option<ComponentParse<'a>>,
}

impl<'a> Vue1DescriptorObservation<'a> {
    pub fn source(&self) -> &'a str {
        self.container.source
    }
    pub fn root(&self) -> Option<SourceRoot<'a>> {
        self.root
    }
    pub fn options(&self) -> DescriptorOptions {
        self.options
    }
    pub fn container(&self) -> &Container<'a> {
        &self.container
    }
    pub fn issues(&self) -> &[DescriptorIssue] {
        &self.issues
    }
    pub fn component(&self) -> Option<&ComponentParse<'a>> {
        self.component.as_ref()
    }

    /// Borrow this envelope and its actual once-created historical Component.
    /// A selected envelope can still contain original body errors or holes.
    pub fn selected(&self) -> Result<Vue1TemplateView<'_, 'a>, Vue1DescriptorRefusal<'_>> {
        view::selected(self).ok_or(Vue1DescriptorRefusal {
            issues: &self.issues,
            errors: &self.container.errors,
        })
    }
}

/// Original policy and splitter evidence for an unsupported envelope.
#[derive(Debug, Clone, Copy)]
pub struct Vue1DescriptorRefusal<'o> {
    issues: &'o [DescriptorIssue],
    errors: &'o [ContainerError],
}

impl Vue1DescriptorRefusal<'_> {
    pub fn issues(&self) -> &[DescriptorIssue] {
        self.issues
    }
    pub fn errors(&self) -> &[ContainerError] {
        self.errors
    }
}
