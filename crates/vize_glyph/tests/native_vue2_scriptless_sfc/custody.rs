//! Complete original syntax/profile/map/comment/diagnostic/source facts.

use super::*;
use vize_l0::Span;
use vize_l1::{SurfaceChild, dialect::vue2::surface::TextChildren};

pub(super) fn facts(owner: &NativeVue2SfcObservation<'_>) -> std::vec::Vec<std::string::String> {
    let descriptor = owner.descriptor();
    let mut result = vec![format!(
        "{:p}/{}/{:?}/{:p}/{:?}/{:?}",
        owner.source().as_ptr(),
        owner.source().len(),
        owner.options(),
        descriptor.container().blocks.as_ptr(),
        descriptor.issues(),
        descriptor.container().errors
    )];
    if let Some(component) = descriptor.component() {
        result.push(format!(
            "{:?}/{:p}/{:p}/{:p}/{:?}/{:?}/{:?}",
            component.block(),
            component.tree().source.as_ptr(),
            component.tree().children.as_ptr(),
            component.bindings().as_ptr(),
            component.errors(),
            component.unsupported(),
            component.version()
        ));
        tokens(component.children(), &mut result);
        for binding in component.bindings() {
            result.push(format!(
                "{:p}/{:?}/{:p}/{}/{:p}/{:?}/{:?}",
                binding,
                binding.span(),
                binding.raw_content().as_ptr(),
                binding.raw_content().len(),
                binding.source().text().as_ptr(),
                binding.source().span(),
                binding.admitted().is_some()
            ));
            if let Some(chain) = binding.chain() {
                for original in core::iter::once(chain.base())
                    .chain(chain.filters().iter().flat_map(|filter| filter.arguments()))
                {
                    let source = original.source();
                    result.push(format!(
                        "{:p}/{:?}/{:?}/{:?}/{:p}/{}/{:p}/{}/{:?}/{:?}/{:?}/{:?}/{:?}",
                        original,
                        original.expression().map(core::ptr::from_ref),
                        original.grammar(),
                        original.source_type(),
                        source.authored_root().as_ptr(),
                        source.authored_root().len(),
                        source.text().as_ptr(),
                        source.text().len(),
                        source.span(),
                        original.hole(),
                        source
                            .decode_map()
                            .map(|map| (map.segments().as_ptr(), map.segments().to_vec())),
                        original
                            .comments()
                            .map(|comment| (
                                comment.kind(),
                                comment.decoded_span(),
                                comment.authored_span(),
                                comment.text().map(|text| (text.as_ptr(), text.to_owned()))
                            ))
                            .collect::<std::vec::Vec<_>>(),
                        original
                            .diagnostics()
                            .map(|diagnostic| (
                                diagnostic.message().as_ptr(),
                                diagnostic.message().len(),
                                format!("{diagnostic:?}")
                            ))
                            .collect::<std::vec::Vec<_>>()
                    ));
                    if let Some(view) = original.borrow_expression() {
                        result.push(format!(
                            "{:p}/{:p}/{:p}/{:?}/{:?}/{}",
                            view.expression(),
                            view.admitted_expression().original(),
                            view.admitted_expression().comments().as_ptr(),
                            (
                                view.admitted_expression().diagnostics().as_ptr(),
                                view.admitted_expression().diagnostics().len()
                            ),
                            (view.parser_prefix(), view.options()),
                            view.has_legacy_literals()
                        ));
                    }
                }
            }
        }
    }
    result
}
fn tokens(children: TextChildren<'_, '_>, output: &mut std::vec::Vec<std::string::String>) {
    for child in children {
        match child.surface() {
            SurfaceChild::Element(element) => {
                output.push(format!(
                    "{:p}/{:?}/{:?}/{:?}",
                    &**element,
                    child.ordinal(),
                    element.open,
                    element.close
                ));
                for attribute in &element.open.attrs {
                    output.push(format!(
                        "{:p}/{:p}/{:?}",
                        attribute,
                        attribute.name.text.as_ptr(),
                        attribute
                    ));
                }
                tokens(child.children().unwrap(), output);
            }
            node => output.push(format!("{:p}/{:?}/{:?}", node, child.ordinal(), node)),
        }
    }
}

#[test]
fn normally_moved_whole_owner_keeps_original_frame_every_operand_and_source_token() {
    let source = "\u{feff}<!--前🙂-->\r\n<template lang='html'><div id='雪'>甲{{ &#26085;&#26412;+1 | add(2+3,) | upper () }}乙<span>{{ a+b }}</span></div></template><!--尾-->\r\n";
    let arena = Allocator::default();
    let opts = options(200, 2, LineEnding::CrLf);
    let owner = observe_native_vue2_sfc_in(&arena, source, opts);
    original(&owner, source, opts);
    let before = facts(&owner);
    let block = owner.selected().unwrap().block().span();
    assert_eq!(
        block,
        Span::new(
            source.find("<div").unwrap() as u32,
            source.find("</template>").unwrap() as u32
        )
    );
    assert_eq!(owner.descriptor().component().unwrap().bindings().len(), 2);
    let base = owner.descriptor().component().unwrap().bindings()[0]
        .admitted()
        .unwrap()
        .base();
    assert!(base.source().decode_map().is_some());
    assert_eq!(base.source().text(), "日本+1");
    assert!(base.source_type().is_module());
    assert_eq!(base.hole(), None);
    let mut parked = vec![owner];
    parked.reserve(32);
    let moved = parked.pop().unwrap();
    assert_eq!(facts(&moved), before);
    let expected = "\u{feff}<!--前🙂-->\r\n<template lang='html'><div id='雪'>甲{{ &#26085;&#26412; + 1 | add(2 + 3,) | upper () }}乙<span>{{ a + b }}</span></div></template><!--尾-->\r\n";
    for _ in 0..3 {
        assert_eq!(moved.format().unwrap().code, expected);
        assert_eq!(facts(&moved), before);
    }
    fixed(source, expected, opts);
}
