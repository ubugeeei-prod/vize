//! Authentic L3 head custody must not silently grant a loop writer.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::NativeTemplateComponent,
};
use vize_l2::{lang::js::NativeTemplateOwner, op::Op};
use vize_l3::decision::{dom::DomUnsupported, native::build_native_dom_file_decisions};
use vize_l4::{
    targets::dom::{DomErrorKind, emit_template},
    write::Recorded,
};

#[test]
fn authentic_whole_head_still_returns_typed_dom_operation_refusal_before_output() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'>text</div></template>";
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().unwrap();
    let script = admitted.setup().unwrap();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, script.block().span()).unwrap(),
        ProgramOptions::module(script.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(&arena, admitted)
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
    owner
        .setup_program(syntax.admitted_program().unwrap())
        .unwrap();
    {
        let mut walk = owner.begin().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    let owner = owner.finish();
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    let [Op::OriginalFor(original)] = analysis.artifact().root().ops.as_slice() else {
        panic!("original")
    };
    let row = analysis.for_head(original.id().node()).unwrap();
    assert!(core::ptr::eq(
        row.resolution(),
        analysis
            .file()
            .for_head_for(original)
            .unwrap()
            .resolution()
            .unwrap()
    ));
    let error = match emit_template::<Recorded>(&analysis) {
        Ok(_) => panic!("head custody cannot mint executable loop output"),
        Err(error) => error,
    };
    assert_eq!(error.node, Some(original.id().node()));
    assert_eq!(error.span, original.span);
    assert_eq!(
        error.kind,
        DomErrorKind::Unsupported(DomUnsupported::Operation)
    );
}
