use super::*;
use crate::{server::ServerState, source_project::SourceQueryProject};
use futures::executor::block_on;
use std::sync::Arc;

fn separate(
    project: &NativeNavigationProject<'_>,
    uri: &Url,
    pos: Position,
    include: bool,
) -> Result<SymbolNavigationProducts, NavigationRefusal> {
    Ok(SymbolNavigationProducts {
        definition: block_on(project.definition(uri, pos))?,
        references: block_on(project.references(uri, pos, include))?,
        highlights: block_on(project.highlights(uri, pos))?,
    })
}

fn offsets(source: &str) -> impl Iterator<Item = Position> + '_ {
    let mut pos = Position::new(0, 0);
    source.chars().map(move |ch| {
        let here = pos;
        if ch == '\n' {
            pos.line += 1;
            pos.character = 0;
        } else {
            pos.character += ch.len_utf16() as u32;
        }
        here
    })
}

#[test]
fn public_navigation_corresponds_at_every_position_in_programs_and_original_sfc_files() {
    let state = Arc::new(ServerState::new());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    let mut results = 0;
    for (name, lang, source) in [
        (
            "Main.vue",
            "vue",
            "<script setup>/*🌱*/let value=1;value++;value;</script>\r\n<template><p :title='value'>{{value}}</p></template>",
        ),
        (
            "Main.vue",
            "vue",
            "<script setup lang='ts'>const fj=1;const café=2;</script>\n<template><p :title='&fjlig;'>{{caf\\u00e9}}</p></template>",
        ),
        (
            "mod.ts",
            "typescript",
            "type A=number;\nexport const v:A=2; function f(v:number){return v;}\nf(v);",
        ),
        (
            "mod.js",
            "javascript",
            "let x = 4;\r\nx += 1;export { x as other };",
        ),
        (
            "Widget.tsx",
            "typescriptreact",
            "const A=()=>null; const view=<A/>; view;",
        ),
        (
            "Rejected.vue",
            "vue",
            "<template><Unknown/></template><script setup>missing()</script>",
        ),
    ] {
        let uri = Url::parse(&vize_l0::cstr!("file:///navigation/{name}")).unwrap();
        state
            .documents
            .open(uri.clone(), source.into(), 1, lang.into());
        project.notify_host_change(&uri);
        for pos in offsets(source).chain([Position::new(9999, 0)]) {
            for include in [false, true] {
                let expected = separate(&project, &uri, pos, include);
                let actual = block_on(project.symbol_products(&uri, pos, include));
                assert_eq!(actual, expected, "{name} {pos:?} {include}");
                results += usize::from(actual.is_ok_and(|p| p.definition.is_some()));
            }
        }
        state.documents.close(&uri);
        project.notify_host_change(&uri);
        assert_eq!(
            block_on(project.symbol_products(&uri, Position::new(0, 0), false)),
            Err(NavigationRefusal::Host(
                crate::source_project::SnapshotRefusal::MissingDocument
            ))
        );
    }
    assert!(results > 20);
}

#[test]
fn ready_products_cannot_be_published_after_a_configuration_or_revision_change() {
    let state = Arc::new(ServerState::new());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    let uri = Url::parse("file:///navigation/Old.vue").unwrap();
    let source = "<script setup>const value=1</script><template>{{value}}</template>";
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let pos = Position::new(0, source.find("value").unwrap() as u32);
    for configuration in [true, false] {
        let (query, _) = project.source.begin_query(&uri).unwrap();
        let owner = &project;
        let ready = block_on(query.run(|snap| async move {
            let worker = owner.worker(snap)?;
            Ok((worker.profile(), worker.symbol_sites(pos).await))
        }))
        .unwrap();
        if configuration {
            state.set_dialect_config(Some(vize_l0::config::VueDialect::PetiteVue));
            let actual = ready
                .publish(|result| project.checked_response(result))
                .unwrap();
            assert_eq!(actual, Err(NavigationRefusal::ConfigurationChanged));
            state.set_dialect_config(None);
        } else {
            state
                .documents
                .open(uri.clone(), "<template/>".into(), 1, "vue".into());
            assert_eq!(
                ready.publish(|result| project.checked_response(result)),
                Err(crate::source_project::SnapshotRefusal::Superseded)
            );
        }
    }
}

mod corpus;
#[cfg(feature = "micro-profiling")]
mod profile;

#[cfg(feature = "experimental-lexical-arrow-queries")]
mod lexical;
