use super::*;
use crate::document::DocumentStore;
use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};
use core::cell::Cell;
use tower_lsp::lsp_types::Url;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
};

#[test]
fn original_frame_hit_even_with_different_case_finishes_the_cancellable_cst_walk() {
    for source in [
        "<template><p></p></template>",
        "<template><p></p></TeMPLATE >",
    ] {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///App.vue").unwrap();
        store.open(uri.clone(), source.into(), 1, "vue".into());
        let snapshot = SourceSnapshotCache::default()
            .capture(&store, &uri)
            .unwrap();
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(
            &arena,
            snapshot.source(),
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let admitted = descriptor.admitted().unwrap();
        let frame = admitted.template().unwrap().frame_names().unwrap();
        let selected = NativeTemplateComponent::parse_in(&arena, admitted)
            .unwrap()
            .unwrap();
        let lines = vize_l0::line_index::LineBreaks::Lsp
            .line_starts(snapshot.source())
            .collect::<Vec<_>>();
        let query = TemplateNames {
            snapshot: &snapshot,
            selected: &selected,
            frame,
            lines: &lines,
            original: super::super::super::super::worker::linked::Inspection {
                descriptor: core::ptr::from_ref(&descriptor) as usize,
                selected: core::ptr::from_ref(&selected) as usize,
                component: core::ptr::from_ref(selected.component()) as usize,
                productions: 1,
            },
        };
        let visits = Cell::new(0);
        assert_eq!(
            query.ranges(Position::new(0, 1), || {
                visits.set(visits.get() + 1);
                true
            }),
            Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
        );
        assert_eq!(
            visits.get(),
            1,
            "neither a seeded pair nor case-mismatched null bypasses traversal"
        );
        assert!(query.frame.accepts(&admitted.template().unwrap()));
        assert_eq!(query.frame.container_index(), selected.template_index());
    }
}

#[test]
fn original_frame_keeps_later_body_traversal_cancellation_before_pair_publication() {
    let source = "<template><p></p><div></div></template>";
    let store = DocumentStore::new();
    let uri = Url::parse("file:///App.vue").unwrap();
    store.open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&store, &uri)
        .unwrap();
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        snapshot.source(),
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().unwrap();
    let frame = admitted.template().unwrap().frame_names().unwrap();
    let selected = NativeTemplateComponent::parse_in(&arena, admitted)
        .unwrap()
        .unwrap();
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(snapshot.source())
        .collect::<Vec<_>>();
    let query = TemplateNames {
        snapshot: &snapshot,
        selected: &selected,
        frame,
        lines: &lines,
        original: super::super::super::super::worker::linked::Inspection {
            descriptor: core::ptr::from_ref(&descriptor) as usize,
            selected: core::ptr::from_ref(&selected) as usize,
            component: core::ptr::from_ref(selected.component()) as usize,
            productions: 1,
        },
    };
    let visits = Cell::new(0);
    assert_eq!(
        query.ranges(Position::new(0, 1), || {
            visits.set(visits.get() + 1);
            visits.get() == 3
        }),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    assert_eq!(
        visits.get(),
        3,
        "later original traversal remains authoritative after candidate creation"
    );
}
