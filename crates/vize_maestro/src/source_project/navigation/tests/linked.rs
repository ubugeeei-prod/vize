//! Genuine lexical owners and original history coordinates, independent of File.
mod capacity;
mod lifecycle;
use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject, block_on,
};
use crate::server::ServerState;
use tower_lsp::lsp_types::{LinkedEditingRanges, Range, Url};
const SOURCE: &str = include_str!("../../../../tests/fixtures/native-linked-history-3471.vue");
fn uri() -> Url {
    Url::parse("file:///App.vue").unwrap()
}
fn project(source: &str) -> (Arc<ServerState>, NativeNavigationProject<'static>) {
    let state = Arc::new(ServerState::new());
    state.documents.open(uri(), source.into(), 1, "vue".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    (state, project)
}
fn worker(project: &NativeNavigationProject<'_>) -> Arc<super::super::worker::NavigationWorker> {
    Arc::clone(
        project
            .names_workers
            .lock()
            .get(&uri())
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    )
}
fn ranges(first: (u32, u32, u32), second: (u32, u32, u32)) -> Option<LinkedEditingRanges> {
    Some(LinkedEditingRanges {
        ranges: vec![
            Range::new(
                Position::new(first.0, first.1),
                Position::new(first.0, first.2),
            ),
            Range::new(
                Position::new(second.0, second.1),
                Position::new(second.0, second.2),
            ),
        ],
        word_pattern: None,
    })
}
#[test]
fn original_3471_inner_pair_and_outer_body_pair_have_complete_ordered_ranges() {
    let (_, project) = project(SOURCE);
    for position in [
        Position::new(6, 6),
        Position::new(6, 33),
        Position::new(6, 8),
    ] {
        assert_eq!(
            block_on(project.linked_editing(&uri(), position)),
            Ok(ranges((6, 5, 8), (6, 32, 35)))
        );
    }
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(5, 4))),
        Ok(ranges((5, 3, 6), (7, 4, 7)))
    );
    let original = worker(&project);
    let before = block_on(original.names_inspect()).unwrap();
    assert_eq!(before.productions, 1);
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(6, 33))),
        Ok(ranges((6, 5, 8), (6, 32, 35)))
    );
    assert_eq!(before, block_on(original.names_inspect()).unwrap());
    assert_eq!(original.counts(), (0, 5));
    assert!(project.workers.lock().is_empty());
    assert!(project.selected_workers.lock().is_empty());
}
#[test]
fn original_non_name_script_attribute_interpolation_single_and_frame_positions_are_empty() {
    let (_, project) = project(SOURCE);
    for position in [
        Position::new(6, 4),
        Position::new(6, 10),
        Position::new(6, 18),
        Position::new(6, 25),
        Position::new(1, 8),
        Position::new(0, 4),
        Position::new(8, 4),
        Position::new(4, 3),
    ] {
        assert_eq!(block_on(project.linked_editing(&uri(), position)), Ok(None));
    }
}
#[test]
fn original_utf8_crlf_non_bmp_and_authored_case_keep_exact_utf16_names() {
    let source =
        "<!--😀-->\r\n<template>\r\n<Card-é title=\"</Card-é>\">😀</Card-é>\r\n</template>";
    let (_, project) = project(source);
    for position in [Position::new(2, 2), Position::new(2, 31)] {
        assert_eq!(
            block_on(project.linked_editing(&uri(), position)),
            Ok(ranges((2, 1, 7), (2, 30, 36)))
        );
    }
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(2, 27))),
        Err(NavigationRefusal::Position)
    );
    let (_, project) = self::project("<template><Widget></ Widget ></template>");
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 12))),
        Ok(ranges((0, 11, 17), (0, 21, 27)))
    );
}
#[test]
fn original_missing_implicit_void_and_self_closing_do_not_guess_a_second_name() {
    for source in [
        "<template><p></template>",
        "<template><a><a>inner</a></a></template>",
        "<template><br><Child/></template>",
    ] {
        let (_, project) = project(source);
        assert_eq!(
            block_on(project.linked_editing(&uri(), Position::new(0, 11))),
            Ok(None)
        );
    }
}
#[test]
fn actual_recovery_descriptor_and_language_refusals_are_sticky_without_other_parsers() {
    let (state, project) = project("<template><p></p><div title='unterminated</template>");
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 11))),
        Err(NavigationRefusal::TemplateNamesProducer)
    );
    let original = worker(&project);
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 12))),
        Err(NavigationRefusal::TemplateNamesProducer)
    );
    assert!(Arc::ptr_eq(&original, &worker(&project)));
    assert_eq!(original.sfc_productions(), 1);
    state.documents.open(
        uri(),
        "<template src='./other.vue'/>".into(),
        2,
        "vue".into(),
    );
    project.notify_host_change(&uri());
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 1))),
        Err(NavigationRefusal::TemplateNamesProducer)
    );
    state
        .documents
        .open(uri(), "<p></p>".into(), 3, "html".into());
    project.notify_host_change(&uri());
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 1))),
        Err(NavigationRefusal::Language)
    );
}
#[test]
fn equal_foreign_source_and_revision_cannot_reuse_original_name_owner() {
    let (_, project) = project(SOURCE);
    let (local, _) = project.source.begin_query(&uri()).unwrap();
    let worker = project
        .worker_family(
            Arc::clone(local.snapshot()),
            super::super::QueryFamily::Names,
        )
        .unwrap();
    let foreign = crate::document::DocumentStore::new();
    foreign.open(uri(), SOURCE.into(), 1, "vue".into());
    let cache = crate::source_project::SourceSnapshotCache::default();
    let snapshot = cache.capture(&foreign, &uri()).unwrap();
    assert_eq!(snapshot.source(), local.snapshot().source());
    assert!(!worker.belongs_to(&snapshot));
    assert!(matches!(
        project.worker_family(snapshot, super::super::QueryFamily::Names),
        Err(NavigationRefusal::Host(
            crate::source_project::SnapshotRefusal::Superseded
        ))
    ));
}
#[test]
fn deep_original_elements_use_an_iterative_request_stack() {
    let source = vize_l0::cstr!(
        "<template>{}x{}</template>",
        "<p>".repeat(2048),
        "</p>".repeat(2048)
    );
    let (_, project) = project(&source);
    let start = 10 + 3 * 2047 + 1;
    let close = 10 + 3 * 2048 + 1 + 2;
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, start))),
        Ok(ranges((0, start, start + 1), (0, close, close + 1)))
    );
    assert_eq!(worker(&project).sfc_productions(), 1);
}

#[test]
fn original_case_insensitive_pair_cannot_forge_identical_standard_linked_text() {
    let (_, project) = project("<template><DIV></div></template>");
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 12))),
        Ok(None)
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 17))),
        Ok(None)
    );
    assert_eq!(worker(&project).sfc_productions(), 1);
}
