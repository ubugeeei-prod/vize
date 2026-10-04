//! One genuine descriptor, selected surface and observed document traversal.

use vize_l0::{Allocator, SourceFrameError, Span, Vec};
use vize_l1::container::Vue;
use vize_l1::container::vue::{AdmittedDescriptor, DescriptorObservation};
use vize_l1::markup::NativeTemplateComponent;

use super::super::{Doc, NativeTemplateValuePolicy, observed_native_template_document_with_policy};
use super::observation::Outcome;
use super::{NativeSfcBlockRole, NativeSfcObservation, NativeSfcOptions, NativeSfcRefusal};

/// Observe one complete original SFC under explicit native policies.
///
/// The current family requires a genuine ordinary Vue 3 HTML template with no
/// script/setup/style blocks. Descriptor policy refusals take priority, then
/// unsupported roles in original source order, selected surface construction,
/// the same observed-template child traversal and complete root framing.
/// Every refusal retains the actual descriptor/options, selected owner when
/// created, and every successfully made observation including the rejected one.
/// An actual interpolation observation failure is retained separately.
///
/// Prefix/opening-tag and closing-tag/suffix bytes remain authored. One full
/// Doc includes them in layout lookahead; no template-only print is spliced
/// into the outer source. This does not invoke any legacy formatter, replace a
/// default route, reparse an embed, normalize an AST or add a body traversal.
pub fn observe_native_sfc_in<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSfcOptions,
) -> NativeSfcObservation<'a> {
    let mut observation = NativeSfcObservation {
        descriptor: Vue.observe_descriptor(allocator, source, options.descriptor),
        selected: None,
        operands: std::vec::Vec::new(),
        interpolation_failure: None,
        options,
        outcome: Outcome::Refused(NativeSfcRefusal::Descriptor),
    };
    observation.outcome = match build(&mut observation, allocator) {
        Ok(document) => Outcome::Document(document),
        Err(refusal) => Outcome::Refused(refusal),
    };
    observation
}

fn build<'a>(
    observation: &mut NativeSfcObservation<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, NativeSfcRefusal> {
    let admitted = observation
        .descriptor
        .admitted()
        .map_err(|_| NativeSfcRefusal::Descriptor)?;
    refuse_other_roles(&observation.descriptor, admitted)?;
    let template = admitted
        .template()
        .ok_or(NativeSfcRefusal::MissingTemplate)?;
    let index = template.container_index();
    let content = template.block().span();
    observation.selected = NativeTemplateComponent::parse_in(allocator, admitted)
        .map_err(NativeSfcRefusal::Component)?;
    let selected = observation
        .selected
        .as_ref()
        .ok_or(NativeSfcRefusal::MissingTemplate)?;
    let template = match observed_native_template_document_with_policy(
        selected,
        allocator,
        NativeTemplateValuePolicy::RefuseValuedDirectives,
    ) {
        Ok(document) => {
            let (_, operands, document) = document.into_parts();
            observation.operands = operands;
            document
        }
        Err(failure) => {
            let (_, operands, refusal, original_failure) = failure.into_parts();
            observation.operands = operands;
            observation.interpolation_failure = original_failure;
            return Err(NativeSfcRefusal::Template(refusal));
        }
    };
    frame(
        &observation.descriptor,
        selected,
        index,
        content,
        template,
        allocator,
    )
}

fn refuse_other_roles(
    descriptor: &DescriptorObservation<'_>,
    admitted: AdmittedDescriptor<'_, '_>,
) -> Result<(), NativeSfcRefusal> {
    let candidates = [
        admitted.ordinary().map(|view| {
            (
                view.container_index(),
                NativeSfcBlockRole::Script(view.role()),
            )
        }),
        admitted.setup().map(|view| {
            (
                view.container_index(),
                NativeSfcBlockRole::Script(view.role()),
            )
        }),
        admitted
            .styles()
            .next()
            .map(|view| (view.container_index(), NativeSfcBlockRole::Style)),
    ];
    if let Some((index, role)) = candidates
        .into_iter()
        .flatten()
        .min_by_key(|(index, _)| *index)
    {
        let block =
            descriptor
                .container()
                .blocks
                .get(index)
                .ok_or(NativeSfcRefusal::SourceFrame {
                    span: Span::new(0, 0),
                    error: SourceFrameError::BlockOutOfBounds,
                })?;
        return Err(NativeSfcRefusal::UnsupportedBlock {
            index,
            span: block.open_tag,
            role,
        });
    }
    Ok(())
}

fn frame<'a>(
    descriptor: &DescriptorObservation<'a>,
    selected: &NativeTemplateComponent<'a>,
    index: usize,
    content: Span,
    template: Doc<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, NativeSfcRefusal> {
    let invalid = |error| NativeSfcRefusal::SourceFrame {
        span: content,
        error,
    };
    let root = descriptor
        .root()
        .ok_or_else(|| invalid(SourceFrameError::BlockOutOfBounds))?;
    let block = descriptor
        .container()
        .blocks
        .get(index)
        .ok_or_else(|| invalid(SourceFrameError::BlockOutOfBounds))?;
    let selected_block = selected.component().block();
    if selected.template_index() != index
        || block.content != content
        || selected_block.span() != content
        || !core::ptr::eq(selected_block.root_source(), descriptor.source())
    {
        return Err(invalid(SourceFrameError::BlockNotRootSlice));
    }
    root.block(selected_block.source(), content.start)
        .map_err(invalid)?;
    let close = block
        .close_tag
        .ok_or_else(|| invalid(SourceFrameError::BlockOutOfBounds))?;
    if block.open_tag.end != content.start
        || content.end != close.start
        || !root.contains_span(block.open_tag)
        || !root.contains_span(content)
        || !root.contains_span(close)
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
    parts.push(template);
    parts.push(Doc::text(suffix));
    Ok(Doc::concat(parts))
}
