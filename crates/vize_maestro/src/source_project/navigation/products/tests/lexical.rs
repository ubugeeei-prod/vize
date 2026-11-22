//! Compare every public local symbol packet with independently parsed OXC scopes.
use super::*;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};
use tower_lsp::lsp_types::Range;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::{
    file::{DeclarationKind, InitializerKind},
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};

fn range(source: &str, span: Span) -> Range {
    Range::new(
        corpus::position(source, span.start),
        corpus::position(source, span.end),
    )
}

fn original_has_arrows(source: &str, ty: SourceType) -> bool {
    let arena = Allocator::new();
    let parsed = Parser::new(&arena, source, ty).parse_observed();
    let Some(program) = parsed.admitted() else {
        return false;
    };
    let input =
        ProgramInput::checked(program, SourceRoot::new(source).unwrap().whole_block(), 0).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            input.with_lexical_arrow_initializers(),
            ProgramScope::Module,
        )
        .unwrap();
    let Ok(file) = producer.finish() else {
        return false;
    };
    file.is_complete()
        && file.bindings().filter_map(|b| b.declaration()).any(|d| {
            d.initializer == InitializerKind::Function
                && matches!(
                    d.kind,
                    DeclarationKind::Const | DeclarationKind::Var | DeclarationKind::Let
                )
        })
}

fn compare_script(
    project: &NativeNavigationProject<'_>,
    uri: &Url,
    name: &str,
    source: &str,
    ty: SourceType,
) -> usize {
    let arena = Allocator::new();
    let parsed = Parser::new(&arena, source, ty).parse();
    assert!(parsed.diagnostics.is_empty());
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty());
    let semantic = built.semantic;
    let mut positions = 0;
    for id in semantic.scoping().symbol_ids() {
        let decl = range(source, semantic.scoping().symbol_span(id));
        let mut sites = vec![DocumentHighlight {
            range: decl,
            kind: Some(DocumentHighlightKind::TEXT),
        }];
        for reference in semantic.symbol_references(id) {
            sites.push(DocumentHighlight {
                range: range(source, semantic.reference_span(reference)),
                kind: Some(if reference.is_write() {
                    DocumentHighlightKind::WRITE
                } else {
                    DocumentHighlightKind::READ
                }),
            });
        }
        sites.sort_by_key(|site| (site.range.start, site.range.end));
        for include in [true, false] {
            let expected = SymbolNavigationProducts {
                definition: Some(Location::new(uri.clone(), decl)),
                references: sites
                    .iter()
                    .filter(|site| include || site.range != decl)
                    .map(|site| Location::new(uri.clone(), site.range))
                    .collect(),
                highlights: sites.clone(),
            };
            for site in &sites {
                let actual =
                    block_on(project.symbol_products(uri, site.range.start, include)).unwrap();
                assert_eq!(actual, expected, "{name} {:?}", site.range);
                assert_eq!(
                    separate(project, uri, site.range.start, include).unwrap(),
                    expected,
                    "{name}"
                );
                positions += 1;
            }
        }
    }
    positions
}

#[test]
fn opted_in_workers_resolve_closures_and_drop_superseded_document_scopes() {
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///lexical/mod.ts").unwrap();
    let project = NativeNavigationProject::for_lexical_initializers(
        SourceQueryProject::new_server(Arc::clone(&state)),
    );
    let usual = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    let source = "const 外=4;\r\nconst id=(外:number)=>外;\r\nconst read=()=>外;";
    state
        .documents
        .open(uri.clone(), source.into(), 1, "typescript".into());
    let first = corpus::position(source, source.rfind("外").unwrap() as u32);
    assert!(block_on(usual.symbol_products(&uri, first, true)).is_err());
    assert_eq!(
        compare_script(
            &project,
            &uri,
            "first",
            source,
            SourceType::ts().with_module(true)
        ),
        12
    );
    let changed = "const 外=2; const read=(外:number)=>外;";
    state
        .documents
        .open(uri.clone(), changed.into(), 2, "typescript".into());
    project.notify_host_change(&uri);
    assert_eq!(
        compare_script(
            &project,
            &uri,
            "edited",
            changed,
            SourceType::ts().with_module(true)
        ),
        8
    );
    let vue = Url::parse("file:///lexical/Main.vue").unwrap();
    let sfc = "<script setup>const value=3; const save=()=>value;</script>\r\n<template><button @click='save'/></template>";
    state
        .documents
        .open(vue.clone(), sfc.into(), 1, "vue".into());
    let pos = corpus::position(sfc, sfc.rfind("save").unwrap() as u32);
    let response = block_on(project.symbol_products(&vue, pos, true)).unwrap();
    assert_eq!(response, separate(&project, &vue, pos, true).unwrap());
    let start = sfc.find("save").unwrap() as u32;
    assert_eq!(
        response.definition,
        Some(Location::new(
            vue.clone(),
            range(sfc, Span::new(start, start + 4))
        ))
    );
    assert_eq!(response.references.len(), 2);
    assert_eq!(response.highlights.len(), 2);
}

#[test]
#[ignore = "every public local navigation response on admitted original JS and TS modules"]
fn original_lexical_script_navigation() {
    let list =
        std::fs::read_to_string(std::env::var_os("VIZE_LEXICAL_MODULE_MANIFEST").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let state = Arc::new(ServerState::new());
    let project = NativeNavigationProject::for_lexical_initializers(
        SourceQueryProject::new_server(Arc::clone(&state)),
    );
    let uri = Url::parse("file:///lexical/module.ts").unwrap();
    let (mut admitted, mut requests) = (0i32, 0);
    #[cfg(feature = "micro-profiling")]
    let mut failures = Vec::new();
    #[cfg(feature = "micro-profiling")]
    davinci_harness::alloc::mark_installed();
    for name in list.split_terminator('\0') {
        let bytes = std::fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            continue;
        };
        if !source.contains("=>") {
            continue;
        }
        let ts = name.ends_with(".ts") || name.ends_with(".mts");
        let ty = if ts {
            SourceType::ts()
        } else {
            SourceType::mjs()
        }
        .with_module(true);
        if !original_has_arrows(source, ty) {
            continue;
        }
        admitted += 1;
        state.documents.open(
            uri.clone(),
            source.into(),
            admitted,
            if ts { "typescript" } else { "javascript" }.into(),
        );
        project.notify_host_change(&uri);
        requests += compare_script(&project, &uri, name, source, ty);
        #[cfg(feature = "micro-profiling")]
        {
            let arena = Allocator::new();
            let parsed = Parser::new(&arena, source, ty).parse();
            let semantic = SemanticBuilder::new()
                .with_build_nodes(true)
                .build(&parsed.program)
                .semantic;
            let scopes = semantic.scoping();
            let symbol = scopes
                .symbol_ids()
                .find(|id| !scopes.get_resolved_reference_ids(*id).is_empty())
                .or_else(|| scopes.symbol_ids().next())
                .unwrap();
            profile::measure(
                &project,
                &uri,
                corpus::position(source, scopes.symbol_span(symbol).start),
                name,
                &mut failures,
            );
        }
        state.documents.close(&uri);
        project.notify_host_change(&uri);
    }
    #[cfg(feature = "micro-profiling")]
    assert!(failures.is_empty(), "{failures:?}");
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_ARROWS").is_some() {
        assert!(admitted >= 6 && requests > 0)
    }
    eprintln!("lexical_navigation original_files={admitted} request_positions={requests}");
}
