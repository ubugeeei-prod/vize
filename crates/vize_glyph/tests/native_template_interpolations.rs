//! Native selected-template formatting through original interpolation owners.

use vize_glyph::native_doc::{PrintOptions, native_template_document, print};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, String};
use vize_l1::container::Vue;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1::markup::{NativeChildren, NativeInterpolationOperand, NativeTemplateComponent};
use vize_l1::{SurfaceChild, SurfaceParseOptions};

#[path = "native_template_interpolations/authored_newlines.rs"]
mod authored_newlines;
#[path = "native_template_interpolations/call.rs"]
mod call;
#[path = "native_template_interpolations/conditional.rs"]
mod conditional;
#[path = "native_template_interpolations/layout.rs"]
mod layout;
#[path = "native_template_interpolations/member.rs"]
mod member;
#[path = "native_template_interpolations/preservation.rs"]
mod preservation;
#[path = "native_template_interpolations/refusals.rs"]
mod refusals;
#[path = "native_template_interpolations/unary.rs"]
mod unary;

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

fn operands<'a>(
    selected: &NativeTemplateComponent<'a>,
) -> std::vec::Vec<NativeInterpolationOperand<'a>> {
    fn visit<'a>(
        selected: &NativeTemplateComponent<'a>,
        children: NativeChildren<'_, 'a>,
        output: &mut std::vec::Vec<NativeInterpolationOperand<'a>>,
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
        vize_l1::check_fidelity(&selected.component().carrier().tree),
        Ok(())
    );
    print(doc.document(), &options)
}
