use super::{NativeWalkState, PendingConstructionError, PendingNativeComponent};
use crate::native::{NativeComponent, NativeHoleKind};
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_l0::{Allocator, SourceRoot, String, id::NodeId};
use vize_l1::embed::Lang;
use vize_l2::artifact::{ArtifactError, Builder};
use vize_l2::expr::ExprRef;
use vize_l2::op::Op;

extern crate std;
mod factory;
use factory::{Callbacks, Factory};

#[test]
fn caught_actual_callback_keeps_current_observations_and_partial_owner() {
    let a = Allocator::default();
    let file = "<p @click=\"unused\">{{first /* kept */}}{{second}}</p>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let carrier = component.carrier().tree.children.as_ptr();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut builder = Builder::new(&a, file).unwrap();
    let mut callbacks = Callbacks {
        source_panics: false,
        interpolation_panics: false,
        interpolation_records: 0,
        arena_at_panic: None,
    };
    let caught = catch_unwind(AssertUnwindSafe(|| {
        pending
            .construct_diagnostic_in(
                &mut Factory {
                    region: &mut builder.region(),
                    allocator: &a,
                    callbacks: &mut callbacks,
                },
                Lang::Js,
            )
            .unwrap();
    }));
    assert_eq!(
        caught.unwrap_err().downcast_ref::<&str>(),
        Some(&"actual second-expression provenance callback unwind")
    );
    assert_eq!(pending.state(), NativeWalkState::Interrupted);
    assert_eq!(
        pending.component().carrier().tree.children.as_ptr(),
        carrier
    );
    assert_eq!(
        pending.component().block().root_source().as_ptr(),
        file.as_ptr()
    );
    let observations = pending.observations();
    assert_eq!(observations.holes().len(), 1);
    assert_eq!(
        observations.holes().first().unwrap().kind,
        NativeHoleKind::Directive
    );
    assert_eq!(observations.diagnostics().len(), 1);
    assert!(observations.rejected_syntax().is_empty());
    assert_eq!(observations.embeds().len(), 2);
    assert_eq!(
        observations
            .embeds()
            .first()
            .unwrap()
            .syntax
            .source()
            .text(),
        "first /* kept */"
    );
    assert_eq!(
        observations
            .embeds()
            .first()
            .unwrap()
            .syntax
            .comments()
            .count(),
        1
    );
    assert_eq!(
        observations
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
    let [_, second] = observations.embeds() else {
        panic!("two retained original expressions");
    };
    assert_eq!(second.syntax.source().text(), "second");
    assert_eq!(
        observations.embeds().first().unwrap().node,
        Some(NodeId::from_index(1).unwrap())
    );
    assert_eq!(second.node, Some(NodeId::from_index(2).unwrap()));
    // Arena-only measurement; no whole-heap allocation claim is made by this law.
    assert_eq!(Some(a.allocated_bytes()), callbacks.arena_at_panic);
    let pending = pending.into_produced().unwrap_err();
    assert_eq!(pending.state(), NativeWalkState::Interrupted);
    let rejected = builder.finish().unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        }
    );
    assert_eq!(rejected.parts.provenance.len(), 3);
    let [Op::Element(element)] = rejected.parts.root.ops.as_slice() else {
        panic!("actual pending element")
    };
    let [Op::Interpolation(first), Op::Interpolation(second)] = element.children.ops.as_slice()
    else {
        panic!("actual partial interpolation nodes")
    };
    let (ExprRef::Js(first), ExprRef::Js(second)) = (first.expression, second.expression) else {
        panic!("genuine retained roots")
    };
    assert!(core::ptr::eq(
        first.ast,
        pending
            .observations()
            .embeds()
            .first()
            .unwrap()
            .syntax
            .expression()
            .unwrap()
    ));
    let [_, retained_second] = pending.observations().embeds() else {
        panic!("two retained original expressions");
    };
    assert!(core::ptr::eq(
        second.ast,
        retained_second.syntax.expression().unwrap()
    ));
}

#[test]
fn first_source_callback_unwind_is_interrupted_before_any_mint() {
    let a = Allocator::default();
    let file = "<p>{{value}}</p>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let mut pending = PendingNativeComponent::from_component(component);
    let arena_before = a.allocated_bytes();
    let mut builder = Builder::new(&a, file).unwrap();
    let mut callbacks = Callbacks {
        source_panics: true,
        interpolation_panics: false,
        interpolation_records: 0,
        arena_at_panic: None,
    };
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            pending
                .construct_diagnostic_in(
                    &mut Factory {
                        region: &mut builder.region(),
                        allocator: &a,
                        callbacks: &mut callbacks,
                    },
                    Lang::Js,
                )
                .unwrap();
        }))
        .is_err()
    );
    assert_eq!(pending.state(), NativeWalkState::Interrupted);
    assert_eq!(pending.component().block().source(), file);
    assert!(pending.observations().embeds().is_empty());
    assert_eq!(a.allocated_bytes(), arena_before);
    assert_eq!(
        pending.construct_diagnostic_in(&mut builder.region(), Lang::Js),
        Err(PendingConstructionError::AlreadyStarted)
    );
    assert_eq!(builder.finish().unwrap().node_count(), 0);
    assert_eq!(
        pending.into_produced().unwrap_err().state(),
        NativeWalkState::Interrupted
    );
}

#[test]
fn pre_mint_callback_keeps_current_ast_without_inventing_a_node() {
    let a = Allocator::default();
    let file = "<p>{{value /* kept */}}</p>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut builder = Builder::new(&a, file).unwrap();
    let mut callbacks = Callbacks {
        source_panics: false,
        interpolation_panics: true,
        interpolation_records: 0,
        arena_at_panic: None,
    };
    let caught = catch_unwind(AssertUnwindSafe(|| {
        pending
            .construct_diagnostic_in(
                &mut Factory {
                    region: &mut builder.region(),
                    allocator: &a,
                    callbacks: &mut callbacks,
                },
                Lang::Js,
            )
            .unwrap();
    }));
    assert_eq!(
        caught.unwrap_err().downcast_ref::<&str>(),
        Some(&"actual pre-mint expression callback unwind")
    );
    assert_eq!(pending.state(), NativeWalkState::Interrupted);
    assert_eq!(pending.observations().embeds().len(), 1);
    let embed = pending.observations().embeds().first().unwrap();
    assert_eq!(embed.node, None);
    assert_eq!(embed.syntax.source().text(), "value /* kept */");
    assert_eq!(embed.syntax.comments().count(), 1);
    assert_eq!(
        embed.syntax.comments().next().unwrap().text().unwrap(),
        "/* kept */"
    );
    assert!(embed.syntax.expression().is_some());
    assert_eq!(Some(a.allocated_bytes()), callbacks.arena_at_panic);
    let rejected = builder.finish().unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        }
    );
    let [Op::Element(element)] = rejected.parts.root.ops.as_slice() else {
        panic!("actual unfinished owner")
    };
    assert!(element.children.ops.is_empty());
    assert_eq!(rejected.parts.provenance.len(), 1);
}

#[test]
fn foreign_equal_source_refuses_without_mint_or_discarding_original_owner() {
    let a = Allocator::default();
    let original = String::from("<p>{{value}}</p>");
    let foreign = original.clone();
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(&original).unwrap().whole_block()).unwrap();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut builder = Builder::new(&a, &foreign).unwrap();
    assert_eq!(
        pending.construct_diagnostic_in(&mut builder.region(), Lang::Js),
        Err(PendingConstructionError::ForeignSource)
    );
    assert_eq!(pending.state(), NativeWalkState::Refused);
    assert_eq!(
        pending.component().block().root_source().as_ptr(),
        original.as_ptr()
    );
    assert!(pending.observations().embeds().is_empty());
    assert_eq!(builder.finish().unwrap().node_count(), 0);
    let pending = pending.into_produced().unwrap_err();
    assert_eq!(pending.state(), NativeWalkState::Refused);
}

#[test]
fn normal_end_moves_the_same_owner_and_retained_ast_only_once() {
    let a = Allocator::default();
    let file = "<p>{{value /* kept */}}</p>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let carrier = component.carrier().tree.children.as_ptr();
    let mut pending = PendingNativeComponent::from_component(component);
    let mut builder = Builder::new(&a, file).unwrap();
    pending
        .construct_diagnostic_in(&mut builder.region(), Lang::Js)
        .unwrap();
    assert_eq!(pending.state(), NativeWalkState::NormalEnd);
    let root = pending
        .observations()
        .embeds()
        .first()
        .unwrap()
        .syntax
        .expression()
        .unwrap();
    assert_eq!(
        pending.construct_diagnostic_in(&mut builder.region(), Lang::Js),
        Err(PendingConstructionError::AlreadyStarted)
    );
    let produced = pending.into_produced().unwrap();
    assert!(produced.is_supported());
    assert_eq!(produced.component.carrier().tree.children.as_ptr(), carrier);
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
    assert_eq!(
        produced.embeds.first().unwrap().syntax.comments().count(),
        1
    );
    assert_eq!(builder.finish().unwrap().node_count(), 2);
}
