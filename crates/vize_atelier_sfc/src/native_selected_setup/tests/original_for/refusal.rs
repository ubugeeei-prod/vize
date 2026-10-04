use super::*;
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::targets::dom::DomErrorKind;

#[test]
fn actual_callback_helper_collisions_refuse_without_discarding_original_owners() {
    for alias in ["_openBlock", "_createElementBlock"] {
        let arena = Allocator::default();
        let source = format!(
            "<script setup>let count=2</script><template><i v-for='{alias} in count'>fixed</i></template>"
        );
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        assert!(matches!(
            compiled.result().err(),
            Some(NativeSelectedSetupSfcDomError::Dom(DomError {
                kind: DomErrorKind::GeneratedForBindingCollision,
                ..
            }))
        ));
        let view = compiled.observation().admitted().unwrap();
        let [Op::OriginalFor(original)] = view.setup().file().artifact().root().ops.as_slice()
        else {
            panic!("For")
        };
        let resolution = view
            .setup()
            .file()
            .for_head_for(original)
            .unwrap()
            .resolution()
            .unwrap();
        assert_eq!(resolution.value_declaration().fact().name(), alias);
        assert!(core::ptr::eq(
            view.setup().syntax().source().authored_root(),
            source.as_str()
        ));
        assert!(view.setup().file().is_complete());
    }
}

#[test]
fn dynamic_nested_keyed_attribute_and_mixed_root_bodies_remain_sticky_refusals() {
    for (template, reason) in [
        (
            "<i v-for='item in count'>{{count}}</i>",
            DomUnsupported::ForBody,
        ),
        (
            "<i v-for='item in count'>{{item}}</i>",
            DomUnsupported::ForBody,
        ),
        (
            "<i class='fixed' v-for='item in count'>fixed</i>",
            DomUnsupported::ForBody,
        ),
        ("<i v-for='item in count'><b/></i>", DomUnsupported::ForBody),
        (
            "<i v-for='item in count'><b v-for='child in item'/></i>",
            DomUnsupported::ForBody,
        ),
        (
            "<i v-for='(item,key) in count'>fixed</i>",
            DomUnsupported::ForShape,
        ),
        (
            "<i v-for='item in count'>fixed</i><b/>",
            DomUnsupported::ForShape,
        ),
        (
            "<i @click='let unused=$event;' v-for='item in count'>fixed</i>",
            DomUnsupported::ForBody,
        ),
    ] {
        let arena = Allocator::default();
        let source = format!("<script setup>let count=2</script><template>{template}</template>");
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert_eq!(
            compiled.result().err().map(|e| match e {
                NativeSelectedSetupSfcDomError::Dom(e) => e.kind,
                _ => panic!("genuine producer should complete"),
            }),
            Some(DomErrorKind::Unsupported(reason)),
            "{template}"
        );
        let view = compiled.observation().admitted().unwrap();
        assert!(view.setup().file().is_complete());
        let [Op::OriginalFor(original), ..] = view.setup().file().artifact().root().ops.as_slice()
        else {
            panic!("For")
        };
        assert!(
            view.setup()
                .file()
                .for_head_for(original)
                .unwrap()
                .resolution()
                .is_some()
        );
    }
}

#[test]
fn constant_generic_and_zero_occurrence_literal_for_never_gain_mutable_policy() {
    let arena = Allocator::default();
    let source = "<script setup>const count=2</script><template><i v-for='item in count'>fixed</i></template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    assert!(matches!(
        compiled.result().err(),
        Some(NativeSelectedSetupSfcDomError::Dom(DomError {
            kind: DomErrorKind::Unsupported(DomUnsupported::Operation),
            ..
        }))
    ));
    let view = compiled.observation().admitted().unwrap();
    let file = view.setup().file();
    let generic = vize_l3::decision::build_dom_file_decisions(file).unwrap();
    assert_eq!(
        generic.dom().unwrap().unsupported()[0].reason,
        DomUnsupported::Operation
    );
    let literal = compile_native_selected_setup_sfc_dom(
        &arena,
        "<script setup>let count=2</script><template><i v-for='item in 2'>fixed</i></template>",
        NativeSelectedSfcDomOptions::default(),
    );
    assert!(matches!(
        literal.result().err(),
        Some(NativeSelectedSetupSfcDomError::Observation)
    ));
    let owner = literal.observation().original().template().unwrap();
    let rejected = owner.rejected_file().unwrap();
    let [vize_l2::file::RejectedFileFor::Resolution { input, error }] =
        rejected.rejected_for_heads()
    else {
        panic!("whole zero-occurrence refusal");
    };
    assert_eq!(error.part, vize_l1::embed::syntax::ForHeadPart::Collection);
    assert_eq!(
        error.kind,
        vize_l2::resolution::ForResolutionErrorKind::Reference(
            vize_l2::resolution::ResolutionErrorKind::UnsupportedSyntax
        )
    );
    assert!(input.collection().is_literal());
    assert!(owner.retained_setup().is_some());
}
