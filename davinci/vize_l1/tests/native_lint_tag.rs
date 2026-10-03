//! Selected Vue 3 lint facts retain original custody and bounded grammar.

use vize_l0::{
    Allocator, SourceRoot, Span,
    config::{VueDialect, VueVersion},
    cstr,
};
use vize_l1::{
    Element, ElementClose, OpenTag, SurfaceParseOptions, SurfaceTree, Token, check_fidelity,
    container::{Vue, vue::DescriptorOptions},
    markup::{
        NativeChildren, NativeComponent, NativeLintTagKind as Kind,
        NativeLintTagRefusal as Refusal, NativeTemplateComponent, NativeTemplateGrammar,
    },
    render,
};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

fn rendered(tree: &SurfaceTree<'_>) -> vize_l0::String {
    let mut text = vize_l0::String::default();
    render(tree, &mut |piece| text.push_str(piece));
    text
}

fn collect<'a>(
    component: &NativeComponent<'a>,
    children: NativeChildren<'_, 'a>,
    output: &mut Vec<(&'a str, Result<Kind, Refusal>)>,
) {
    for child in children {
        if let Some(element) = child.into_element() {
            let kind = element.lint_tag().map(|receipt| {
                assert!(core::ptr::eq(receipt.component(), component));
                assert!(core::ptr::eq(receipt.element(), element.surface()));
                let block = component.block();
                let opening = block.span_of(element.surface().open.lt_name.text).unwrap();
                assert_eq!(receipt.span(), Span::new(opening.start + 1, opening.end));
                assert_eq!(
                    receipt.span(),
                    block.span_of(element.surface().tag()).unwrap()
                );
                assert!(block.contains_block_span(receipt.span()));
                receipt.kind()
            });
            output.push((element.surface().tag(), kind));
            collect(component, element.children(), output);
        }
    }
}

fn tags<'a>(owner: &NativeTemplateComponent<'a>) -> Vec<(&'a str, Result<Kind, Refusal>)> {
    let component = owner.component();
    let before = rendered(&component.carrier().tree);
    let mut output = Vec::new();
    collect(component, owner.children(), &mut output);
    assert_eq!(rendered(&component.carrier().tree), before);
    assert_eq!(before.as_str(), component.block().source());
    assert_eq!(check_fidelity(&component.carrier().tree), Ok(()));
    output
}

#[path = "native_lint_tag/grammar.rs"]
mod grammar;
#[path = "native_lint_tag/ownership.rs"]
mod ownership;
