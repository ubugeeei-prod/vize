mod capacity;
mod lifecycle;

use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject, block_on,
};
use crate::server::ServerState;
use tower_lsp::lsp_types::{Location, Range, Url};

fn uri() -> Url {
    Url::parse("file:///selected.vue").unwrap()
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
            .selected_workers
            .lock()
            .get(&uri())
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    )
}
fn original_worker(
    project: &NativeNavigationProject<'_>,
) -> Arc<super::super::worker::NavigationWorker> {
    Arc::clone(
        project
            .workers
            .lock()
            .get(&uri())
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    )
}
fn position(source: &str, byte: usize) -> Position {
    let prefix = &source[..byte];
    Position::new(
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn occurrence(source: &str, needle: &str, index: usize) -> (Position, Location) {
    let byte = source.match_indices(needle).nth(index).unwrap().0;
    let start = position(source, byte);
    (
        start,
        Location::new(
            uri(),
            Range::new(start, position(source, byte + needle.len())),
        ),
    )
}

#[test]
fn genuine_selected_event_local_declarations_and_uses_retain_one_original_owner() {
    let source = "<template><button @click='let value=$event;{let value=2;value;}return value' @blur='let value=$event;value'/></template>";
    let (_, project) = project(source);
    let (outer, outer_decl) = occurrence(source, "value", 0);
    let (inner, inner_decl) = occurrence(source, "value", 1);
    let (inner_use, inner_location) = occurrence(source, "value", 2);
    let (outer_use, outer_location) = occurrence(source, "value", 3);
    let (blur, blur_decl) = occurrence(source, "value", 4);
    let (_, blur_location) = occurrence(source, "value", 5);
    assert_eq!(
        block_on(project.template_definition(&uri(), outer_use)),
        Ok(vec![outer_decl.clone()])
    );
    let original = worker(&project);
    let before = block_on(original.selected_inspect(outer)).unwrap();
    assert_eq!(before.productions, 1);
    assert!(before.handler_body.is_some());
    assert_eq!(
        block_on(project.template_references(&uri(), outer, true)),
        Ok(vec![outer_decl, outer_location])
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), inner_use)),
        Ok(vec![inner_decl.clone()])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), inner, true)),
        Ok(vec![inner_decl, inner_location])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), blur, true)),
        Ok(vec![blur_decl, blur_location])
    );
    assert_eq!(
        before,
        block_on(original.selected_inspect(outer_use)).unwrap()
    );
    assert!(Arc::ptr_eq(&original, &worker(&project)));
    assert!(project.workers.lock().is_empty());
    assert_eq!(original.sfc_productions(), 1);
}

#[test]
fn every_actual_var_declaration_site_is_returned_without_a_fabricated_preference() {
    let source =
        "<template><button @click='var value=$event;{var value=2;}return value'/></template>";
    let (_, project) = project(source);
    let (first, first_location) = occurrence(source, "value", 0);
    let (_, second_location) = occurrence(source, "value", 1);
    let (reference, reference_location) = occurrence(source, "value", 2);
    assert_eq!(
        block_on(project.template_definition(&uri(), reference)),
        Ok(vec![first_location.clone(), second_location.clone()])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), first, true)),
        Ok(vec![first_location, second_location, reference_location])
    );
}

#[test]
fn implicit_event_has_no_authored_declaration_and_only_its_actual_uses() {
    let source = "<template><button @click='$event;$event' @blur='$event'/></template>";
    let (_, project) = project(source);
    let (first, first_location) = occurrence(source, "$event", 0);
    let (_, second_location) = occurrence(source, "$event", 1);
    let (other, other_location) = occurrence(source, "$event", 2);
    assert_eq!(
        block_on(project.template_definition(&uri(), first)),
        Ok(vec![])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), first, true)),
        Ok(vec![first_location, second_location])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), other, true)),
        Ok(vec![other_location])
    );
}

#[test]
fn original_entity_crlf_non_bmp_and_half_open_header_positions_keep_authored_utf16() {
    let source =
        "<template><button @click='/*kept\r\n😀*/ let café=$event; return caf&#233;'/></template>";
    let (_, project) = project(source);
    let (declaration, declaration_location) = occurrence(source, "café", 0);
    let (reference, reference_location) = occurrence(source, "caf&#233;", 0);
    assert_eq!(
        block_on(project.template_definition(&uri(), reference)),
        Ok(vec![declaration_location.clone()])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), declaration, true)),
        Ok(vec![declaration_location, reference_location.clone()])
    );
    let entity_tail = position(source, source.find("&#233;").unwrap() + 2);
    assert_eq!(
        block_on(project.template_references(&uri(), entity_tail, false)),
        Ok(vec![reference_location.clone()])
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), reference_location.range.end)),
        Ok(vec![])
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(0, 12))),
        Ok(vec![])
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(1, 1))),
        Err(NavigationRefusal::Position)
    );
}

#[test]
fn actual_scripts_styles_and_encoded_for_refusals_are_sticky_without_fallback() {
    for source in [
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>",
        "<template><button @click='$event'/></template><style>button{color:red}</style>",
        "<template><div v-for='caf&#233; in it&#101;ms'/></template>",
    ] {
        let (_, project) = project(source);
        for _ in 0..2 {
            assert!(matches!(
                block_on(project.template_definition(&uri(), Position::new(0, 1))),
                Err(NavigationRefusal::SelectedSfcProducer(_))
            ));
        }
        assert_eq!(worker(&project).sfc_productions(), 1);
        assert!(project.workers.lock().is_empty());
    }
}

#[test]
fn generic_native_sfc_and_selected_requests_keep_distinct_live_original_families() {
    let source = "<template><button/></template>";
    let (_, project) = project(source);
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 1))),
        Ok(None)
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(0, 1))),
        Ok(vec![])
    );
    let selected = worker(&project);
    let original = original_worker(&project);
    let selected_file = block_on(selected.selected_inspect(Position::new(0, 1))).unwrap();
    let original_file = block_on(original.inspect(Position::new(0, 1))).unwrap();
    assert_ne!(selected_file.file, original_file.file);
    assert!(!Arc::ptr_eq(&selected, &original));
    assert_eq!(selected.sfc_productions(), 1);
    assert_eq!(original.sfc_productions(), 1);
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 1), true)),
        Ok(vec![])
    );
    assert_eq!(
        block_on(project.template_references(&uri(), Position::new(0, 1), true)),
        Ok(vec![])
    );
}
