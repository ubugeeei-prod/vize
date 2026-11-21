#![expect(
    clippy::unwrap_used,
    reason = "test inputs and open document revisions are asserted"
)]
use super::*;
use crate::source_project::SourceSnapshotCache;
use std::path::Path;
use tower_lsp::lsp_types::Url;
use vize_canon::batch::{project_declaration_as_typescript, reexports_vue_template_ref};

fn separate(source: &str) -> DeclarationToolProducts {
    let checker = DeclarationCheckerProducts {
        typescript: project_declaration_as_typescript(source),
        reexports_vue_template_ref: reexports_vue_template_ref(source),
    };
    let component_tags = if source.len() <= 4 * 1024 * 1024 {
        super::super::declared_names(source)
    } else {
        Vec::new()
    };
    DeclarationToolProducts {
        checker,
        component_tags,
    }
}

const COMBINED: &str = "interface Widgets { '雪-card': unknown; Base: any }\ndeclare module 'vue' { interface GlobalComponents extends Widgets { Local: any } }\nexport { useTemplateRef } from 'vue';\nexport default class<T> { value: T }";

fn directed() -> [&'static str; 13] {
    [
        COMBINED,
        "interface A extends B { A: unknown } interface B extends A { B: unknown } declare module '@vue/runtime-core' { interface GlobalComponents extends A { A: any; [key]: any; 'ui-slot': any } }",
        "declare module 'elsewhere' { interface GlobalComponents { Other: any } } export type { useTemplateRef } from 'vue';",
        r"declare module '\u0076ue' { interface Gl\u006fbalComponents { '\u96ea': any } }",
        "export * from '../node_modules/vue'; export default function<T>(p:T): T;",
        "export { useTemplateRef as other } from 'vue'; export { useTemplateRef } from 'not-vue';",
        "export type { type T } from 'types'; export { useTemplateRef } from 'vue';",
        "export { useTemplateRef } from 'vue'; declare module 'vue' { interface GlobalComponents {",
        "const x: string; export enum Mode { A }; export namespace API { type T=1 }",
        "/* 雪 */\r\nexport default class { a(): void }",
        "export * as v from 'vue'; export type * from 'vue'",
        "export/*no literal gate*/* from 'vue'",
        "",
    ]
}

fn production_inputs(mut visit: impl FnMut(&str, &str)) {
    let Ok(inputs) = std::env::var("VIZE_DECLARATION_INPUTS") else {
        return;
    };
    for entry in std::fs::read(inputs)
        .unwrap()
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
    {
        let name = std::str::from_utf8(entry).unwrap();
        let path = Path::new(name);
        assert!(DeclarationCheckerProducts::supports_path(path));
        let source = std::fs::read_to_string(path).unwrap();
        visit(name, &source);
    }
}

#[test]
fn providers_projection_and_completion_preserve_their_own_recovery_contracts() {
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    for suffix in ["d.ts", "d.mts", "d.cts"] {
        let uri = Url::parse(&format!("file:///types/components.{suffix}")).unwrap();
        for source in directed() {
            state
                .documents
                .open(uri.clone(), source.into(), 1, "typescript".into());
            let snapshot = cache.capture(&state.documents, &uri).unwrap();
            assert_eq!(
                snapshot.declaration_tool_products(&state).unwrap(),
                separate(source),
                "{suffix} {source}"
            );
        }
    }
    let result = separate(COMBINED);
    assert!(result.checker.reexports_vue_template_ref);
    assert!(result.checker.typescript.contains("declare class"));
    assert_eq!(
        result.component_tags,
        ["Base", "Local", "雪-card"].map(String::from)
    );
    assert_eq!(separate(directed()[3]).component_tags, [String::from("雪")]);
}

#[test]
fn source_contribution_matches_workspace_completion_for_unsaved_declarations() {
    let root = tempfile::tempdir().unwrap();
    let declaration = root.path().join("types.d.mts");
    let uri = Url::from_file_path(&declaration).unwrap();
    std::fs::write(&declaration, "export {}").unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let cache = SourceSnapshotCache::default();
    for source in [
        COMBINED.into(),
        format!("/*{}*/{COMBINED}", " ".repeat(4 * 1024 * 1024)),
    ] {
        state
            .documents
            .open(uri.clone(), source, 1, "typescript".into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        let result = snapshot.declaration_tool_products(&state).unwrap();
        assert_eq!(
            result.component_tags,
            crate::runtime::block_on(state.global_component_tag_names())
        );
        assert_eq!(result, separate(snapshot.source()));
    }
}

#[test]
fn encoded_files_and_metadata_use_the_url_librarys_path() {
    for (raw, supported) in [
        ("file:///repo/types%2Ed%2Ets", true),
        ("file:///repo/%E9%9B%AA.d.mts", true),
        ("file:///repo/%2Egit/types.d.ts", false),
        ("file:///repo/entry%2Ets", false),
        ("https://example.test/repo/types.d.ts", false),
    ] {
        let uri = Url::parse(raw).unwrap();
        assert_eq!(supported_declaration(&uri), supported, "{raw}");
    }
}

#[test]
fn host_refuses_closed_reopened_and_non_declaration_paths() {
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    for raw in [
        "file:///repo/component.ts",
        "file:///repo/.git/x.d.ts",
        "untitled:globals.d.ts",
        "file:///repo/components.d.ts",
    ] {
        let uri = Url::parse(raw).unwrap();
        state
            .documents
            .open(uri.clone(), COMBINED.into(), 4, "typescript".into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        if raw != "file:///repo/components.d.ts" {
            assert_eq!(
                snapshot.declaration_tool_products(&state),
                Err(DeclarationToolRefusal::Path)
            );
        }
        state.documents.close(&uri);
        assert_eq!(
            snapshot.declaration_tool_products(&state),
            Err(DeclarationToolRefusal::Host(
                SnapshotRefusal::MissingDocument
            ))
        );
        state
            .documents
            .open(uri.clone(), "export {};".into(), 4, "typescript".into());
        assert_eq!(
            snapshot.declaration_tool_products(&state),
            Err(DeclarationToolRefusal::Host(SnapshotRefusal::Superseded))
        );
    }
}

#[test]
#[ignore = "compare complete original declaration products on public file inputs"]
fn original_declaration_products_match() {
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///types/project.d.ts").unwrap();
    let (mut files, mut authored, mut projected, mut changed, mut providers, mut tags) =
        (0, 0, 0, 0, 0, 0);
    production_inputs(|name, source| {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "typescript".into());
        let result = cache
            .capture(&state.documents, &uri)
            .unwrap()
            .declaration_tool_products(&state)
            .unwrap();
        assert_eq!(result, separate(source), "{name}");
        files += 1;
        authored += source.len();
        projected += result.checker.typescript.len();
        changed += usize::from(result.checker.typescript != source);
        providers += usize::from(result.checker.reexports_vue_template_ref);
        tags += result.component_tags.len();
    });
    assert!(files > 0);
    eprintln!(
        "declarations files={files} authored={authored} projected={projected} changed={changed} providers={providers} tags={tags}"
    );
}

#[cfg(feature = "micro-profiling")]
mod profile;
