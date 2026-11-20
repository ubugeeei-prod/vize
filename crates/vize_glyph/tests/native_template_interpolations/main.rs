//! Native selected-template formatting through original interpolation owners.

use vize_glyph::native_doc::{PrintOptions, native_template_document, print};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, String};
use vize_l1::container::Vue;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1::markup::{ChildrenView, InterpolationOperand, SelectedTemplateComponent};
use vize_l1::markup::{SurfaceChild, SurfaceParseOptions};

mod array;
mod authored_newlines;
mod bigint;
mod call;
mod conditional;
mod layout;
mod member;
mod object;
mod preservation;
mod refusals;
mod sequence;
mod unary;

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> SelectedTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    SelectedTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

fn operands<'a>(
    selected: &SelectedTemplateComponent<'a>,
) -> std::vec::Vec<InterpolationOperand<'a>> {
    fn visit<'a>(
        selected: &SelectedTemplateComponent<'a>,
        children: ChildrenView<'_, 'a>,
        output: &mut std::vec::Vec<InterpolationOperand<'a>>,
    ) {
        for child in children {
            match child.surface() {
                SurfaceChild::Element(_) => {
                    visit(selected, child.into_element().unwrap().children(), output)
                }
                SurfaceChild::Interpolation(_) => {
                    output.push(selected.observe_interpolation_expression(child).unwrap())
                }
                _ => {}
            }
        }
    }
    let mut output = std::vec::Vec::new();
    visit(selected, selected.children(), &mut output);
    output
}

fn format(source: &str, options: PrintOptions) -> String {
    let arena = Allocator::default();
    let selected = selected(&arena, source);
    let operands = operands(&selected);
    let borrowed = operands.iter().collect::<std::vec::Vec<_>>();
    let doc = native_template_document(&selected, &borrowed, &arena).unwrap();
    assert!(core::ptr::eq(doc.original(), &selected));
    assert!(core::ptr::eq(doc.operands(), borrowed.as_slice()));
    assert_eq!(
        vize_l1::markup::check_fidelity(&selected.component().carrier().tree),
        Ok(())
    );
    print(doc.document(), &options)
}
