//! Selected Vue 3 lint facts retain original custody and bounded grammar.

#[cfg(test)]
mod native_lint_tag {

    use vize_l0::{
        Allocator, SourceRoot, Span,
        config::{VueDialect, VueVersion},
        cstr,
    };
    use vize_l1::{
        container::{Vue, vue::DescriptorOptions},
        markup::{
            ChildrenView, Element, ElementClose, OpenTag, ParsedComponent, SurfaceParseOptions,
            SurfaceTree, TagFactsKind as Kind, TagFactsRefusal as Refusal, TemplateComponent,
            TemplateGrammar, Token, check_fidelity, render,
        },
    };

    fn selected<'a>(arena: &'a Allocator, source: &'a str) -> TemplateComponent<'a> {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        TemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap()
    }

    fn rendered(tree: &SurfaceTree<'_>) -> vize_l0::String {
        let mut text = vize_l0::String::default();
        render(tree, &mut |piece| text.push_str(piece));
        text
    }

    fn collect<'a>(
        component: &ParsedComponent<'a>,
        children: ChildrenView<'_, 'a>,
        output: &mut Vec<(&'a str, Result<Kind, Refusal>)>,
    ) {
        for child in children {
            if let Some(element) = child.into_element() {
                let kind = element.tag_facts().map(|receipt| {
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

    fn tags<'a>(owner: &TemplateComponent<'a>) -> Vec<(&'a str, Result<Kind, Refusal>)> {
        let component = owner.component();
        let before = rendered(&component.carrier().tree);
        let mut output = Vec::new();
        collect(component, owner.children(), &mut output);
        assert_eq!(rendered(&component.carrier().tree), before);
        assert_eq!(before.as_str(), component.block().source());
        assert_eq!(check_fidelity(&component.carrier().tree), Ok(()));
        output
    }

    mod grammar;
    mod ownership;
    mod recovery;
    mod table;
}
