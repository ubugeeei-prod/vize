use super::*;
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
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::Op,
};

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
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
        arena,
        EmbedSource::authored(source, script.block().span()).unwrap(),
        ProgramOptions::module(script.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(arena, admitted)
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
    owner.finish()
}

#[test]
fn private_join_refuses_foreign_file_and_wrong_canonical_node() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let owner = completed(&arena, source);
    let foreign_owner = completed(&arena, source);
    let file = owner.file().unwrap();
    let foreign_file = foreign_owner.file().unwrap();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let [Op::OriginalFor(foreign)] = foreign_file.artifact().root().ops.as_slice() else {
        panic!("foreign For")
    };
    let node = original.id().node();
    assert_eq!(node, foreign.id().node());
    assert!(join(file, node, original).is_ok());
    assert!(matches!(
        join(foreign_file, node, original),
        Err(DomUnsupported::FileForHead)
    ));
    assert!(matches!(
        join(file, NodeId::from_index(1).unwrap(), original),
        Err(DomUnsupported::FileForHead)
    ));
    let head = file.for_head_for(original).unwrap();
    let foreign_head = foreign_file.for_head_for(foreign).unwrap();
    let expected = head.resolution().unwrap().value_declaration();
    assert!(!alias(
        foreign_head.value(),
        &expected,
        file,
        head.id(),
        head.scope().unwrap()
    ));
    assert!(!alias(
        head.value(),
        &expected,
        file,
        head.id(),
        head.enclosing_scope().unwrap()
    ));
}
