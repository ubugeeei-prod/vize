extern crate std;

use super::super::{NativeWalkState, PendingNativeComponent};
use crate::native::{NativeComponent, NativeVueConstructionError, Point, Probe};
use crate::vue_file::VueFileProducer;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_l0::{Allocator, SourceRoot, id::NodeId};
use vize_l1::embed::Lang;
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::ProgramInput;
use vize_l2::{artifact::Builder, expr::ExprRef, file::FileIssueKind, op::Op};

mod admission;

#[test]
fn real_native_route_retains_original_ast_comments_and_actual_file_tables() {
    let a = Allocator::default();
    let script = "const first = 1; const second = 2;";
    let source = "const first = 1; const second = 2;<p>{{first /* kept */}}{{second}}</p>";
    let root = SourceRoot::new(source).unwrap();
    let script_block = root.block(source.get(..script.len()).unwrap(), 0).unwrap();
    let block = root
        .block(source.get(script.len()..).unwrap(), script.len() as u32)
        .unwrap();
    let syntax = parse_program_once(
        &a,
        EmbedSource::authored(source, script_block.span()).unwrap(),
        ProgramOptions::module(Lang::Js),
    );
    let component = NativeComponent::parse_in(&a, block).unwrap();
    let carrier = component.carrier().tree.children.as_ptr();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut producer = VueFileProducer::new(&a, source).unwrap();
    producer
        .setup(ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap())
        .unwrap();
    pending
        .construct_vue_file_in(&mut producer.template_region().unwrap())
        .unwrap();
    let root = pending
        .observations()
        .embeds()
        .first()
        .unwrap()
        .syntax
        .expression()
        .unwrap();
    assert_eq!(pending.state(), NativeWalkState::NormalEnd);
    let native = pending.into_vue_produced().unwrap();
    let produced = native.produced();
    assert!(produced.is_supported());
    assert_eq!(produced.component.carrier().tree.children.as_ptr(), carrier);
    assert_eq!(produced.embeds.len(), 2);
    assert_eq!(
        produced.embeds.first().unwrap().syntax.grammar().lang,
        Lang::Js
    );
    assert_eq!(
        produced
            .embeds
            .first()
            .unwrap()
            .syntax
            .comments()
            .next()
            .unwrap()
            .text()
            .unwrap(),
        "/* kept */"
    );
    assert!(core::ptr::eq(
        root,
        produced
            .embeds
            .first()
            .unwrap()
            .syntax
            .expression()
            .unwrap()
    ));
    let vue = producer.finish().unwrap();
    assert!(vue.file().is_complete());
    assert_eq!(vue.file().artifact().node_count(), 3);
    for embed in &produced.embeds {
        let table = vue
            .file()
            .expression(embed.node.unwrap())
            .unwrap()
            .table()
            .unwrap();
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert_eq!(table.expression().source, embed.syntax.source().text());
    }
}

#[test]
fn real_guard_keeps_original_owner_before_between_and_after_last_callbacks() {
    for (point, expected_embeds, expected_second_site) in [
        (Point::BeforeSource, 0, None),
        (Point::BetweenExpressions, 2, None),
        (
            Point::AfterLastExpression,
            2,
            Some(NodeId::from_index(2).unwrap()),
        ),
    ] {
        let a = Allocator::default();
        let script = "const first = 1; const second = 2;";
        let source = "const first = 1; const second = 2;<p>{{first /* kept */}}{{second}}</p>";
        let root = SourceRoot::new(source).unwrap();
        let script_block = root.block(source.get(..script.len()).unwrap(), 0).unwrap();
        let block = root
            .block(source.get(script.len()..).unwrap(), script.len() as u32)
            .unwrap();
        let span = block.span();
        let syntax = parse_program_once(
            &a,
            EmbedSource::authored(source, script_block.span()).unwrap(),
            ProgramOptions::module(Lang::Js),
        );
        let component = NativeComponent::parse_in(&a, block).unwrap();
        let carrier = component.carrier().tree.children.as_ptr();
        let mut pending = PendingNativeComponent::from_component(component);
        let mut producer = VueFileProducer::new(&a, source).unwrap();
        producer
            .setup(
                ProgramInput::checked(syntax.admitted_program().unwrap(), script_block, 0).unwrap(),
            )
            .unwrap();
        let probe = Probe::arm(point);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                pending
                    .construct_vue_file_in(&mut producer.template_region().unwrap())
                    .unwrap();
            }))
            .is_err()
        );
        assert_eq!(pending.state(), NativeWalkState::Interrupted);
        assert_eq!(
            pending.component().carrier().tree.children.as_ptr(),
            carrier
        );
        assert_eq!(pending.observations().embeds().len(), expected_embeds);
        // Only arena usage at the actual panic boundary is measured here.
        assert_eq!(probe.arena_at_panic(), Some(a.allocated_bytes()));
        drop(probe);
        if expected_embeds == 2 {
            let [_, second] = pending.observations().embeds() else {
                panic!("two original native expressions");
            };
            assert_eq!(
                pending.observations().embeds().first().unwrap().node,
                Some(NodeId::from_index(1).unwrap())
            );
            assert_eq!(second.node, expected_second_site);
            assert_eq!(
                pending
                    .observations()
                    .embeds()
                    .first()
                    .unwrap()
                    .syntax
                    .comments()
                    .next()
                    .unwrap()
                    .text()
                    .unwrap(),
                "/* kept */"
            );
        }
        let pending = pending.into_vue_produced().unwrap_err();
        assert_eq!(pending.state(), NativeWalkState::Interrupted);
        let rejected = producer.finish().unwrap_err();
        let (interruption, authored, ops) = match (rejected.file(), rejected.rejected_file()) {
            (Some(file), None) => (
                file.template_interruption().unwrap(),
                file.artifact().source(),
                file.artifact().root().ops.as_slice(),
            ),
            (None, Some(file)) => (
                file.template_interruption().unwrap(),
                file.source(),
                file.artifact().parts.root.ops.as_slice(),
            ),
            _ => panic!("one genuine partial File owner"),
        };
        assert_eq!(interruption.span, span);
        assert_eq!(interruption.kind, FileIssueKind::InterruptedTemplate);
        assert_eq!(authored.as_ptr(), source.as_ptr());
        if expected_embeds == 2 {
            let [Op::Element(element)] = ops else {
                panic!("partial actual owner")
            };
            let Op::Interpolation(first) = element.children.ops.first().unwrap() else {
                panic!("first actual interpolation")
            };
            let ExprRef::Js(js) = first.expression else {
                panic!("real once-parsed AST")
            };
            assert!(core::ptr::eq(
                js.ast,
                pending
                    .observations()
                    .embeds()
                    .first()
                    .unwrap()
                    .syntax
                    .expression()
                    .unwrap()
            ));
            assert_eq!(
                element.children.ops.len(),
                if expected_second_site.is_some() { 2 } else { 1 }
            );
        } else {
            assert!(ops.is_empty());
        }
    }
}

#[test]
fn diagnostic_normal_end_cannot_become_native_or_strip_started_observations() {
    let a = Allocator::default();
    let source = "<p>{{value}}</p>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(source).unwrap().whole_block()).unwrap();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut builder = Builder::new(&a, source).unwrap();
    pending
        .construct_diagnostic_in(&mut builder.region(), Lang::Js)
        .unwrap();
    let pending = pending.into_vue_produced().unwrap_err();
    assert_eq!(pending.state(), NativeWalkState::NormalEnd);
    assert_eq!(pending.observations().embeds().len(), 1);
    let pending = (*pending).into_unconstructed_component().unwrap_err();
    assert_eq!(pending.observations().embeds().len(), 1);
    assert_eq!(builder.finish().unwrap().node_count(), 2);
}

#[test]
fn foreign_equal_source_keeps_pending_and_rejects_native_before_any_ast_mint() {
    let a = Allocator::default();
    let original = vize_l0::String::from("<p>{{value}}</p>");
    let foreign = original.clone();
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(&original).unwrap().whole_block()).unwrap();
    let carrier = component.carrier().tree.children.as_ptr();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut producer = VueFileProducer::new(&a, &foreign).unwrap();
    assert_eq!(
        pending.construct_vue_file_in(&mut producer.template_region().unwrap()),
        Err(NativeVueConstructionError::ForeignSource)
    );
    assert_eq!(pending.state(), NativeWalkState::Refused);
    assert_eq!(
        pending.component().carrier().tree.children.as_ptr(),
        carrier
    );
    assert!(pending.observations().embeds().is_empty());
    assert_eq!(
        pending.into_vue_produced().unwrap_err().state(),
        NativeWalkState::Refused
    );
    let rejected = producer.finish().unwrap_err();
    assert_eq!(rejected.file().unwrap().artifact().node_count(), 0);
    assert_eq!(
        rejected
            .file()
            .unwrap()
            .template_interruption()
            .unwrap()
            .kind,
        FileIssueKind::InterruptedTemplate
    );
}
