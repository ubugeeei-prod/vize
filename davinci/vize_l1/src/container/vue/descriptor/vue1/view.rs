use super::super::{TemplateNamePair, TemplateSelection};
use super::Vue1DescriptorObservation;
use crate::dialect::vue1::surface::ComponentParse;
use crate::parse::SurfaceParseOptions;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{SourceBlock, Span};

/// This original envelope's actual Component and splitter-selected frame.
/// No body grammar, File or runtime admission is implied.
///
/// All constructor fields are supplied with correct types; privacy rejects it:
/// ```compile_fail
/// use vize_l1::{container::vue::{Vue1DescriptorObservation, Vue1TemplateView},
///     dialect::vue1::surface::ComponentParse};
/// fn forge<'o, 'a>(owner: &'o Vue1DescriptorObservation<'a>,
///     component: &'o ComponentParse<'a>) {
///     let original = owner.selected().unwrap();
///     let _ = Vue1TemplateView { owner, component,
///         index: original.container_index(), opening: original.opening_name(),
///         closing: original.closing_name() };
/// }
/// ```
/// A view cannot outlive or move its normal owner:
/// ```compile_fail
/// use vize_l1::container::vue::Vue1DescriptorObservation;
/// fn discard(owner: Vue1DescriptorObservation<'_>) {
///     let view = owner.selected().unwrap();
///     drop(owner);
///     let _ = view.component();
/// }
/// ```
/// ```compile_fail
/// use vize_l1::container::vue::Vue1DescriptorObservation;
/// fn move_owner(owner: Vue1DescriptorObservation<'_>) {
///     let view = owner.selected().unwrap();
///     let moved = owner;
///     let _ = view.component();
///     drop(moved);
/// }
/// ```
/// No detached Clone handle or modern conversion is available:
/// ```compile_fail
/// use vize_l1::container::vue::Vue1TemplateView;
/// fn clone(view: Vue1TemplateView<'_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l1::container::vue::{AdmittedDescriptor, Vue1TemplateView};
/// fn modern<'o, 'a>(view: Vue1TemplateView<'o, 'a>) -> AdmittedDescriptor<'o, 'a> {
///     view.into()
/// }
/// ```
#[derive(Debug)]
pub struct Vue1TemplateView<'o, 'a> {
    owner: &'o Vue1DescriptorObservation<'a>,
    component: &'o ComponentParse<'a>,
    index: usize,
    opening: Span,
    closing: Span,
}

impl<'o, 'a> Vue1TemplateView<'o, 'a> {
    pub fn observation(&self) -> &'o Vue1DescriptorObservation<'a> {
        self.owner
    }
    pub fn component(&self) -> &'o ComponentParse<'a> {
        self.component
    }
    pub fn block(&self) -> SourceBlock<'a> {
        self.component.block()
    }
    pub fn container_index(&self) -> usize {
        self.index
    }
    pub fn opening_name(&self) -> Span {
        self.opening
    }
    pub fn closing_name(&self) -> Span {
        self.closing
    }
}

pub(super) fn frame<'o, 'a>(
    owner: &'o Vue1DescriptorObservation<'a>,
) -> Option<(&'o TemplateSelection<'a>, &'o TemplateNamePair)> {
    let root = owner.root?;
    if owner.options.version != VueVersion::V1
        || owner.options.dialect != VueDialect::Vue
        || owner.options.template != SurfaceParseOptions::default()
        || !core::ptr::eq(root.source(), owner.source())
    {
        return None;
    }
    let selected = owner.selection.as_ref()?;
    let block = owner.container.blocks.get(selected.selection.index)?;
    let names = selected.names.as_ref()?;
    let source = root
        .source()
        .get(block.content.start as usize..block.content.end as usize)?;
    if block.name != "template"
        || !block.attrs.is_empty()
        || selected.selection.block.span() != block.content
        || !core::ptr::eq(selected.selection.block.source(), source)
        || !core::ptr::eq(selected.selection.block.root_source(), root.source())
        || root.whole_block().span_of(block.name) != Some(names.opening)
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
        return None;
    }
    Some((selected, names))
}

pub(super) fn selected<'o, 'a>(
    owner: &'o Vue1DescriptorObservation<'a>,
) -> Option<Vue1TemplateView<'o, 'a>> {
    if !owner.issues.is_empty() || !owner.container.errors.is_empty() {
        return None;
    }
    let (selection, names) = frame(owner)?;
    let component = owner.component.as_ref()?;
    if component.block().span() != selection.selection.block.span()
        || !core::ptr::eq(
            component.block().source(),
            selection.selection.block.source(),
        )
        || !core::ptr::eq(component.block().root_source(), owner.source())
    {
        return None;
    }
    Some(Vue1TemplateView {
        owner,
        component,
        index: selection.selection.index,
        opening: names.opening,
        closing: names.closing,
    })
}
