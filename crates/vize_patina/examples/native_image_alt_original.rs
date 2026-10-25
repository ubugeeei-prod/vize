//! Same original-header API observer copied byte-for-byte to base and head.
mod image_alt_measurement;

use image_alt_measurement::{Result, ensure, failure};
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
) -> Result<()> {
    for child in children {
        if let Some(element) = child.into_element() {
            if let Some(finding) = lint.img_alt(&element, messages).map_err(|error| {
                failure(&vize_l0::cstr!("original-header query refused: {error:?}"))
            })? {
                output.push(finding);
            }
            collect(lint, element.children(), messages, output)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let a = Allocator::default();
    let source = vize_l0::cstr!("<template>{}</template>", image_alt_measurement::body());
    let descriptor = Vue.observe_descriptor(
        &a,
        &source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor
        .admitted()
        .map_err(|error| failure(&vize_l0::cstr!("original descriptor refused: {error:?}")))?;
    let original = NativeTemplateComponent::parse_in(&a, admitted)
        .map_err(|error| failure(&vize_l0::cstr!("original template refused: {error:?}")))?
        .ok_or_else(|| failure("missing selected original template"))?;
    let lint = NativeSyntaxLint::new(&original).map_err(|error| {
        failure(&vize_l0::cstr!(
            "original syntax context refused: {error:?}"
        ))
    })?;
    ensure(
        core::ptr::eq(lint.owner(), &original),
        "same original owner",
    )?;
    let messages = image_alt_measurement::messages();
    let observation = image_alt_measurement::measure(
        "patina.native.original.img_alt.observer",
        "L1 NativeTemplateComponent + NativeSyntaxLint original children",
        &source,
        || {
            vize_l0::profile!("patina.native.original.img_alt.observer", {
                let mut output = Vec::new();
                collect(&lint, original.children(), &messages, &mut output)?;
                Ok(output)
            })
        },
    )?;
    println!("{}", serde_json::to_string(&observation)?);
    Ok(())
}
