use super::*;
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::targets::dom::DomErrorKind;

#[test]
fn constant_same_owner_generic_and_neutral_policies_still_refuse_operation() {
    let arena = Allocator::default();
    let source = "<script setup>const count=2</script><template><i v-for='item in count'>{{item}}</i></template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    assert!(compiled.result().is_ok());
    let view = compiled.observation().admitted().unwrap();
    let selected = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
    assert!(selected.dom().unwrap().unsupported().is_empty());
    let generic = vize_l3::decision::build_dom_file_decisions(view.setup().file()).unwrap();
    assert_eq!(
        generic.dom().unwrap().unsupported()[0].reason,
        DomUnsupported::Operation
    );
    let neutral_view = compiled.observation().admitted().unwrap();
    let neutral = vize_l3::decision::native::build_native_dom_file_decisions(
        neutral_view.into_template_view(),
    )
    .unwrap();
    assert_eq!(
        neutral.dom().unwrap().unsupported()[0].reason,
        DomUnsupported::Operation
    );
}

#[test]
fn stable_mode_keeps_exact_outer_grouped_mixed_nested_and_literal_body_refusals() {
    for (body, reason) in [
        ("{{count}}", DomUnsupported::ForBody),
        ("{{2}}", DomUnsupported::ForBody),
        ("{{(item)}}", DomUnsupported::ForBody),
        ("{{item + 1}}", DomUnsupported::Expression),
        ("fixed{{item}}", DomUnsupported::ForBody),
        ("{{item}}{{item}}", DomUnsupported::ForBody),
        ("<b>{{item}}</b>", DomUnsupported::ForBody),
    ] {
        let arena = Allocator::default();
        let source = format!(
            "<script setup>const count=2</script><template><i v-for='item in count'>{body}</i></template>"
        );
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        let view = compiled.observation().admitted().unwrap();
        assert!(view.setup().file().is_complete());
        assert!(
            matches!(compiled.result().err(), Some(NativeSelectedSetupSfcDomError::Dom(DomError {
            kind: DomErrorKind::Unsupported(actual), ..
        })) if actual == reason),
            "{body}: {:?}",
            compiled.result().err()
        );
    }
}

#[test]
fn stable_mode_still_requires_one_unkeyed_attribute_free_root_carrier() {
    for (template, reason) in [
        (
            "<i v-for='(item, index) in count'>{{item}}</i>",
            DomUnsupported::ForShape,
        ),
        (
            "<i title='fixed' v-for='item in count'>{{item}}</i>",
            DomUnsupported::ForBody,
        ),
        (
            "<i v-for='item in count'>fixed</i><b>outside</b>",
            DomUnsupported::ForShape,
        ),
        (
            "<b><i v-for='item in count'>fixed</i></b>",
            DomUnsupported::ForShape,
        ),
    ] {
        let arena = Allocator::default();
        let source = format!("<script setup>const count=2</script><template>{template}</template>");
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(
            compiled.observation().admitted().is_some(),
            "{template}: real complete owner"
        );
        assert!(
            matches!(compiled.result().err(), Some(NativeSelectedSetupSfcDomError::Dom(DomError {
            kind: DomErrorKind::Unsupported(actual), ..
        })) if actual == reason),
            "{template}: {:?}",
            compiled.result().err()
        );
    }
}

#[test]
fn new_vnode_helper_alias_keeps_earliest_reserved_alias_and_whole_original_input() {
    let arena = Allocator::default();
    let source = "<script setup>const count=2</script><template><i v-for='_createElementVNode in count'>fixed</i></template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    assert_eq!(
        compiled.result().err(),
        Some(NativeSelectedSetupSfcDomError::Observation)
    );
    let owner = compiled.observation().original().template().unwrap();
    let rejected = owner
        .file()
        .map(|file| file.rejected_for_heads())
        .or_else(|| owner.rejected_file().map(|file| file.rejected_for_heads()))
        .unwrap();
    let [vize_l2::file::RejectedFileFor::Resolution { input, error }] = rejected else {
        panic!("whole reserved alias");
    };
    assert_eq!(
        error.kind,
        vize_l2::resolution::ForResolutionErrorKind::ReservedAlias
    );
    assert_eq!(input.operand().raw_value(), "_createElementVNode in count");
    assert_eq!(input.aliases().len(), 1);
    assert!(input.collection().is_identifier_reference());
    assert!(core::ptr::eq(
        input.operand().syntax().source().authored_root(),
        source
    ));
    assert!(owner.retained_setup().is_some());
}

#[test]
fn constant_syntax_hole_remains_rejected_file_with_its_whole_normal_original_owners() {
    let arena = Allocator::default();
    let source = "<script setup>const count=2</script><template><i v-for='item in count'>{{item +}}</i></template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    assert_eq!(
        compiled.result().err(),
        Some(NativeSelectedSetupSfcDomError::Observation)
    );
    let owner = compiled.observation().original().template().unwrap();
    assert!(owner.file().is_none());
    let file = owner.rejected_file().unwrap();
    assert!(file.template_interruption().is_some());
    assert!(core::ptr::eq(file.source(), source));
    let mut inputs = file.original_for_inputs();
    let input = inputs.next().unwrap();
    assert!(inputs.next().is_none());
    assert_eq!(input.operand().raw_value(), "item in count");
    assert_eq!(input.aliases().len(), 1);
    let [record] = file.native_interpolations() else {
        panic!("normally parked original");
    };
    assert_eq!(record.input().operand().raw_content(), "item +");
    assert_eq!(
        record.input().operand().syntax().hole(),
        Some(vize_l1::embed::syntax::EmbedHole::Syntax)
    );
    assert!(owner.retained_setup().is_some());
}
