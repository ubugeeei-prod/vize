//! Absolute phases over genuine canonical owners, never a legacy/native ratio.
mod image_alt_measurement;

use image_alt_measurement::{Result, ensure, failure};
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::lang::js::JsxFileProducer;
use vize_patina::native::canonical::NativeCanonicalLint;

fn main() -> Result<()> {
    let a = Allocator::default();
    let messages = image_alt_measurement::messages();
    let body = image_alt_measurement::body();
    let source = vize_l0::cstr!("<template>{body}</template>");
    let owner = lower_sfc_native(
        &a,
        &source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = owner.admitted().ok_or_else(|| {
        failure(&vize_l0::cstr!(
            "native Vue owner refused: {:?}",
            owner.issues()
        ))
    })?;
    let lint = NativeCanonicalLint::from_vue(admitted);
    let file = owner
        .file()
        .ok_or_else(|| failure("missing genuine Vue File"))?;
    ensure(
        core::ptr::eq(lint.file(), file.file()),
        "same genuine Vue File",
    )?;
    let mut observations = vec![image_alt_measurement::measure(
        "patina.native.canonical.img_alt",
        "Vue NativeSfc canonical File",
        &source,
        || {
            lint.img_alt(&messages)
                .map_err(|error| failure(&vize_l0::cstr!("native Vue query refused: {error:?}")))
        },
    )?];
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
        let source = vize_l0::cstr!("const view = {body};");
        let original =
            oxc_parser::Parser::new(&a, &source, kind.with_module(true)).parse_observed();
        let block = SourceRoot::new(&source)
            .map_err(|error| failure(&vize_l0::cstr!("native source refused: {error:?}")))?
            .whole_block();
        let mut producer = JsxFileProducer::new(&a, original, block, 0).map_err(|error| {
            failure(&vize_l0::cstr!(
                "native Program refused: {:?}",
                error.error()
            ))
        })?;
        producer
            .walk()
            .map_err(|error| failure(&vize_l0::cstr!("native Program walk refused: {error:?}")))?;
        let owner = producer.finish_markup().map_err(|error| {
            failure(&vize_l0::cstr!(
                "canonical File refused: {:?}",
                error.issues()
            ))
        })?;
        let lint = NativeCanonicalLint::from_jsx(&owner)
            .map_err(|error| failure(&vize_l0::cstr!("native JSX context refused: {error:?}")))?;
        ensure(
            core::ptr::eq(lint.file(), owner.file()),
            "same genuine JSX File",
        )?;
        observations.push(image_alt_measurement::measure(
            "patina.native.canonical.img_alt",
            representation,
            &source,
            || {
                lint.img_alt(&messages).map_err(|error| {
                    failure(&vize_l0::cstr!("native JSX query refused: {error:?}"))
                })
            },
        )?);
    }
    println!("{}", serde_json::to_string(&observations)?);
    Ok(())
}
