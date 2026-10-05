use vize_l0::{Allocator, SourceFrameError, Vec};
use vize_l1::container::{
    Vue,
    vue::{Vue2DescriptorObservation, Vue2TemplateView},
};

use super::super::{Doc, vue2_template};
use super::observation::Outcome;
use super::{NativeVue2SfcObservation, NativeVue2SfcOptions, NativeVue2SfcRefusal};

/// Keep one original splitter/Component and one authentic whole Doc visit.
/// Refusals retain the real descriptor and every existing stock observation.
/// The original prefix and suffix participate in the sole printer's lookahead.
/// This explicit bounded V2 family performs no extra parse/decode/AST/body pass.
pub fn observe_native_vue2_sfc_in<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeVue2SfcOptions,
) -> NativeVue2SfcObservation<'a> {
    let descriptor = Vue.observe_vue2_descriptor(allocator, source, options.descriptor);
    let outcome = match build(&descriptor, allocator) {
        Ok(document) => Outcome::Document(document),
        Err(refusal) => Outcome::Refused(refusal),
    };
    NativeVue2SfcObservation {
        descriptor,
        options,
        outcome,
    }
}

fn build<'a>(
    descriptor: &Vue2DescriptorObservation<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, NativeVue2SfcRefusal> {
    let selected = descriptor
        .selected()
        .map_err(|_| NativeVue2SfcRefusal::Descriptor)?;
    let document = vue2_template::document(selected.component(), allocator)?;
    frame(descriptor, &selected, document, allocator)
}

pub(super) fn frame<'a>(
    descriptor: &Vue2DescriptorObservation<'a>,
    selected: &Vue2TemplateView<'_, 'a>,
    document: Doc<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, NativeVue2SfcRefusal> {
    let content = selected.block().span();
    let invalid = |error| NativeVue2SfcRefusal::Frame {
        span: content,
        error,
    };
    let root = descriptor
        .root()
        .ok_or_else(|| invalid(SourceFrameError::BlockOutOfBounds))?;
    let block = descriptor
        .container()
        .blocks
        .get(selected.container_index())
        .ok_or_else(|| invalid(SourceFrameError::BlockOutOfBounds))?;
    if !core::ptr::eq(selected.observation(), descriptor)
        || !descriptor
            .component()
            .is_some_and(|original| core::ptr::eq(original, selected.component()))
        || !core::ptr::eq(root.source(), descriptor.source())
        || !core::ptr::eq(selected.block().root_source(), descriptor.source())
        || block.content != content
        || root.whole_block().span_of(block.name) != Some(selected.opening_name())
    {
        return Err(invalid(SourceFrameError::BlockNotRootSlice));
    }
    root.block(selected.block().source(), content.start)
        .map_err(invalid)?;
    let close = block
        .close_tag
        .ok_or_else(|| invalid(SourceFrameError::BlockOutOfBounds))?;
    if block.open_tag.end != content.start
        || content.end != close.start
        || !root.contains_span(block.open_tag)
        || !root.contains_span(content)
        || !root.contains_span(close)
        || !root.contains_span(selected.opening_name())
        || !root.contains_span(selected.closing_name())
        || selected.opening_name().start < block.open_tag.start
        || selected.opening_name().end > block.open_tag.end
        || selected.closing_name().start < close.start
        || selected.closing_name().end > close.end
    {
        return Err(invalid(SourceFrameError::BlockOutOfBounds));
    }
    let prefix = descriptor
        .source()
        .get(..content.start as usize)
        .ok_or_else(|| invalid(SourceFrameError::BlockBoundary))?;
    let suffix = descriptor
        .source()
        .get(content.end as usize..)
        .ok_or_else(|| invalid(SourceFrameError::BlockBoundary))?;
    let mut parts = Vec::new_in(&allocator);
    parts.push(Doc::text(prefix));
    parts.push(document);
    parts.push(Doc::text(suffix));
    Ok(Doc::concat(parts))
}
