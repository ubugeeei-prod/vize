//! Revalidate the actual original receiver without fabricating neutral scopes.

use crate::lang::js::NativeTemplateOwner;
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

#[test]
fn actual_original_for_tree_revalidates_with_file_owned_scope_custody() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'>body</div></template>";
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
    let setup = admitted.setup().unwrap();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, setup.block().span()).unwrap(),
        ProgramOptions::module(setup.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(&arena, admitted)
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("actual owner"));
    owner
        .setup_program(syntax.admitted_program().unwrap())
        .unwrap();
    {
        let mut walk = owner.begin().unwrap();
        walk.child(walk.selected().children().next().unwrap())
            .unwrap();
        walk.complete().unwrap();
    }
    let output = owner.finish();
    let file = output.view().unwrap().file().unwrap();
    let [crate::op::Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual original For");
    };
    let head = file.for_head_for(original).unwrap();
    assert_ne!(head.scope(), head.enclosing_scope());
    assert_eq!(file.artifact().scopes().len(), 0);
    assert_eq!(
        super::check(&file.artifact().parts).unwrap(),
        file.artifact().node_count()
    );
}
