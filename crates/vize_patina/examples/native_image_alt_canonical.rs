//! Absolute phases over genuine canonical owners, never a legacy/native ratio.
mod image_alt_measurement;

use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::lang::js::JsxFileProducer;
use vize_patina::native::canonical::NativeCanonicalLint;

fn main() {
    let a = Allocator::default();
    let messages = image_alt_measurement::messages();
    let body = image_alt_measurement::body();
    let source = format!("<template>{body}</template>");
    let owner = lower_sfc_native(
        &a,
        &source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let lint = NativeCanonicalLint::from_vue(owner.admitted().unwrap());
    assert!(core::ptr::eq(lint.file(), owner.file().unwrap().file()));
    let mut observations = vec![image_alt_measurement::measure(
        "patina.native.canonical.img_alt",
        "Vue NativeSfc canonical File",
        &source,
        || lint.img_alt(&messages).unwrap(),
    )];
    for (kind, representation) in [
        (
            oxc_span::SourceType::jsx(),
            "JSX JsxMarkupFile same-Program canonical File",
        ),
        (
            oxc_span::SourceType::tsx(),
            "TSX JsxMarkupFile same-Program canonical File",
        ),
    ] {
        let source = format!("const view = {body};");
        let original =
            oxc_parser::Parser::new(&a, &source, kind.with_module(true)).parse_observed();
        let block = SourceRoot::new(&source).unwrap().whole_block();
        let mut producer = JsxFileProducer::new(&a, original, block, 0)
            .unwrap_or_else(|_| panic!("native Program"));
        producer.walk().unwrap();
        let owner = producer
            .finish_markup()
            .unwrap_or_else(|_| panic!("canonical File"));
        let lint = NativeCanonicalLint::from_jsx(&owner).unwrap();
        assert!(core::ptr::eq(lint.file(), owner.file()));
        observations.push(image_alt_measurement::measure(
            "patina.native.canonical.img_alt",
            representation,
            &source,
            || lint.img_alt(&messages).unwrap(),
        ));
    }
    println!("{}", serde_json::to_string(&observations).unwrap());
}
