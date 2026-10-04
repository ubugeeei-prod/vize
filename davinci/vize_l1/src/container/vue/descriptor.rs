//! Source-owned admission for the first ordinary Vue 3 JS/TS SFC family.
//!
//! Capture records stay public; only the original splitter can create this
//! owner or its admitted views. No JavaScript/template syntax is parsed here.

use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, Vec};

use super::super::{Container, ContainerError};
use crate::{embed::Lang, parse::SurfaceParseOptions};

mod policy;
mod template;
pub(super) mod vue1;
pub(super) mod vue2;
pub use template::{NativeTemplateFrameNameRefusal, NativeTemplateFrameNames, TemplateView};
use template::{TemplateNamePair, TemplateSelection};
pub use vue1::{Vue1DescriptorObservation, Vue1DescriptorRefusal, Vue1TemplateView};
pub use vue2::{Vue2DescriptorObservation, Vue2DescriptorRefusal, Vue2TemplateView};
#[cfg(test)]
mod style_tests;
#[cfg(test)]
mod tests;

/// Explicit file policy, retained even when the observation is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DescriptorOptions {
    pub version: VueVersion,
    pub dialect: VueDialect,
    pub template: SurfaceParseOptions,
}

/// Why the first bounded descriptor family cannot admit this original input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorIssueCode {
    UnsupportedVersion,
    UnsupportedDialect,
    UnsupportedOptions,
    UnsupportedBlock,
    UnsupportedBlockSpelling,
    UnsupportedAttribute,
    DuplicateAttribute,
    AmbiguousAttribute,
    UnsupportedSetupValue,
    ExternalSource,
    EncodedLanguage,
    UnsupportedLanguage,
    LanguageMismatch,
    DuplicateRole,
    UnsupportedBoundary,
    InvalidSourceFrame,
    MissingComponentBlock,
    UnsupportedScript,
    UnsupportedStyle,
}

/// Original block index and authored evidence for a descriptor refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DescriptorIssue {
    pub code: DescriptorIssueCode,
    pub container_index: Option<usize>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptRole {
    Ordinary,
    Setup,
}

#[derive(Debug, Clone, Copy)]
struct Selection<'a> {
    index: usize,
    block: SourceBlock<'a>,
    lang: Lang,
}

#[derive(Debug, Clone, Copy)]
struct StyleSelection<'a> {
    index: usize,
    block: SourceBlock<'a>,
}

/// Complete original observation. Fields and all admission constructors are private.
///
/// Capture data cannot be promoted into admission:
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l1::container::{ContainerFormat, Vue};
/// use vize_l1::container::vue::DescriptorObservation;
/// let arena = Allocator::default();
/// let capture = Vue.split(&arena, "<script>x</script>");
/// let owner = DescriptorObservation::from_container(capture);
/// ```
/// Nor can its retained original capture be mutated:
/// ```compile_fail
/// use vize_l1::container::vue::DescriptorObservation;
/// fn change(owner: &mut DescriptorObservation<'_>) {
///     owner.container().blocks.clear();
/// }
/// ```
/// An admitted view itself has no public constructor:
/// ```compile_fail
/// use vize_l1::container::vue::{AdmittedDescriptor, DescriptorObservation};
/// fn forge(owner: &DescriptorObservation<'_>) {
///     let view = AdmittedDescriptor { owner, root: owner.root().unwrap() };
/// }
/// ```
/// Setup-only TypeScript determines the template's language:
/// ```
/// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::Vue, embed::Lang};
/// use vize_l1::container::vue::DescriptorOptions;
/// let arena = Allocator::default();
/// let source = "<template>{{ x }}</template><script setup lang=ts>const x=1</script>";
/// let options = DescriptorOptions {
///     version: VueVersion::V3, dialect: VueDialect::Vue,
///     template: SurfaceParseOptions::default(),
/// };
/// let owner = Vue.observe_descriptor(&arena, source, options);
/// let view = owner.admitted().expect("original supported descriptor");
/// assert!(view.ordinary().is_none());
/// assert_eq!(view.template_lang(), Lang::Ts);
/// assert_eq!(view.setup().unwrap().container_index(), 1);
/// ```
#[derive(Debug)]
pub struct DescriptorObservation<'a> {
    container: Container<'a>,
    root: Option<SourceRoot<'a>>,
    options: DescriptorOptions,
    issues: Vec<'a, DescriptorIssue>,
    ordinary: Option<Selection<'a>>,
    setup: Option<Selection<'a>>,
    template: Option<TemplateSelection<'a>>,
    styles: Vec<'a, StyleSelection<'a>>,
}

impl<'a> DescriptorObservation<'a> {
    pub fn source(&self) -> &'a str {
        self.container.source
    }
    pub fn root(&self) -> Option<SourceRoot<'a>> {
        self.root
    }
    pub fn container(&self) -> &Container<'a> {
        &self.container
    }
    pub fn options(&self) -> DescriptorOptions {
        self.options
    }
    pub fn issues(&self) -> &[DescriptorIssue] {
        &self.issues
    }

    /// Borrow admission from this exact original owner, never from a capture copy.
    pub fn admitted(&self) -> Result<AdmittedDescriptor<'_, 'a>, DescriptorRefusal<'_>> {
        if let Some(root) = self.root
            && self.issues.is_empty()
            && self.container.errors.is_empty()
        {
            Ok(AdmittedDescriptor { owner: self, root })
        } else {
            Err(DescriptorRefusal {
                issues: &self.issues,
                errors: &self.container.errors,
            })
        }
    }
}

/// Both native policy issues and unchanged original splitter diagnostics.
#[derive(Debug, Clone, Copy)]
pub struct DescriptorRefusal<'o> {
    issues: &'o [DescriptorIssue],
    errors: &'o [ContainerError],
}

impl DescriptorRefusal<'_> {
    pub fn issues(&self) -> &[DescriptorIssue] {
        self.issues
    }
    pub fn errors(&self) -> &[ContainerError] {
        self.errors
    }
}

/// Opaque, read-only capability borrowing its original descriptor observation.
#[derive(Debug, Clone, Copy)]
pub struct AdmittedDescriptor<'o, 'a> {
    owner: &'o DescriptorObservation<'a>,
    root: SourceRoot<'a>,
}

impl<'o, 'a> AdmittedDescriptor<'o, 'a> {
    pub fn root(self) -> SourceRoot<'a> {
        self.root
    }
    pub fn source(self) -> &'a str {
        self.owner.source()
    }
    pub fn ordinary(self) -> Option<ScriptView<'o, 'a>> {
        self.owner.ordinary.map(|selected| ScriptView {
            owner: self.owner,
            selected,
            role: ScriptRole::Ordinary,
        })
    }
    pub fn setup(self) -> Option<ScriptView<'o, 'a>> {
        self.owner.setup.map(|selected| ScriptView {
            owner: self.owner,
            selected,
            role: ScriptRole::Setup,
        })
    }
    pub fn template(self) -> Option<TemplateView<'o, 'a>> {
        self.owner.template.as_ref().map(|selected| TemplateView {
            owner: self.owner,
            selected,
        })
    }
    /// Opaque style sources in authored order; this grants no CSS semantics.
    pub fn styles(self) -> impl ExactSizeIterator<Item = StyleView<'o, 'a>> {
        self.owner
            .styles
            .iter()
            .copied()
            .map(move |selected| StyleView {
                owner: self.owner,
                selected,
            })
    }
    /// Matching real script roles determine template language; no scripts means JS.
    pub fn template_lang(self) -> Lang {
        self.owner
            .ordinary
            .or(self.owner.setup)
            .map_or(Lang::Js, |selected| selected.lang)
    }
}

/// Authentic script role and checked authored content; first policy is JS/TS Module.
#[derive(Debug, Clone, Copy)]
pub struct ScriptView<'o, 'a> {
    owner: &'o DescriptorObservation<'a>,
    selected: Selection<'a>,
    role: ScriptRole,
}

impl<'a> ScriptView<'_, 'a> {
    pub fn source(&self) -> &'a str {
        self.owner.source()
    }
    pub fn container_index(&self) -> usize {
        self.selected.index
    }
    pub fn block(&self) -> SourceBlock<'a> {
        self.selected.block
    }
    pub fn lang(&self) -> Lang {
        self.selected.lang
    }
    pub fn role(&self) -> ScriptRole {
        self.role
    }
}

/// Checked original style content and unchanged attributes, without CSS parsing.
/// `lang`, `scoped` and `module` are raw source metadata, not resolved profiles.
///
/// A public capture cannot forge an original style capability:
/// ```compile_fail
/// use vize_l1::container::vue::{DescriptorObservation, StyleView};
/// fn forge(owner: &DescriptorObservation<'_>) {
///     let view = StyleView { owner, selected: todo!() };
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StyleView<'o, 'a> {
    owner: &'o DescriptorObservation<'a>,
    selected: StyleSelection<'a>,
}

impl<'o, 'a> StyleView<'o, 'a> {
    pub fn source(&self) -> &'a str {
        self.owner.source()
    }
    pub fn container_index(&self) -> usize {
        self.selected.index
    }
    pub fn block(&self) -> SourceBlock<'a> {
        self.selected.block
    }
    #[expect(
        clippy::indexing_slicing,
        reason = "Only the original splitter records indices; the owner exposes no mutable blocks"
    )]
    pub fn original_block(&self) -> &'o crate::container::Block<'a> {
        &self.owner.container.blocks[self.selected.index]
    }
    pub fn attrs(&self) -> &'o [crate::container::BlockAttr<'a>] {
        &self.original_block().attrs
    }
}

pub(super) fn observe<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> DescriptorObservation<'a> {
    let mut state = policy::Policy::new(allocator, source, options);
    let container = super::split_with(
        allocator,
        source,
        |index, block, uncertain, self_closing, closing| {
            state.record(index, block, uncertain, self_closing, closing);
        },
    );
    state.finish();
    DescriptorObservation {
        container,
        root: state.root,
        options,
        issues: state.issues,
        ordinary: state.ordinary,
        setup: state.setup,
        template: state.template,
        styles: state.styles,
    }
}
