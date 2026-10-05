use super::*;
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::targets::dom::DomErrorKind;

#[test]
fn only_current_value_singleton_admits_while_outer_literal_mixed_and_compound_bodies_refuse() {
    for (body, reason) in [
        ("{{count}}", DomUnsupported::ForBody),
        ("{{2}}", DomUnsupported::ForBody),
        ("{{/*original*/item}}", DomUnsupported::ForBody),
        ("fixed{{item}}", DomUnsupported::ForBody),
        ("{{item}}fixed", DomUnsupported::ForBody),
        ("{{item}}{{item}}", DomUnsupported::ForBody),
        ("{{item + 1}}", DomUnsupported::Expression),
        ("{{item.name}}", DomUnsupported::Expression),
        ("{{(item)}}", DomUnsupported::ForBody),
        ("<b>{{item}}</b>", DomUnsupported::ForBody),
    ] {
        let arena = Allocator::default();
        let source = format!(
            "<script setup>let count=2</script><template><i v-for='item in count'>{body}</i></template>"
        );
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(
            compiled.observation().admitted().is_some(),
            "{body}: genuine owner"
        );
        assert!(
            matches!(compiled.result().err(),
            Some(NativeSelectedSetupSfcDomError::Dom(DomError {
                kind: DomErrorKind::Unsupported(actual), ..
            })) if actual == reason),
            "{body}: {:?}",
            compiled.result().err()
        );
        let view = compiled.observation().admitted().unwrap();
        assert!(view.setup().file().is_complete());
        assert!(view.setup().owner().retained_setup().is_some());
    }
}

#[test]
fn generic_and_neutral_same_mutable_owner_keep_operation_refusal() {
    let arena = Allocator::default();
    for declaration in ["let count=2", "var count=2"] {
        let source = format!(
            "<script setup>{declaration}</script><template><i v-for='item in count'>{{{{item}}}}</i></template>"
        );
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(compiled.result().is_ok());
        let view = compiled.observation().admitted().unwrap();
        let file = view.setup().file();
        let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            panic!("actual original For");
        };
        let head = file.for_head_for(original).unwrap();
        let [interpolation] = file.native_interpolations() else {
            panic!("one original Vue interpolation, never literal braces");
        };
        let vize_l2::file::NativeFileInterpolationState::Admitted(node) = interpolation.state()
        else {
            panic!("normally attached original interpolation");
        };
        let selected = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
        assert!(selected.dom().unwrap().unsupported().is_empty());
        let row = selected.expression(node).unwrap();
        let table = row.resolution().table().unwrap();
        let [read] = row.reads() else {
            panic!("exact current callback value singleton");
        };
        assert_eq!(table.occurrences().len(), 1);
        assert!(core::ptr::eq(read.occurrence(), &table.occurrences()[0]));
        assert_eq!(read.kind(), VueReadKind::ForValue);
        assert_eq!(read.binding().id(), head.value().unwrap().id());
        assert_eq!(row.resolution().scope(), head.scope());
        assert!(core::ptr::eq(
            table.expression().ast,
            interpolation
                .input()
                .operand()
                .syntax()
                .expression()
                .unwrap()
        ));
        let generic = vize_l3::decision::build_dom_file_decisions(file).unwrap();
        assert_eq!(
            generic.dom().unwrap().unsupported()[0].reason,
            DomUnsupported::Operation
        );
        let neutral = vize_l3::decision::native::build_native_dom_file_decisions(
            compiled
                .observation()
                .admitted()
                .unwrap()
                .into_template_view(),
        )
        .unwrap();
        assert_eq!(
            neutral.dom().unwrap().unsupported()[0].reason,
            DomUnsupported::Operation
        );
    }
}

#[test]
fn display_helper_alias_keeps_original_lower_reserved_alias_owner() {
    let arena = Allocator::default();
    let source = "<script setup>let count=2</script><template><i v-for='_toDisplayString in count'>{{_toDisplayString}}</i></template>";
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
        panic!("whole reserved alias input");
    };
    assert_eq!(
        error.kind,
        vize_l2::resolution::ForResolutionErrorKind::ReservedAlias
    );
    assert_eq!(input.operand().raw_value(), "_toDisplayString in count");
    assert_eq!(input.aliases().len(), 1);
    assert!(input.collection().is_identifier_reference());
    assert!(owner.retained_setup().is_some());
    assert!(core::ptr::eq(
        input.operand().syntax().source().authored_root(),
        source
    ));
}

#[test]
fn syntax_hole_keeps_whole_for_setup_and_original_body_without_read_admission() {
    let arena = Allocator::default();
    let source = "<script setup>let count=2</script><template><i v-for='item in count'>{{item +}}</i></template>";
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
    assert_eq!(
        owner.view().err().unwrap().kind,
        vize_l2::lang::js::NativeTemplateIssueKind::Interpolation(
            vize_l2::lang::js::NativeInterpolationInputError::Hole(
                vize_l1::embed::syntax::EmbedHole::Syntax
            )
        )
    );
    assert!(owner.file().is_none());
    let file = owner.rejected_file().unwrap();
    assert!(file.template_interruption().is_some());
    assert!(core::ptr::eq(file.source(), source));
    let mut inputs = file.original_for_inputs();
    let input = inputs.next().unwrap();
    assert!(inputs.next().is_none());
    assert_eq!(input.operand().raw_value(), "item in count");
    assert_eq!(input.aliases().len(), 1);
    assert!(input.collection().is_identifier_reference());
    let [record] = file.native_interpolations() else {
        panic!("normally parked original body");
    };
    assert_eq!(
        record.state(),
        vize_l2::file::NativeFileInterpolationState::Refused(
            vize_l2::lang::js::NativeTemplateIssueKind::Interpolation(
                vize_l2::lang::js::NativeInterpolationInputError::Hole(
                    vize_l1::embed::syntax::EmbedHole::Syntax
                )
            )
        )
    );
    assert_eq!(record.input().operand().raw_content(), "item +");
    assert_eq!(
        record.input().operand().syntax().hole(),
        Some(vize_l1::embed::syntax::EmbedHole::Syntax)
    );
    assert!(core::ptr::eq(
        record.input().operand().syntax().source().authored_root(),
        source
    ));
    assert!(core::ptr::eq(
        owner.retained_setup().unwrap().source().authored_root(),
        source
    ));
}
