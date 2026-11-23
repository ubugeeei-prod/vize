use super::*;
use crate::source_project::SourceSnapshotCache;
use tower_lsp::lsp_types::Url;

#[derive(Default, Debug)]
struct Counts {
    sources: usize,
    responses: usize,
    symbols: usize,
    candidates: usize,
    symbol_bytes: usize,
    invalid: usize,
}

fn compare(state: &ServerState, uri: &Url, text: &str, name: &str, counts: &mut Counts) {
    state
        .documents
        .open(uri.clone(), text.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let original = ScriptQueryProducts::reference_symbols(state, uri);
    let specifiers = state
        .sfc_descriptor(uri, text)
        .map(|desc| ScriptQueryProducts::reference_specifiers(&desc));
    let actual = snapshot.script_query_products(state).unwrap();
    counts.sources += 1;
    assert_eq!(
        original.as_ref(),
        actual.as_ref().map(|out| &out.symbols),
        "{name}"
    );
    assert_eq!(
        specifiers.as_ref(),
        actual.as_ref().map(|out| &out.module_specifiers),
        "{name}"
    );
    if let (Some(original), Some(actual)) = (original, actual) {
        let bytes = serde_json::to_vec(&actual.symbols).unwrap();
        assert_eq!(serde_json::to_vec(&original).unwrap(), bytes, "{name}");
        counts.responses += 1;
        counts.symbols += actual.symbols.len();
        counts.candidates += actual.module_specifiers.len();
        counts.symbol_bytes += bytes.len();
        for specifier in actual.module_specifiers {
            assert!(
                text.get(specifier.span.start as usize..specifier.span.end as usize)
                    .is_some(),
                "{name} {specifier:?}"
            );
        }
    }
}

#[test]
fn declaration_profiles_and_recovered_imports_match_both_original_readers() {
    let state = ServerState::new();
    let uri = Url::parse("file:///projects/Published.vue").unwrap();
    let mut counts = Counts::default();
    for text in [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/ScriptEnd.vue.txt"
        )),
        "<!-- 雪🌱 -->\r\n<script lang='ts'>export type { Leaf } from './leaf'; export class Theme { get tone() { return 2; } }</script><script setup lang='ts'>import type { Leaf } from './leaf'; const { item: node } = {} as Leaf; const loader = () => import('./async')</script>",
        "<script setup lang='tsx'>import { A } from './A'; const el = <A/>;</script>",
        "<script lang='jsx'>export { A } from './A'; export const el = <A/>;</script>",
        "<script setup lang='ts'>import './retained'; const invalid: = ;</script>",
        "<script lang='ts'>const value = 1;\n<!-- HTML comment\nconst answer = value;</script>",
        "<script>const loaded = require('./one'); const other = import('./two');</script>",
        "<script></script><script setup>  </script><style>.a{}</style>",
        "<template><span/></template>",
        "<script>let a</script><script>let b</script>",
    ] {
        compare(&state, &uri, text, text, &mut counts);
    }
    assert!(
        counts.responses >= 8 && counts.symbols >= 8 && counts.candidates >= 7,
        "{counts:?}"
    );
}

#[test]
#[cfg(feature = "native")]
fn authored_candidates_identify_the_public_file_rename_edits() {
    use crate::ide::{FileRenameService, position_to_offset};
    use tower_lsp::lsp_types::{FileRename, RenameFilesParams};

    let root = tempfile::tempdir().unwrap();
    let old = root.path().join("Leaf.vue");
    let new = root.path().join("Renamed.vue");
    let app = root.path().join("App.vue");
    let uri = Url::from_file_path(&app).unwrap();
    let text = concat!(
        "<!-- 雪🌱 -->\r\n",
        "<script lang='ts'>export { default as Leaf } from './Leaf.vue';</script>\r\n",
        "<script setup lang='ts'>\r\n",
        "import Somewhere from './Other.vue';\r\n",
        "const title = '🌱'; const load = () => import('./Le\\u0061f.vue');\r\n",
        "type LeafModule = typeof import('./Leaf.vue');\r\n",
        "</script>"
    );
    std::fs::write(&app, "<template><div/></template>").unwrap();
    std::fs::write(&old, "<template><span/></template>").unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), text.into(), 1, "vue".into());

    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, &uri)
        .unwrap();
    let products = snapshot.script_query_products(&state).unwrap().unwrap();
    let original = futures::executor::block_on(FileRenameService::will_rename_files(
        &state,
        &RenameFilesParams {
            files: vec![FileRename {
                old_uri: Url::from_file_path(&old).unwrap().into(),
                new_uri: Url::from_file_path(&new).unwrap().into(),
            }],
        },
    ))
    .unwrap();
    let changes = original.changes.unwrap();
    assert_eq!(changes.len(), 1);
    let edits = changes.get(&uri).unwrap();
    let selected = products
        .module_specifiers
        .iter()
        .filter(|item| item.specifier == "./Leaf.vue")
        .map(|item| item.span)
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 3);
    assert_eq!(edits.len(), selected.len());
    for (edit, span) in edits.iter().zip(selected) {
        let offset = |p: tower_lsp::lsp_types::Position| {
            position_to_offset(text, p.line, p.character).unwrap()
        };
        assert_eq!(
            (offset(edit.range.start), offset(edit.range.end)),
            (span.start as usize, span.end as usize)
        );
        assert_eq!(edit.new_text, "./Renamed.vue");
    }
}

#[test]
fn jsx_host_suffix_matches_the_editor_dispatcher() {
    for first in ["j", "J", "t", "T"] {
        for second in ["s", "S"] {
            for third in ["x", "X"] {
                for prefix in ["", "/projects/雪🌱", "/projects/many.dots/App.vue"] {
                    let path = vize_l0::cstr!("{prefix}.{first}{second}{third}");
                    assert!(crate::utils::is_jsx_path(&path));
                    assert!(is_jsx_host(&path));
                }
            }
        }
    }
    for path in [
        "",
        ".",
        "x",
        "tsx",
        ".ts",
        "🌱x",
        ".tſx",
        ".jsx/",
        ".tsx.vue",
        "A.jsxx",
        ".jsX\u{301}",
    ] {
        assert_eq!(is_jsx_host(path), crate::utils::is_jsx_path(path), "{path}");
    }
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    let text = "<script setup>import './leaf';</script>";
    for name in [".JsX", "Leaf.tSX", "Leaf.tsx.vue"] {
        let uri = Url::parse(&vize_l0::cstr!("file:///projects/{name}")).unwrap();
        state
            .documents
            .open(uri.clone(), text.into(), 1, "vue".into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        let result = snapshot.script_query_products(&state);
        if crate::utils::is_jsx_path(uri.path()) {
            assert_eq!(result, Err(ScriptQueryRefusal::Language));
        } else {
            assert_eq!(
                result.unwrap().unwrap().module_specifiers[0].specifier,
                "./leaf"
            );
        }
    }
}

#[test]
fn closed_reopened_and_non_vue_hosts_cannot_publish_old_scripts() {
    let state = ServerState::new();
    let uri = Url::parse("file:///projects/Current.vue").unwrap();
    let cache = SourceSnapshotCache::default();
    let text = "<script setup>const original = 1</script>";
    state
        .documents
        .open(uri.clone(), text.into(), 3, "vue".into());
    let old = cache.capture(&state.documents, &uri).unwrap();
    state.documents.close(&uri);
    assert_eq!(
        old.script_query_products(&state),
        Err(ScriptQueryRefusal::Host(SnapshotRefusal::MissingDocument))
    );
    state
        .documents
        .open(uri.clone(), text.into(), 3, "vue".into());
    assert_eq!(
        old.script_query_products(&state),
        Err(ScriptQueryRefusal::Host(SnapshotRefusal::Superseded))
    );
    state
        .documents
        .open(uri.clone(), text.into(), 3, "typescript".into());
    assert_eq!(
        cache
            .capture(&state.documents, &uri)
            .unwrap()
            .script_query_products(&state),
        Err(ScriptQueryRefusal::Language)
    );
}

#[test]
#[ignore = "requires a NUL-delimited manifest of original Vue sources"]
fn original_edits_and_outlines() {
    let manifest = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let state = ServerState::new();
    let uri = Url::parse("file:///projects/Original.vue").unwrap();
    let mut counts = Counts::default();
    for entry in manifest.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        if let Ok(text) = std::str::from_utf8(&bytes) {
            compare(&state, &uri, text, name, &mut counts);
        } else {
            counts.invalid += 1;
        }
    }
    assert!(counts.responses > 0 && counts.symbols > 0 && counts.candidates > 0);
    eprintln!("authored_script_products {counts:?}");
}
