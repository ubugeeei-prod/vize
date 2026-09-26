//! Actual native cross-root locations must retain authored coordinates.

use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{DocumentChanges, Location, Range, Url, WorkspaceEdit};
use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaScriptVirtualDocumentRequest};

use super::super::{
    CanonicalVirtualDocument, canonical_source_offset_to_position, map_canonical_corsa_location,
    map_canonical_corsa_workspace_edit,
};
use crate::ide::{IdeContext, diagnostics::VirtualTsResult};
use crate::server::ServerState;

fn native_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let binary = std::env::var_os("CORSA_PATH")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| vize_l0::corsa_resolver::discover_corsa_in_ancestors(workspace));
    assert!(
        binary.is_some() || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
        "VIZE_TEST_REQUIRE_TSGO requires the native catalog provider oracle"
    );
    binary
}

async fn open(
    bridge: &CorsaBridge,
    path: &Path,
    source: &str,
    overlays: &[(PathBuf, &str)],
) -> CanonicalVirtualDocument {
    let uri = Url::from_file_path(path).unwrap();
    let project = bridge
        .open_script_virtual_project(CorsaScriptVirtualDocumentRequest {
            source_path: path,
            request_path: uri.as_str(),
            code: source,
            source_type: oxc_span::SourceType::from_path(path).unwrap(),
            options: vize_canon::CorsaVueVirtualDocumentOptions {
                jsx_typecheck: true,
                ..Default::default()
            },
            overlays,
            virtual_ts_options: &Default::default(),
        })
        .await
        .unwrap();
    let host = project.document;
    CanonicalVirtualDocument {
        source_uri: uri,
        request_uri: host.request_uri,
        virtual_result: VirtualTsResult::from_projection(
            host.code.to_string(),
            vize_canon::virtual_ts::ProjectionMapping::from_spans(host.mappings),
            host.import_source_map,
        ),
        dependencies: Vec::new(),
        // Exercise the catalog fallback for every foreign native identity.
        materialized_sources: Vec::new(),
        session_project_roots: project.session_project_root.into_iter().collect(),
        source_catalogs: vec![project.source_catalog],
    }
}

fn site(path: &Path, source: &str) -> Location {
    let offset = source.rfind("label").unwrap();
    let (line, character) = crate::ide::offset_to_position(source, offset);
    Location::new(
        Url::from_file_path(path).unwrap(),
        Range::new(
            tower_lsp::lsp_types::Position::new(line, character),
            tower_lsp::lsp_types::Position::new(line, character + 5),
        ),
    )
}

fn sorted(mut sites: Vec<Location>) -> Vec<Location> {
    sites.sort_by(|a, b| {
        (&a.uri, a.range.start.line, a.range.start.character).cmp(&(
            &b.uri,
            b.range.start.line,
            b.range.start.character,
        ))
    });
    sites
}

fn rename_sites(edit: WorkspaceEdit) -> Vec<Location> {
    let mut sites = Vec::new();
    if let Some(changes) = edit.changes {
        for (uri, edits) in changes {
            sites.extend(edits.into_iter().map(|edit| {
                assert_eq!(edit.new_text, "title");
                Location::new(uri.clone(), edit.range)
            }));
        }
    }
    if let Some(changes) = edit.document_changes {
        let DocumentChanges::Edits(edits) = changes else {
            panic!("unexpected resource operation")
        };
        for document in edits {
            for edit in document.edits {
                let tower_lsp::lsp_types::OneOf::Left(edit) = edit else {
                    panic!("unexpected annotated edit")
                };
                assert_eq!(edit.new_text, "title");
                sites.push(Location::new(
                    document.text_document.uri.clone(),
                    edit.range,
                ));
            }
        }
    }
    sorted(sites)
}

#[test]
fn native_catalog_maps_independent_tsx_jsx_roots_and_retains_the_query_epoch() {
    let Some(binary) = native_binary() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    std::fs::write(root.join("tsconfig.json"),
        "{\"compilerOptions\":{\"allowJs\":true,\"checkJs\":true,\"jsx\":\"preserve\",\"module\":\"ESNext\",\"moduleResolution\":\"bundler\"},\"include\":[\"**/*\"]}").unwrap();
    let a = root.join("A.tsx");
    let b = root.join("B.jsx");
    let model = root.join("model.ts");
    let a_source = "import { model } from './model'; const mark = 'é'; export const A = () => <div>{model.label}</div>;";
    let b_source = "import { model } from './model'; const mark = '日本語'; export const B = () => <span>{model.label}</span>;";
    let model_source = "export const model = { label: 'model' };";
    let state = ServerState::new();
    state.set_workspace_root(root.clone());
    for (path, source) in [(&a, a_source), (&b, b_source), (&model, model_source)] {
        std::fs::write(path, source).unwrap();
        state.documents.open(
            Url::from_file_path(path).unwrap(),
            source.into(),
            1,
            "typescript".into(),
        );
    }
    let uri = Url::from_file_path(&a).unwrap();
    let ctx = IdeContext::testing(
        &state,
        &uri,
        a_source.rfind("label").unwrap(),
        a_source.into(),
    );
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(binary),
        working_dir: Some(root),
        ..Default::default()
    });
    crate::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let overlays = [
            (a.clone(), a_source),
            (b.clone(), b_source),
            (model.clone(), model_source),
        ];
        let first = open(&bridge, &a, a_source, &overlays).await;
        let sibling = open(&bridge, &b, b_source, &overlays).await;
        // Another root can reload the native project handle. Production
        // providers reopen their query host before asking for coordinates.
        let document = open(&bridge, &a, a_source, &overlays).await;
        assert!(first.source_catalogs[0].shares_revision_with(&document.source_catalogs[0]));
        assert!(document.source_catalogs[0].shares_revision_with(&sibling.source_catalogs[0]));
        let (line, character) = canonical_source_offset_to_position(&document, ctx.offset).unwrap();
        let generated =
            crate::ide::position_to_offset(&document.virtual_result.code, line, character).unwrap();
        assert_eq!(
            document.virtual_result.code.get(generated..generated + 5),
            Some("label")
        );
        let raw = bridge
            .references(&document.request_uri, line, character, true)
            .await
            .unwrap();
        let mapped = raw
            .iter()
            .map(|location| map_canonical_corsa_location(&ctx, &document, location).unwrap())
            .collect();
        let expected = sorted(vec![
            site(&a, a_source),
            site(&b, b_source),
            site(&model, model_source),
        ]);
        assert_eq!(sorted(mapped), expected);
        let raw_edit = bridge
            .rename(&document.request_uri, line, character, "title")
            .await
            .unwrap()
            .unwrap();
        let edit: WorkspaceEdit = serde_json::from_value(raw_edit).unwrap();
        assert_eq!(
            rename_sites(map_canonical_corsa_workspace_edit(&ctx, &document, edit).unwrap()),
            expected
        );
        let edited = b_source.replace("日本語", "a longer 🦀 prefix");
        let overlays = [
            (a.clone(), a_source),
            (b.clone(), edited.as_str()),
            (model.clone(), model_source),
        ];
        let current = open(&bridge, &a, a_source, &overlays).await;
        assert!(!document.source_catalogs[0].shares_revision_with(&current.source_catalogs[0]));
        let overlays = [(a.clone(), a_source), (model.clone(), model_source)];
        state.documents.close(&Url::from_file_path(&b).unwrap());
        let closed = open(&bridge, &a, a_source, &overlays).await;
        let sibling_path = Url::parse(&sibling.request_uri)
            .unwrap()
            .to_file_path()
            .unwrap();
        assert!(closed.source_catalogs[0].get(&sibling_path).is_none());
        let mapped = raw
            .iter()
            .map(|location| map_canonical_corsa_location(&ctx, &document, location).unwrap())
            .collect();
        assert_eq!(
            sorted(mapped),
            expected,
            "an old response retains its exact source coordinates after edit and close"
        );
        bridge.shutdown().await.unwrap();
    });
}
