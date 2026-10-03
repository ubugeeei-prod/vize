mod lifecycle;

use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject, block_on,
};
use crate::server::ServerState;
use tower_lsp::lsp_types::{Location, Range, Url};

fn uri() -> Url {
    Url::parse("file:///original.vue").unwrap()
}

fn new_project(source: &str) -> (Arc<ServerState>, NativeNavigationProject<'static>) {
    let state = Arc::new(ServerState::new());
    state.documents.open(uri(), source.into(), 1, "vue".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    (state, project)
}

fn worker(project: &NativeNavigationProject<'_>) -> Arc<super::super::worker::NavigationWorker> {
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

// The expected locations select authored spellings, independently of native
// resolution/decoder tables. In particular &fjlig; must retain all seven bytes.
fn position(source: &str, byte: usize) -> Position {
    let prefix = source.get(..byte).unwrap();
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
fn genuine_sfc_file_descriptor_programs_and_embeds_survive_repeated_requests() {
    let source = "\r\n<template><div :title=\"café\">{{café}}</div></template>\r\n<script setup lang=ts>/*😀*/ const café=1; café;</script>";
    let (_, project) = new_project(source);
    let (attribute, attribute_location) = occurrence(source, "café", 0);
    let (template, template_location) = occurrence(source, "café", 1);
    let (declaration, declaration_location) = occurrence(source, "café", 2);
    let (_, script_location) = occurrence(source, "café", 3);
    assert_eq!(
        block_on(project.definition(&uri(), attribute)),
        Ok(Some(declaration_location.clone()))
    );
    let owner = worker(&project);
    let before = block_on(owner.inspect(attribute)).unwrap();
    let sfc = before.sfc.as_ref().unwrap();
    assert_eq!(sfc.productions, 1);
    assert_eq!(sfc.programs.len(), 1);
    assert_eq!(sfc.programs[0].1, 2);
    assert_eq!(sfc.expressions.len(), 2);
    assert_eq!(before.parses, 0); // Not a fabricated one-Program parser count.
    assert_eq!(
        block_on(project.references(&uri(), declaration, true)),
        Ok(vec![
            attribute_location,
            template_location.clone(),
            declaration_location.clone(),
            script_location
        ])
    );
    assert_eq!(
        block_on(project.definition(&uri(), template)),
        Ok(Some(declaration_location))
    );
    let after = block_on(owner.inspect(template)).unwrap();
    assert_eq!(before, after);
    assert!(Arc::ptr_eq(&owner, &worker(&project)));
}

#[test]
fn exact_entity_and_escaped_identifier_projection_keeps_authored_ranges() {
    let source = "<script setup>const fj=1; const café=2;</script>\n<template><div :title=\"&fjlig;\">{{caf\\u00e9}}</div></template>";
    let (_, project) = new_project(source);
    let (entity, entity_location) = occurrence(source, "&fjlig;", 0);
    let (_, fj_declaration) = occurrence(source, "fj", 0);
    let (escaped, escaped_location) = occurrence(source, "caf\\u00e9", 0);
    let (_, cafe_declaration) = occurrence(source, "café", 0);
    assert_eq!(
        block_on(project.definition(&uri(), entity)),
        Ok(Some(fj_declaration.clone()))
    );
    // Selecting an authored byte inside the entity still selects the genuine
    // identifier occurrence; returning its location retains the complete entity.
    assert_eq!(
        block_on(project.references(
            &uri(),
            Position::new(entity.line, entity.character + 2),
            true
        )),
        Ok(vec![fj_declaration, entity_location])
    );
    assert_eq!(
        block_on(project.definition(&uri(), escaped)),
        Ok(Some(cafe_declaration))
    );
    assert_eq!(
        block_on(project.references(&uri(), escaped, false)),
        Ok(vec![escaped_location])
    );
}

#[test]
fn reversed_script_order_setup_shadow_and_local_import_aliases_use_real_binding_ids() {
    let source = "<script setup>import { run as local } from 'dep'; const value=2;</script>\n<script>const value=1;</script>\n<template>{{local(value)}}</template>";
    let (_, project) = new_project(source);
    let (setup, setup_location) = occurrence(source, "value", 0);
    let (ordinary, ordinary_location) = occurrence(source, "value", 1);
    let (template, template_location) = occurrence(source, "value", 2);
    let (_, alias_location) = occurrence(source, "local", 0);
    let (call, call_location) = occurrence(source, "local", 1);
    assert_eq!(
        block_on(project.definition(&uri(), template)),
        Ok(Some(setup_location.clone()))
    );
    assert_eq!(
        block_on(project.references(&uri(), setup, true)),
        Ok(vec![setup_location, template_location])
    );
    assert_eq!(
        block_on(project.references(&uri(), ordinary, true)),
        Ok(vec![ordinary_location])
    );
    assert_eq!(
        block_on(project.definition(&uri(), call)),
        Ok(Some(alias_location.clone()))
    );
    assert_eq!(
        block_on(project.references(&uri(), call, true)),
        Ok(vec![alias_location, call_location])
    );
    let retained = block_on(worker(&project).inspect(call)).unwrap();
    assert_eq!(retained.sfc.unwrap().programs.len(), 2);
}

#[test]
fn half_open_names_static_source_holes_and_opaque_styles_have_no_fabricated_references() {
    let source = "<script setup>const value=1;</script>\n<template>{{value}}</template>\n<style scoped>.value{color:red}</style>";
    let (_, project) = new_project(source);
    let (template, _) = occurrence(source, "value", 1);
    let (style, _) = occurrence(source, "value", 2);
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(template.line, template.character + 5))),
        Ok(None)
    );
    assert_eq!(block_on(project.definition(&uri(), style)), Ok(None));
    assert_eq!(
        block_on(project.references(&uri(), style, true)),
        Ok(Vec::new())
    );
    let (_, static_project) = new_project("<template><div>日本語</div></template>");
    assert_eq!(
        block_on(static_project.definition(&uri(), Position::new(0, 17))),
        Ok(None)
    );
}

#[test]
fn original_sfc_refusals_are_sticky_without_script_only_or_legacy_fallback() {
    for source in [
        "<script>const value=1;</script><template>{{value}}</template>",
        "<script setup lang=ts>const value:number=1;</script><template>{{value}}</template>",
        "<script setup>const value=/x/uv;</script><template>{{value}}</template>",
        "<script setup>const value=1;</script><template><div v-if=\"value\"/></template>",
        "<script setup src='./external.ts'></script><template></template>",
    ] {
        let (_, project) = new_project(source);
        let result = block_on(project.definition(&uri(), Position::new(0, 1)));
        assert!(
            matches!(&result, Err(NavigationRefusal::SfcProducer(issues)) if !issues.is_empty()),
            "{source}: {result:?}"
        );
        let original = worker(&project);
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(0, 1))),
            result
        );
        assert!(Arc::ptr_eq(&original, &worker(&project)));
        assert_eq!(original.sfc_productions(), 1);
    }
}
