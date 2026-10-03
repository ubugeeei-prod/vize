use super::classify_modifiers;

#[test]
fn whole_original_file_handler_is_refused_before_expression_emission() {
    use vize_l0::config::{VueDialect, VueVersion};
    use vize_l1::{
        SurfaceParseOptions,
        container::{Vue, vue::DescriptorOptions},
        markup::NativeTemplateComponent,
    };
    use vize_l2::{
        lang::js::NativeTemplateOwner,
        op::{BindingOp, Op},
    };
    let arena = vize_l0::Allocator::default();
    let source = "<template><button @click='return $event'/></template>";
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
    let file = output.file().unwrap();
    let [Op::Element(button)] = file.artifact().root().ops.as_slice() else {
        panic!("actual button");
    };
    let [BindingOp::On(on)] = button.bindings.as_slice() else {
        panic!("actual On");
    };
    assert!(file.handler_for(on).is_some());
    assert!(on.expression().is_none());
    let super::EmitError::Unsupported(refusal) = super::admit_on(on).unwrap_err() else {
        panic!("typed refusal");
    };
    assert_eq!(refusal.reason, super::Reason::OnHandlerNotJs);
    assert_eq!(refusal.span, Some(on.span));
}

#[test]
fn common_two_modifier_buckets_stay_inline() {
    let classified = classify_modifiers(
        "keyup",
        ["capture", "once", "stop", "prevent", "enter", "escape"],
    );

    assert!(!classified.options.spilled());
    assert!(!classified.event.spilled());
    assert!(!classified.keys.spilled());
}

#[test]
fn authored_modifiers_spill_without_a_length_ceiling() {
    let classified = classify_modifiers(
        "keyup",
        [
            "capture", "once", "passive", "stop", "prevent", "self", "enter", "escape", "space",
        ],
    );

    assert!(classified.options.spilled());
    assert!(classified.event.spilled());
    assert!(classified.keys.spilled());
    assert_eq!(
        classified.options.as_slice(),
        ["capture", "once", "passive"]
    );
    assert_eq!(classified.event.as_slice(), ["stop", "prevent", "self"]);
    assert_eq!(classified.keys.as_slice(), ["enter", "escape", "space"]);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn inline_storage_stack_tradeoff_is_pinned() {
    assert_eq!(core::mem::size_of::<super::Classified<'_>>(), 120);
    assert_eq!(core::mem::size_of::<super::OptionModifiers<'_>>() * 3, 120);
}
