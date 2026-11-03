use super::{
    NativeComponent, NativeVueConstructionError, NativeWalkState, PendingNativeComponent,
    VueFileProducer,
};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::ProgramInput;

#[test]
fn ready_only_transfer_keeps_original_carrier_before_any_driver() {
    let a = Allocator::default();
    let source = "<p>{{value}}</p>";
    let original =
        NativeComponent::parse_in(&a, SourceRoot::new(source).unwrap().whole_block()).unwrap();
    let carrier = original.carrier().tree.children.as_ptr();
    let pending = PendingNativeComponent::from_component(original);
    let component = pending.into_unconstructed_component().unwrap();
    assert_eq!(component.carrier().tree.children.as_ptr(), carrier);
    assert_eq!(component.block().root_source().as_ptr(), source.as_ptr());
}

#[test]
fn actual_setup_profile_drives_ts_without_a_caller_template_language() {
    let a = Allocator::default();
    let source = "const value = 1;<p>{{value}}</p>";
    let script = source.get(..16).unwrap();
    let template = source.get(16..).unwrap();
    let root = SourceRoot::new(source).unwrap();
    let script_block = root.block(script, 0).unwrap();
    let block = root.block(template, 16).unwrap();
    let syntax = parse_program_once(
        &a,
        EmbedSource::authored(source, script_block.span()).unwrap(),
        ProgramOptions::module(Lang::Ts),
    );
    let mut producer = VueFileProducer::new(&a, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap())
        .unwrap();
    let component = NativeComponent::parse_in(&a, block).unwrap();
    let native = component
        .construct_vue_file_in(&mut producer.template_region().unwrap())
        .unwrap();
    assert_eq!(
        native
            .produced()
            .embeds
            .first()
            .unwrap()
            .syntax
            .grammar()
            .lang,
        Lang::Ts
    );
    assert_eq!(native.produced().component.block(), block);
    let vue = producer.finish().unwrap();
    assert!(vue.file().is_complete());
    assert_eq!(vue.setup().unwrap().source_type(), syntax.source_type());
}

#[test]
fn forgotten_real_walk_refuses_second_entry_and_cannot_issue_native_output() {
    let a = Allocator::default();
    let source = "<p>{{value}}</p>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(source).unwrap().whole_block()).unwrap();
    let span = component.block().span();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut producer = VueFileProducer::new(&a, source).unwrap();
    {
        let mut region = producer.template_region().unwrap();
        let walk = region.begin_native_walk(span).unwrap();
        core::mem::forget(walk);
        assert_eq!(
            pending.construct_vue_file_in(&mut region),
            Err(NativeVueConstructionError::Artifact(
                vize_l2::artifact::ArtifactError::InvalidSpan { node: None, span }
            ))
        );
    }
    assert_eq!(pending.state(), NativeWalkState::Refused);
    assert!(pending.observations().embeds().is_empty());
    assert_eq!(
        pending.into_vue_produced().unwrap_err().state(),
        NativeWalkState::Refused
    );
    let rejected = producer.finish().unwrap_err();
    let file = rejected.file().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.artifact().node_count(), 0);
    assert_eq!(
        file.template_issues(),
        &[vize_l2::file::TemplateIssue {
            node: None,
            span,
            kind: vize_l2::file::FileIssueKind::ActiveTemplateWalk
        }]
    );
}
