extern crate std;

use super::{NativeTemplateObservation, NativeTemplateOutcome, prepare};
use crate::native::{NativeVueConstructionError, Point, Probe};
use crate::native_file::{NativeScriptObservation, NativeSfcIssue, NativeSfcIssueKind, script};
use crate::vue_file::{VueFileIssueKind, VueFileProducer};
use alloc::vec::Vec;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    embed::Lang,
};
use vize_l2::{artifact::Builder, file::FileIssueKind};

fn row<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    NativeTemplateObservation<'a>,
    VueFileProducer<'a>,
    Vec<NativeScriptObservation<'a>>,
    Vec<NativeSfcIssue>,
) {
    let descriptor = Vue.observe_descriptor(
        allocator,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = descriptor.admitted().unwrap();
    let mut producer = VueFileProducer::new(allocator, source).unwrap();
    let mut issues = Vec::new();
    let scripts = [view.ordinary(), view.setup()]
        .into_iter()
        .flatten()
        .map(|selected| script::observe(allocator, selected, Some(&mut producer), &mut issues))
        .collect();
    let template = prepare(
        allocator,
        view.template().unwrap(),
        view.template_lang(),
        &mut issues,
    );
    assert!(issues.is_empty());
    (template, producer, scripts, issues)
}

fn assert_original_views(template: &NativeTemplateObservation<'_>) {
    let original = template.pending.as_ref().unwrap();
    assert!(core::ptr::eq(
        template.component().unwrap(),
        original.component()
    ));
    assert!(core::ptr::eq(
        template.holes(),
        original.observations().holes()
    ));
    assert!(core::ptr::eq(
        template.diagnostics(),
        original.observations().diagnostics()
    ));
    assert!(core::ptr::eq(
        template.embeds(),
        original.observations().embeds()
    ));
    assert!(core::ptr::eq(
        template.rejected_syntax(),
        original.observations().rejected_syntax()
    ));
    assert!(template.produced().is_none());
}

#[test]
fn sole_sfc_row_survives_real_native_unwinds_and_repeat_factory_refusal() {
    for (point, expected_embeds) in [
        (Point::BeforeSource, 0),
        (Point::BetweenExpressions, 2),
        (Point::AfterLastExpression, 2),
    ] {
        let arena = Allocator::default();
        let source = "<script setup>const first=1;const second=2;</script><template><p v-if=first>{{first /* kept */}}{{second}}</p></template>";
        let (mut template, mut producer, scripts, mut issues) = row(&arena, source);
        assert_eq!(scripts.len(), 1);
        let carrier = template
            .component()
            .unwrap()
            .carrier()
            .tree
            .children
            .as_ptr();
        let probe = Probe::arm(point);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                template.construct(Some(&mut producer), &mut issues);
            }))
            .is_err()
        );
        assert_eq!(probe.arena_at_panic(), Some(arena.allocated_bytes()));
        drop(probe);
        assert!(matches!(
            template.outcome(),
            NativeTemplateOutcome::ConstructionPending
        ));
        assert_original_views(&template);
        assert_eq!(
            template
                .component()
                .unwrap()
                .carrier()
                .tree
                .children
                .as_ptr(),
            carrier
        );
        assert_eq!(template.embeds().len(), expected_embeds);
        if expected_embeds == 2 {
            assert!(!template.holes().is_empty());
            assert!(!template.diagnostics().is_empty());
        }
        assert!(issues.is_empty());
        let ast = template
            .embeds()
            .first()
            .and_then(|embed| embed.syntax.expression());
        let embeds = template.embeds().as_ptr();
        template.construct(Some(&mut producer), &mut issues);
        assert!(matches!(
            template.outcome(),
            NativeTemplateOutcome::ConstructionPending
        ));
        assert_original_views(&template);
        assert_eq!(template.embeds().as_ptr(), embeds);
        assert!(matches!(issues.last().unwrap().kind,
            NativeSfcIssueKind::TemplateFactory(issue)
                if issue.kind == VueFileIssueKind::DuplicateTemplate));
        if let Some(original) = ast {
            assert!(core::ptr::eq(
                original,
                template
                    .embeds()
                    .first()
                    .unwrap()
                    .syntax
                    .expression()
                    .unwrap()
            ));
            assert_eq!(
                template
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
        let rejected = producer.finish().unwrap_err();
        let (interruption, retained_source) = match (rejected.file(), rejected.rejected_file()) {
            (Some(file), None) => (
                file.template_interruption().unwrap(),
                file.artifact().source(),
            ),
            (None, Some(file)) => (file.template_interruption().unwrap(), file.source()),
            _ => panic!("one genuine partial File owner"),
        };
        assert_eq!(interruption.kind, FileIssueKind::InterruptedTemplate);
        assert_eq!(interruption.span, template.block().span());
        assert!(core::ptr::eq(retained_source, source));
    }
}

#[test]
fn diagnostic_normal_return_cannot_admit_or_extract_the_started_sfc_row() {
    let arena = Allocator::default();
    let source = "<template><p>{{1 /* original */}}</p></template>";
    let (mut template, mut producer, _scripts, mut issues) = row(&arena, source);
    let carrier = template
        .component()
        .unwrap()
        .carrier()
        .tree
        .children
        .as_ptr();
    let mut builder = Builder::new(&arena, source).unwrap();
    template
        .pending
        .as_mut()
        .unwrap()
        .construct_diagnostic_in(&mut builder.region(), Lang::Js)
        .unwrap();
    assert_eq!(template.embeds().len(), 1);
    let ast = template
        .embeds()
        .first()
        .unwrap()
        .syntax
        .expression()
        .unwrap();
    assert!(template.unconstructed().is_none());
    assert_original_views(&template);
    template.construct(Some(&mut producer), &mut issues);
    assert!(matches!(
        template.outcome(),
        NativeTemplateOutcome::ConstructionRefused(NativeVueConstructionError::AlreadyStarted)
    ));
    assert_original_views(&template);
    assert_eq!(
        template
            .component()
            .unwrap()
            .carrier()
            .tree
            .children
            .as_ptr(),
        carrier
    );
    assert!(core::ptr::eq(
        ast,
        template
            .embeds()
            .first()
            .unwrap()
            .syntax
            .expression()
            .unwrap()
    ));
    assert_eq!(builder.finish().unwrap().node_count(), 2);
    assert_eq!(producer.finish().unwrap().file().artifact().node_count(), 0);
}
