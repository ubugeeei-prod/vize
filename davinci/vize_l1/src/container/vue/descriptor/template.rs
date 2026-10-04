//! Original selected-template view and retained outer-frame names.

use super::{DescriptorObservation, Selection};
use vize_l0::{SourceBlock, SourceRoot, Span};

#[derive(Debug)]
pub(super) struct TemplateNamePair {
    pub(super) opening: Span,
    pub(super) closing: Span,
}

#[derive(Debug)]
pub(super) struct TemplateSelection<'a> {
    pub(super) selection: Selection<'a>,
    pub(super) names: Option<TemplateNamePair>,
}

/// Checked original ordinary HTML template content.
#[derive(Debug, Clone, Copy)]
pub struct TemplateView<'o, 'a> {
    pub(super) owner: &'o DescriptorObservation<'a>,
    pub(super) selected: &'o TemplateSelection<'a>,
}

impl<'o, 'a> TemplateView<'o, 'a> {
    pub fn source(&self) -> &'a str {
        self.owner.source()
    }
    pub fn container_index(&self) -> usize {
        self.selected.selection.index
    }
    pub fn block(&self) -> SourceBlock<'a> {
        self.selected.selection.block
    }

    /// Borrow names recorded at this owner's original splitter callback.
    /// This lexical frame receipt grants no Component grammar or native File.
    ///
    /// ```
    /// use vize_l0::{Allocator, Span, config::{VueDialect, VueVersion}};
    /// use vize_l1::{SurfaceParseOptions, container::vue::{Vue, DescriptorOptions}};
    /// let arena = Allocator::default();
    /// let options = DescriptorOptions {
    ///     version: VueVersion::V3, dialect: VueDialect::Vue,
    ///     template: SurfaceParseOptions::default(),
    /// };
    /// let owner = Vue.observe_descriptor(&arena, "<template>x</template>", options);
    /// let view = owner.admitted().unwrap().template().unwrap();
    /// let names = view.frame_names().unwrap();
    /// assert_eq!((names.opening(), names.closing()), (Span::new(1, 9), Span::new(13, 21)));
    /// assert!(names.accepts(&view));
    /// ```
    pub fn frame_names(
        self,
    ) -> Result<NativeTemplateFrameNames<'o, 'a>, NativeTemplateFrameNameRefusal> {
        use NativeTemplateFrameNameRefusal::{IncompleteBoundary, SourceMismatch};
        let selected = self.owner.template.as_ref().ok_or(IncompleteBoundary)?;
        if !core::ptr::eq(selected, self.selected) {
            return Err(SourceMismatch);
        }
        let root = self.owner.root.ok_or(SourceMismatch)?;
        let names = selected.names.as_ref().ok_or(IncompleteBoundary)?;
        let block = self
            .owner
            .container
            .blocks
            .get(self.container_index())
            .ok_or(SourceMismatch)?;
        if root.whole_block().span_of(block.name) != Some(names.opening)
            || !root.contains_span(names.opening)
            || !root.contains_span(names.closing)
            || names.opening.start >= names.opening.end
            || names.closing.start >= names.closing.end
            || block.open_tag.start > names.opening.start
            || names.opening.end > block.open_tag.end
            || names.opening.end > block.content.start
            || block.content.end > names.closing.start
            || !block.close_tag.is_some_and(|close| {
                close.start <= names.closing.start && names.closing.end <= close.end
            })
        {
            return Err(SourceMismatch);
        }
        Ok(NativeTemplateFrameNames {
            owner: self.owner,
            selected,
            root,
            names,
        })
    }
}

/// Original descriptor admission refuses missing, uncertain or self-closed frames.
/// A receipt never infers geometry from a public capture or whole closing span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateFrameNameRefusal {
    IncompleteBoundary,
    SourceMismatch,
}

/// Names borrowed from one actual descriptor and its original template selection.
/// Authored UTF8 spans exclude delimiters and whitespace; closing case is retained.
/// They belong to the whole source, outside the selected template content block.
///
/// A public capture or caller coordinates cannot mint a receipt:
/// ```compile_fail
/// use vize_l1::container::vue::NativeTemplateFrameNames;
/// let _ = NativeTemplateFrameNames { owner: todo!(), selected: todo!(), root: todo!(), names: todo!() };
/// ```
/// The original descriptor must remain alive:
/// ```compile_fail
/// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::vue::{Vue, DescriptorOptions}};
/// let options = DescriptorOptions {
///     version: VueVersion::V3, dialect: VueDialect::Vue,
///     template: SurfaceParseOptions::default(),
/// };
/// let arena = Allocator::default();
/// let names = {
///     let owner = Vue.observe_descriptor(&arena, "<template></template>", options);
///     owner.admitted().unwrap().template().unwrap().frame_names().unwrap()
/// };
/// let _ = names.opening();
/// ```
/// Its whole-root projection cannot outlive the original source bytes:
/// ```compile_fail
/// use vize_l0::{Allocator, String, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::vue::{Vue, DescriptorOptions}};
/// let options = DescriptorOptions {
///     version: VueVersion::V3, dialect: VueDialect::Vue,
///     template: SurfaceParseOptions::default(),
/// };
/// let arena = Allocator::default();
/// let block = {
///     let source = String::from("<template></template>");
///     let owner = Vue.observe_descriptor(&arena, &source, options);
///     owner.admitted().unwrap().template().unwrap().frame_names().unwrap().source_block()
/// };
/// let _ = block.source();
/// ```
/// The original arena must also remain alive:
/// ```compile_fail
/// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::vue::{Vue, DescriptorOptions}};
/// let options = DescriptorOptions {
///     version: VueVersion::V3, dialect: VueDialect::Vue,
///     template: SurfaceParseOptions::default(),
/// };
/// let owner = {
///     let arena = Allocator::default();
///     Vue.observe_descriptor(&arena, "<template></template>", options)
/// };
/// let _ = owner.admitted().unwrap().template().unwrap().frame_names();
/// ```
/// Template views have no public constructor:
/// ```compile_fail
/// use vize_l1::container::vue::{DescriptorObservation, TemplateView};
/// fn forge(owner: &DescriptorObservation<'_>) {
///     let _ = TemplateView { owner, selected: todo!() };
/// }
/// ```
/// Receipts cannot detach a shared ownership handle:
/// ```compile_fail
/// use vize_l1::container::vue::NativeTemplateFrameNames;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeTemplateFrameNames<'static, 'static>>();
/// ```
pub struct NativeTemplateFrameNames<'o, 'a> {
    owner: &'o DescriptorObservation<'a>,
    selected: &'o TemplateSelection<'a>,
    root: SourceRoot<'a>,
    names: &'o TemplateNamePair,
}

impl core::fmt::Debug for NativeTemplateFrameNames<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeTemplateFrameNames")
            .field("container_index", &self.container_index())
            .field("opening", &self.opening())
            .field("closing", &self.closing())
            .finish_non_exhaustive()
    }
}

impl<'o, 'a> NativeTemplateFrameNames<'o, 'a> {
    #[must_use]
    pub fn template(&self) -> TemplateView<'o, 'a> {
        TemplateView {
            owner: self.owner,
            selected: self.selected,
        }
    }
    #[must_use]
    pub fn source_block(&self) -> SourceBlock<'a> {
        self.root.whole_block()
    }
    #[must_use]
    pub fn container_index(&self) -> usize {
        self.selected.selection.index
    }
    #[must_use]
    pub fn opening(&self) -> Span {
        self.names.opening
    }
    #[must_use]
    pub fn closing(&self) -> Span {
        self.names.closing
    }
    #[must_use]
    pub fn accepts(&self, view: &TemplateView<'_, 'a>) -> bool {
        core::ptr::eq(self.owner, view.owner) && core::ptr::eq(self.selected, view.selected)
    }
}

#[cfg(test)]
mod tests;
