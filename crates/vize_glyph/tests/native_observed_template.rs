//! Whole selected-template documents own authentic original body observations.

use vize_glyph::native_doc::{LineEnding, PrintOptions, observed_native_template_document, print};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, String};
use vize_l1::container::Vue;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1::markup::{NativeChildren, NativeInterpolationOperand, NativeTemplateComponent};
use vize_l1::{SurfaceChild, SurfaceParseOptions};

#[path = "native_observed_template/budgets.rs"]
mod budgets;
#[path = "native_observed_template/custody.rs"]
mod custody;
#[path = "native_observed_template/layout.rs"]
mod layout;
#[path = "native_observed_template/lifecycle.rs"]
mod lifecycle;
#[path = "native_observed_template/refusals.rs"]
mod refusals;

const WIDTHS: [usize; 5] = [0, 1, 7, 80, 200];
const SCRIPTS: [&str; 2] = ["", "<script setup lang=ts>let a=1</script>"];

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

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

fn newline(ending: LineEnding) -> &'static str {
    if ending == LineEnding::Lf {
        "\n"
    } else {
        "\r\n"
    }
}

fn format(source: &str, options: PrintOptions) -> String {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let document = observed_native_template_document(&owner, &arena).unwrap();
    assert!(core::ptr::eq(document.original(), &owner));
    assert_eq!(
        vize_l1::check_fidelity(&owner.component().carrier().tree),
        Ok(())
    );
    print(document.document(), &options)
}

fn assert_fixed(source: &str, script: &str) {
    for width in WIDTHS {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            let options = options(width, ending);
            let output = format(source, options);
            let replay = vize_l0::cstr!("<template>{output}</template>{script}");
            assert_eq!(format(&replay, options), output, "{source}");
        }
    }
}

// A test-only control for the independent preobserved public receiver.
fn preobserved<'a>(
    owner: &NativeTemplateComponent<'a>,
) -> std::vec::Vec<NativeInterpolationOperand<'a>> {
    fn visit<'a>(
        owner: &NativeTemplateComponent<'a>,
        children: NativeChildren<'_, 'a>,
        output: &mut std::vec::Vec<NativeInterpolationOperand<'a>>,
    ) {
        for child in children {
            match child.surface() {
                SurfaceChild::Element(_) => {
                    visit(owner, child.into_element().unwrap().children(), output)
                }
                SurfaceChild::Interpolation(_) => {
                    output.push(owner.observe_interpolation_expression(child).unwrap())
                }
                _ => {}
            }
        }
    }
    let mut output = std::vec::Vec::new();
    visit(owner, owner.children(), &mut output);
    output
}
