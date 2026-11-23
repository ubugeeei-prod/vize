use super::*;
use crate::{runtime::block_on, source_project::SourceSnapshotCache};
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::NoObserver,
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1_to_l2::pass::template::OriginalTemplate;

#[cfg(feature = "micro-profiling")]
mod profile;

fn whole_file_report(source: &str) -> Option<TemplateReport> {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let selected = descriptor.admitted_template().unwrap()?;
    let original = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
    Some(TemplateReport {
        opening: original.open_tag(),
        content: original.block().span(),
        complexity: original.analysis().complexity.clone(),
        diagnostics: original.l2ir().diagnostics.clone(),
    })
}

#[test]
fn decisions_and_diagnostics_follow_original_utf16_edits_and_guard_publication() {
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///component.vue").unwrap();
    let initial =
        "<script src='other.ts'></script>\r\n<template>🧭<p v-if='open && next'>雪</p></template>";
    state
        .documents
        .open(uri.clone(), initial.into(), 1, "vue".into());
    let (q, _) = cache.begin_query(&state.documents, &uri).unwrap();
    let active = &state;
    let old = block_on(q.run(&state.documents, |snapshot| async move {
        snapshot.template_report(active).unwrap().unwrap()
    }))
    .unwrap();
    let snap = cache.capture(&state.documents, &uri).unwrap();
    let first = snap.template_report(&state).unwrap().unwrap();
    assert_eq!(Some(&first), whole_file_report(initial).as_ref());
    assert_eq!(
        (first.complexity.cyclomatic, first.complexity.cognitive),
        (3, 2)
    );
    let [condition, _] = first.complexity.contributions.as_slice() else {
        panic!("expected condition and logical decision");
    };
    assert_eq!(condition.span.slice(snap.source()), "open && next");
    let changes = [("ready", 2), ("", 3), ("fixed", 4)];
    let mut end = 33;
    for (replacement, version) in changes {
        let range = Range::new(Position::new(1, 21), Position::new(1, end));
        assert!(state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: Some(range),
                range_length: None,
                text: replacement.into(),
            }],
            version
        ));
        end = 21 + replacement.len() as u32;
        let current = cache.capture(&state.documents, &uri).unwrap();
        let report = current.template_report(&state).unwrap().unwrap();
        let expected_source = initial.replace("open && next", replacement);
        assert_eq!(current.source(), expected_source);
        assert_eq!(Some(&report), whole_file_report(&expected_source).as_ref());
        if replacement.is_empty() {
            assert_eq!(report.diagnostics.len(), 1);
        } else {
            let [decision] = report.complexity.contributions.as_slice() else {
                panic!("expected repaired decision");
            };
            assert_eq!(decision.span.slice(current.source()), replacement);
            assert!(report.diagnostics.is_empty());
        }
    }
    assert_eq!(
        old.publish(&state.documents, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    assert_eq!(
        snap.template_report(&state),
        Err(TemplateRequestError::Host(SnapshotRefusal::Superseded))
    );
    let without_template = "<script src='other.ts'></script>";
    assert!(state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: without_template.into(),
        }],
        5,
    ));
    let last = cache.capture(&state.documents, &uri).unwrap();
    assert_eq!(last.template_report(&state), Ok(None));
    state.close_document(&uri);
    assert_eq!(
        last.template_report(&state),
        Err(TemplateRequestError::Host(SnapshotRefusal::MissingDocument))
    );
    state
        .documents
        .open(uri.clone(), initial.into(), 1, "html".into());
    assert_eq!(
        cache
            .capture(&state.documents, &uri)
            .unwrap()
            .template_report(&state),
        Err(TemplateRequestError::Language)
    );
}

#[test]
fn separate_documents_can_read_while_another_analysis_is_locked() {
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    let a = Url::parse("file:///first.vue").unwrap();
    let b = Url::parse("file:///second.vue").unwrap();
    let first = "<template><p v-if='first'>一</p></template>";
    let second = "<!--🧭-->\r\n<template><b :x=>二</b></template>";
    state
        .documents
        .open(a.clone(), first.into(), 1, "vue".into());
    state
        .documents
        .open(b.clone(), second.into(), 1, "vue".into());
    let original = cache.capture(&state.documents, &a).unwrap();
    assert_eq!(
        original.template_report(&state).unwrap(),
        whole_file_report(first)
    );
    let passes = state
        .documents
        .get(&a)
        .unwrap()
        .template_passes
        .get()
        .unwrap()
        .clone();
    let _busy = passes.lock();
    let other = cache.capture(&state.documents, &b).unwrap();
    assert_eq!(
        other.template_report(&state).unwrap(),
        whole_file_report(second)
    );
    state.close_document(&a);
    state
        .documents
        .open(a.clone(), second.into(), 1, "vue".into());
    let reopened = cache.capture(&state.documents, &a).unwrap();
    assert_eq!(
        reopened.template_report(&state).unwrap(),
        whole_file_report(second)
    );
    assert_eq!(
        original.template_report(&state),
        Err(TemplateRequestError::Host(SnapshotRefusal::Superseded))
    );
}
