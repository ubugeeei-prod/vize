use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    dump::{Dump, Mode},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::NativeTemplateOwner,
    op::{BindingOp, OnHandlerRef, Op},
};
use vize_l2_to_l3::lower;
use vize_l3::{
    operand::{OperandRole, ValueKind},
    values_dump::Page,
    verify::verify,
};

#[test]
fn original_file_handler_crosses_as_a_typed_id_without_js_source_or_diagnostic_authority() {
    let arena = Allocator::default();
    let l3_arena = Allocator::default();
    let source = "<template><button @click='/*original*/ return $event'/></template>";
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut original =
        NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("original owner"));
    {
        let mut walk = original.begin().unwrap();
        walk.child(walk.selected().children().next().unwrap())
            .unwrap();
        walk.complete().unwrap();
    }
    let output = original.finish();
    let file = output.view().unwrap().file().unwrap();
    let [Op::Element(button)] = file.artifact().root().ops.as_slice() else {
        panic!("button");
    };
    let [BindingOp::On(on)] = button.bindings.as_slice() else {
        panic!("actual event");
    };
    let id = on.handler.and_then(OnHandlerRef::body).unwrap();
    let handler = file.handler_for(on).unwrap();
    assert_eq!(
        handler
            .resolution()
            .unwrap()
            .input()
            .operand()
            .syntax()
            .comments()
            .count(),
        1
    );
    let s3 = lower(&l3_arena, file.artifact().root());
    assert_eq!(verify(&s3.program), []);
    let operand = s3
        .program
        .operands
        .iter()
        .find(|operand| operand.value.kind == ValueKind::NativeHandler)
        .unwrap();
    assert_eq!(operand.role, OperandRole::Value);
    assert_eq!(operand.value.native_handler, Some(id));
    assert_eq!(operand.value.text, "");
    assert_eq!(operand.value.qualifier, "");
    assert_eq!(operand.value.span, on.span);
    assert_eq!(operand.op.index(), id.node().index());
    let page = Page::of(&s3.program);
    let dump = page.print_to_string(Mode::Full);
    assert!(dump.contains("native.handler"));
    assert!(dump.contains(" handler-ref=1"));
    assert!(!dump.contains("return $event"));
    assert_eq!(Page::parse(&dump).unwrap(), page);
    assert!(Page::parse(&dump.replace("handler-ref=1", "handler-ref=2")).is_err());
    assert!(Page::parse(&dump.replace(" handler-ref=1", "")).is_err());
    // Typed source carrier does not broaden the sole DOM decision policy yet.
    let analysis = vize_l2_to_l3::build_dom_file_decisions(file).unwrap();
    assert!(!analysis.dom().unwrap().unsupported().is_empty());
    assert_eq!(
        core::mem::size_of::<vize_l3::operand::OperandValue<'_>>(),
        48
    );
}
