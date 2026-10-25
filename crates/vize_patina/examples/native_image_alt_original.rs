//! Same original-header API observer copied byte-for-byte to base and head.
mod image_alt_measurement;

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    diag::MessageLookup,
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeChildren, NativeTemplateComponent, SurfaceParseOptions},
};
use vize_patina::native::{NativeLintFinding, NativeSyntaxLint};

fn collect<'a>(
    lint: &NativeSyntaxLint<'_, 'a>,
    children: NativeChildren<'_, 'a>,
    messages: &impl MessageLookup,
    output: &mut Vec<NativeLintFinding>,
) {
    for child in children {
        if let Some(element) = child.into_element() {
            if let Some(finding) = lint.img_alt(&element, messages).unwrap() {
                output.push(finding);
            }
            collect(lint, element.children(), messages, output);
        }
    }
}

fn main() {
    let a = Allocator::default();
    let source = format!("<template>{}</template>", image_alt_measurement::body());
    let descriptor = Vue.observe_descriptor(
        &a,
        &source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let original = NativeTemplateComponent::parse_in(&a, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let lint = NativeSyntaxLint::new(&original).unwrap();
    assert!(core::ptr::eq(lint.owner(), &original));
    let messages = image_alt_measurement::messages();
    let observation = image_alt_measurement::measure(
        "patina.native.original.img_alt.observer",
        "L1 NativeTemplateComponent + NativeSyntaxLint original children",
        &source,
        || {
            vize_l0::profile!("patina.native.original.img_alt.observer", {
                let mut output = Vec::new();
                collect(&lint, original.children(), &messages, &mut output);
                output
            })
        },
    );
    println!("{}", serde_json::to_string(&observation).unwrap());
}
