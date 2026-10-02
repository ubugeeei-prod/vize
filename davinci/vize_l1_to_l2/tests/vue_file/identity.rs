use super::support::{block, construct, script};
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::NativeComponent;
use vize_l1_to_l2::vue_file::{VueFileIssueKind, VueFileProducer};
use vize_l2::lang::js::ProgramInput;

#[test]
fn equal_numeric_bindings_in_another_file_cannot_gain_vue_exposure() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
    let make = || {
        let mut producer = VueFileProducer::new(&arena, source).unwrap();
        producer
            .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
            .unwrap();
        producer.finish().unwrap()
    };
    let first = make();
    let second = make();
    let actual = first.file().bindings().next().unwrap();
    let foreign = second.file().bindings().next().unwrap();
    assert_eq!(actual.id(), foreign.id());
    assert!(first.exposure(actual).is_some());
    assert!(first.exposure(foreign).is_none());
}

#[test]
fn template_expression_literal_and_entity_coordinates_remain_original() {
    let arena = Allocator::default();
    let source = "<div :title=\"'a &amp; b'\">{{'日本語'}}</div>";
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let native = construct(&arena, source, source, &mut producer).unwrap();
    assert!(native.is_supported(), "{:?}", native.holes);
    let vue = producer.finish().unwrap();
    assert_eq!(native.embeds.len(), 2);
    for embed in native.embeds {
        let resolution = vue.file().expression(embed.node.unwrap()).unwrap();
        let table = resolution.table().unwrap();
        assert!(table.occurrences().is_empty());
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert_eq!(table.expression().span, embed.syntax.source().span());
    }
}

#[test]
fn duplicate_roles_late_scripts_and_duplicate_template_calls_are_retained_refusals() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let (syntax, block) = script(&arena, source, source, Lang::Js).unwrap();
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
        .unwrap();
    assert_eq!(
        producer
            .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 1).unwrap())
            .unwrap_err()
            .kind,
        VueFileIssueKind::DuplicateRole
    );
    assert_eq!(
        producer
            .ordinary(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 1).unwrap())
            .unwrap_err()
            .kind,
        VueFileIssueKind::OrdinaryAfterSetup
    );
    assert_eq!(
        producer.finish().unwrap_err().file().unwrap().units().len(),
        1
    );
    let mut producer = VueFileProducer::new(&arena, source).unwrap();
    let _ = producer.template_region().unwrap();
    assert_eq!(
        producer
            .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap())
            .unwrap_err()
            .kind,
        VueFileIssueKind::ScriptAfterTemplate
    );
    assert_eq!(
        producer.template_region().err().unwrap().kind,
        VueFileIssueKind::DuplicateTemplate
    );
    assert!(producer.finish().is_err());
}

#[test]
fn native_equal_copy_source_refuses_before_mint_and_retains_whole_component() {
    let arena = Allocator::default();
    let source = "<div>kept</div>";
    let copied = vize_l0::String::from(source);
    let component = NativeComponent::parse_in(&arena, block(source, source).unwrap()).unwrap();
    let mut producer = VueFileProducer::new(&arena, copied.as_str()).unwrap();
    let rejected = {
        let mut region = producer.template_region().unwrap();
        component.construct_in(&mut region, Lang::Js).unwrap_err()
    };
    assert_eq!(rejected.component.block().source(), source);
    assert!(core::ptr::eq(
        rejected.component.block().root_source(),
        source
    ));
    assert_eq!(producer.finish().unwrap().file().artifact().node_count(), 0);
}
