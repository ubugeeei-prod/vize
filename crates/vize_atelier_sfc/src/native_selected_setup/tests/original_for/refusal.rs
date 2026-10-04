use super::*;
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::targets::dom::DomErrorKind;

#[test]
fn generated_alias_prefixes_keep_the_earliest_owned_resolution_refusal() {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_original_for_sfc_vue_3_5_35.json"
    ))
    .unwrap();
    let fixtures = pack["refusals"].as_array().unwrap();
    assert_eq!(fixtures.len(), 4);
    for (fixture, alias) in fixtures.iter().zip([
        "_renderList",
        "_Fragment",
        "_openBlock",
        "_createElementBlock",
    ]) {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert_eq!(
            compiled.result().err(),
            Some(NativeSelectedSetupSfcDomError::Observation)
        );
        assert!(compiled.observation().admitted().is_none());
        let owner = compiled.observation().original().template().unwrap();
        let [vize_l2::file::RejectedFileFor::Resolution { input, error }] = rejected_heads(owner)
        else {
            panic!("original prefix refusal: {alias}");
        };
        assert_eq!(error.part, vize_l1::embed::syntax::ForHeadPart::Aliases);
        assert_eq!(
            error.kind,
            vize_l2::resolution::ForResolutionErrorKind::ReservedAlias
        );
        assert_eq!(input.operand().raw_value(), format!("{alias} in count"));
        assert_eq!(input.aliases().len(), 1);
        assert!(matches!(
            input.collection(),
            oxc_ast::ast::Expression::Identifier(_)
        ));
        assert!(input.operand().syntax().aliases().unwrap().is_ok());
        assert!(input.operand().syntax().collection().unwrap().is_ok());
        assert!(core::ptr::eq(
            input.operand().syntax().source().authored_root(),
            source
        ));
        assert!(owner.retained_setup().is_some());
        assert!(owner.view().is_err());
    }
}

fn rejected_heads<'o, 'a>(
    owner: &'o vize_l2::lang::js::NativeTemplateFile<'a>,
) -> &'o [vize_l2::file::RejectedFileFor<'a>] {
    owner
        .file()
        .map(|file| file.rejected_for_heads())
        .or_else(|| owner.rejected_file().map(|file| file.rejected_for_heads()))
        .expect("normal outcome retains the original rejected head")
}

#[test]
fn dynamic_nested_keyed_attribute_and_mixed_root_bodies_remain_sticky_refusals() {
    for (template, reason) in [
        (
            "<i v-for='item in count'>{{count}}</i>",
            DomUnsupported::ForBody,
        ),
        // This exact original counterexample is now covered by the full
        // six-source original_for_value code/raw-map/runtime capture law.
        (
            "<i id='fixed' v-for='item in count'>fixed</i>",
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
                _ => panic!(
                    "genuine producer should complete: {template}, {:?}",
                    compiled.observation().original().issues()
                ),
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
    // A genuinely mutable selected setup succeeds; the same actual completed
    // File through the generic policy still has no original-loop runtime grant.
    for declaration in ["let count=2", "var count=2"] {
        let source = format!(
            "<script setup>{declaration}</script><template><i v-for='item in count'>fixed</i></template>"
        );
        let mutable = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(mutable.result().is_ok(), "{declaration}");
        let view = mutable.observation().admitted().unwrap();
        let selected = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
        assert!(selected.dom().unwrap().unsupported().is_empty());
        let generic = vize_l3::decision::build_dom_file_decisions(view.setup().file()).unwrap();
        assert_eq!(
            generic.dom().unwrap().unsupported()[0].reason,
            DomUnsupported::Operation
        );
    }
    // The exact original class header stays an earlier native header refusal.
    let class = compile_native_selected_setup_sfc_dom(
        &arena,
        "<script setup>let count=2</script><template><i class='fixed' v-for='item in count'>fixed</i></template>",
        NativeSelectedSfcDomOptions::default(),
    );
    assert_eq!(
        class.result().err(),
        Some(NativeSelectedSetupSfcDomError::Observation)
    );
    let owner = class.observation().original().template().unwrap();
    assert_eq!(
        owner.view().err().unwrap().kind,
        vize_l2::lang::js::NativeTemplateIssueKind::UnsupportedChild
    );
    assert!(owner.retained_setup().is_some());
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
    let [vize_l2::file::RejectedFileFor::Syntax(input)] = rejected_heads(owner) else {
        panic!("whole zero-occurrence refusal");
    };
    assert_eq!(
        input.kind,
        vize_l1::embed::syntax::NativeForRefusal::CollectionShape
    );
    assert_eq!(input.operand().raw_value(), "item in 2");
    assert!(
        input
            .operand()
            .syntax()
            .collection()
            .unwrap()
            .unwrap()
            .expression()
            .unwrap()
            .is_literal()
    );
    assert!(owner.retained_setup().is_some());
}
