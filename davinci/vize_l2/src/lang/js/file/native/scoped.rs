//! Physical original style/template/File custody, without generated scope policy.

use super::{NativeTemplateFile, NativeTemplateIssue, NativeTemplateView};
use crate::file::FileArtifact;
use vize_l0::{
    Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::vue::{AdmittedDescriptor, DescriptorObservation},
    css::{EmptyClassStyle, StyleIssue, StyleSyntax},
    markup::NativeTemplateGrammar,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeScopedTemplateIssueKind {
    Descriptor,
    Profile,
    Script,
    StyleCount,
    StyleProfile,
    StyleSyntax(StyleIssue),
    Template,
    Source,
    Completion(NativeTemplateIssue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeScopedTemplateIssue {
    pub span: Span,
    pub kind: NativeScopedTemplateIssueKind,
}

/// Checked immutable physical source/block/File identity, not Descriptor
/// allocation identity. Two genuine observations of the same physical buffer
/// may join. The caller supplies neither a File nor an eligibility flag.
/// Normal lower orchestration separately owns its one original CSS parse.
///
/// ```compile_fail
/// use vize_l1::{container::vue::AdmittedDescriptor, css::EmptyClassStyle};
/// use vize_l2::{file::FileArtifact, lang::js::{NativeScopedTemplateView, NativeTemplateView}};
/// fn forge<'o, 'a>(template: NativeTemplateView<'o, 'a>, file: &'o FileArtifact<'a>,
///     descriptor: AdmittedDescriptor<'o, 'a>, empty: EmptyClassStyle<'o, 'a>) {
///     let _ = NativeScopedTemplateView { template, file, descriptor, empty };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeScopedTemplateView;
/// fn copy(view: NativeScopedTemplateView<'_, '_>) { let _ = view.clone(); }
/// ```
/// The separately supplied original owners must remain alive:
/// ```compile_fail
/// use vize_l1::{container::vue::DescriptorObservation, css::StyleSyntax};
/// use vize_l2::lang::js::NativeTemplateFile;
/// fn discard_style<'a>(owner: &NativeTemplateFile<'a>, original: &DescriptorObservation<'a>, syntax: StyleSyntax<'a>) {
///     let view = owner.scoped_view(original, &syntax).unwrap();
///     drop(syntax); let _ = view.file();
/// }
/// ```
/// ```compile_fail
/// use vize_l1::{container::vue::DescriptorObservation, css::StyleSyntax};
/// use vize_l2::lang::js::NativeTemplateFile;
/// fn discard_descriptor<'a>(owner: &NativeTemplateFile<'a>, original: DescriptorObservation<'a>, syntax: &StyleSyntax<'a>) {
///     let view = owner.scoped_view(&original, syntax).unwrap();
///     drop(original); let _ = view.file();
/// }
/// ```
pub struct NativeScopedTemplateView<'owner, 'arena> {
    template: NativeTemplateView<'owner, 'arena>,
    file: &'owner FileArtifact<'arena>,
    descriptor: AdmittedDescriptor<'owner, 'arena>,
    empty: EmptyClassStyle<'owner, 'arena>,
}
impl<'owner, 'arena> NativeScopedTemplateView<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.template.owner()
    }
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }
    #[must_use]
    pub fn descriptor(&self) -> AdmittedDescriptor<'owner, 'arena> {
        self.descriptor
    }
    #[must_use]
    pub fn style_syntax(&self) -> &'owner StyleSyntax<'arena> {
        self.empty.syntax()
    }
    #[must_use]
    pub fn empty_style(&self) -> EmptyClassStyle<'owner, 'arena> {
        self.empty
    }
}

impl<'arena> NativeTemplateFile<'arena> {
    /// Derive this exact completed File internally, then join authentic source
    /// capabilities. Generated CSS, compiler scope IDs and neutral Files are
    /// never accepted here. All profile and physical membership checks apply
    /// to public callers as well as the private normally owned lower route.
    pub fn scoped_view<'owner>(
        &'owner self,
        original: &'owner DescriptorObservation<'arena>,
        syntax: &'owner StyleSyntax<'arena>,
    ) -> Result<NativeScopedTemplateView<'owner, 'arena>, NativeScopedTemplateIssue> {
        use NativeScopedTemplateIssueKind as Kind;
        let whole = Span::new(0, original.source().len() as u32);
        let reject = |kind| NativeScopedTemplateIssue { span: whole, kind };
        let options = original.options();
        if options.version != VueVersion::V3
            || options.dialect != VueDialect::Vue
            || options.template != SurfaceParseOptions::default()
        {
            return Err(reject(Kind::Profile));
        }
        let descriptor = original.admitted().map_err(|_| reject(Kind::Descriptor))?;
        let template = self.view().map_err(|issue| NativeScopedTemplateIssue {
            span: issue.span,
            kind: Kind::Completion(issue),
        })?;
        let file = template.file().ok_or_else(|| reject(Kind::Template))?;
        let selected = self.selected();
        if descriptor.ordinary().is_some()
            || descriptor.setup().is_some()
            || selected.ordinary().is_some()
            || selected.setup().is_some()
            || !file.units().is_empty()
        {
            return Err(reject(Kind::Script));
        }
        if descriptor.styles().len() != 1 {
            return Err(reject(Kind::StyleCount));
        }
        let style = descriptor
            .styles()
            .next()
            .ok_or_else(|| reject(Kind::StyleCount))?;
        if !style
            .attrs()
            .iter()
            .any(|attr| attr.name == "scoped" && attr.value.is_none())
            || style.attrs().iter().any(|attr| {
                !matches!(
                    (attr.name, attr.value),
                    ("scoped", None) | ("lang", Some("css"))
                )
            })
        {
            return Err(NativeScopedTemplateIssue {
                span: style.block().span(),
                kind: Kind::StyleProfile,
            });
        }
        let original_template = descriptor
            .template()
            .ok_or_else(|| reject(Kind::Template))?;
        let block = selected.component().block();
        if !selected.has_styles()
            || selected.grammar() != NativeTemplateGrammar::JavaScriptModule
            || selected.template_index() != original_template.container_index()
            || block.span() != original_template.block().span()
            || !core::ptr::eq(block.source(), original_template.block().source())
            || !core::ptr::eq(block.root_source(), descriptor.source())
            || !core::ptr::eq(file.artifact().source(), descriptor.source())
            || syntax.container_index() != style.container_index()
            || syntax.source().span() != style.block().span()
            || !core::ptr::eq(syntax.source().source(), style.block().source())
            || !core::ptr::eq(syntax.source().root_source(), descriptor.source())
        {
            return Err(reject(Kind::Source));
        }
        let empty = syntax
            .empty_class()
            .map_err(|issue| NativeScopedTemplateIssue {
                span: issue.span,
                kind: Kind::StyleSyntax(issue),
            })?;
        Ok(NativeScopedTemplateView {
            template,
            file,
            descriptor,
            empty,
        })
    }
}
