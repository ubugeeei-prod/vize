use tower_lsp::lsp_types::{Location, Position, Range, Url};
use vize_canon::{ImportSourceMap, LspLocation, LspPosition, LspRange, virtual_ts::VizeMapping};

use super::{CanonicalVirtualDocument, map_complete_locations};
use crate::ide::{IdeContext, diagnostics::VirtualTsResult};
use crate::server::ServerState;

const SOURCE: &str = "/*😀*/ export const café = 1;\r\nexport const again = café;\r\n";

fn document(uri: &Url) -> CanonicalVirtualDocument {
    CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: "file:///private/session/query.ts".into(),
        virtual_result: VirtualTsResult {
            code: SOURCE.into(),
            source_mappings: vec![VizeMapping {
                src_range: 0..SOURCE.len(),
                gen_range: 0..SOURCE.len(),
                sub_spans: Vec::new(),
            }],
            semantic_links: Vec::new(),
            import_source_map: ImportSourceMap::empty(),
        },
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: vec!["/private/session".into()],
        source_catalogs: Vec::new(),
    }
}

fn raw(uri: &str, line: u32, character: u32) -> LspLocation {
    LspLocation {
        uri: uri.into(),
        range: LspRange {
            start: LspPosition { line, character },
            end: LspPosition {
                line,
                character: character + 4,
            },
        },
    }
}

fn authored(uri: &Url, line: u32, character: u32) -> Location {
    Location {
        uri: uri.clone(),
        range: Range::new(
            Position::new(line, character),
            Position::new(line, character + 4),
        ),
    }
}

#[test]
fn complete_mapping_keeps_utf16_crlf_order_and_duplicate_occurrences()
-> Result<(), Box<dyn std::error::Error>> {
    let uri = Url::parse("file:///authored/source.ts")?;
    let state = ServerState::new();
    let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
    let document = document(&uri);
    let locations = [
        raw(&document.request_uri, 1, 21),
        raw(&document.request_uri, 0, 20),
        raw(&document.request_uri, 1, 21),
    ];
    assert_eq!(
        map_complete_locations(&ctx, &document, &locations),
        Some(vec![
            authored(&uri, 1, 21),
            authored(&uri, 0, 20),
            authored(&uri, 1, 21),
        ])
    );
    Ok(())
}

#[test]
fn one_private_unmapped_member_refuses_the_complete_packet()
-> Result<(), Box<dyn std::error::Error>> {
    let uri = Url::parse("file:///authored/source.ts")?;
    let state = ServerState::new();
    let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
    let document = document(&uri);
    let good = raw(&document.request_uri, 1, 21);
    let unknown = raw("file:///private/session/unmapped.ts", 0, 0);
    // No retained authored mapping exists for this private session member.
    for locations in [vec![good.clone(), unknown.clone()], vec![unknown, good]] {
        assert_eq!(map_complete_locations(&ctx, &document, &locations), None);
    }
    assert_eq!(map_complete_locations(&ctx, &document, &[]), Some(vec![]));
    Ok(())
}
