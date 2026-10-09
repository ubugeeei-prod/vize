//! A real original File cannot bypass missing native For operand custody.

use crate::l3::{AdmissionFailure, retained::Retained};
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
use vize_l2::lang::js::NativeTemplateOwner;

#[test]
fn actual_original_for_generic_graph_does_not_mint_native_vapor_admission() {
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
    let [vize_l2::op::Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual original For");
    };
    let head = file.for_head_for(original).unwrap().resolution().unwrap();
    assert_eq!(head.input().operand().raw_value(), "item in items");
    let lowered = vize_l2_to_l3::lower(&arena, file.artifact().root());
    assert!(vize_l3::verify::verify(&lowered.program).is_empty());
    assert_eq!(lowered.program.ops[0].kind, vize_l3::op::OpKind::For);
    assert!(
        lowered
            .program
            .operands
            .iter()
            .all(|operand| operand.op != lowered.program.ops[0].id)
    );
    let retained = Retained::collect(&arena, file.artifact().root());
    let result = super::super::NativeArtifact::admit(&lowered, &retained, &[], source);
    assert!(matches!(
        result,
        Err(AdmissionFailure::Invalid("missing required native operand"))
    ));
    // The same normal whole File keeps the authentic collection/parameters;
    // neither the generic graph nor the typed refusal transfers that authority.
    assert!(file.for_head_for(original).is_some());
    assert!(file.is_complete());
}
