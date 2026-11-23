//! Complete typed-hover responses for replay against the stock stdio server.
use super::{tests::options, *};
use crate::source_project::SourceSnapshotCache;
use serde_json::{Value, json};
use tower_lsp::lsp_types::Url;
use vize_l0::line_index::LineIndex;
use vize_l1::container::Vue;

#[test]
#[ignore = "writes actual typed hovers for the stdio comparison"]
fn compare_original_binding_hovers_on_the_wire() {
    let mut cases = Vec::new();
    for (setup, role) in [("", ScriptRole::Ordinary), (" setup", ScriptRole::Setup)] {
        let source = format!(
            "<script{setup} lang=ts>\r\nconst count: number=4; const crab: string='🦀'; let 雪: number=false;\r\nlet message: string=false; var flag: boolean=true; const ref: number=1;\r\nexport let outgoing: string='x';\r\nvar repeated: number=1; var repeated: string='later';\r\n/** Documentation from the project. */ const documented: number=1;\r\n</script>"
        );
        let docs = DocumentStore::new();
        let uri = Url::parse("file:///src/App.vue").unwrap();
        docs.open(uri.clone(), source.clone(), 1, "vue".into());
        let cache = SourceSnapshotCache::default();
        let snap = cache.capture(&docs, &uri).unwrap();
        let arena = Allocator::new();
        let owner = Vue.observe_descriptor(&arena, snap.source(), options());
        let index = LineIndex::new(&source);
        let mut requests = Vec::<Value>::new();
        for (marker, project) in [
            ("count:", false),
            ("crab:", false),
            ("雪:", false),
            ("message:", false),
            ("flag:", false),
            ("ref:", false),
            ("outgoing:", role == ScriptRole::Ordinary),
            ("repeated: number", true),
            ("repeated: string", true),
            ("documented:", true),
        ] {
            let byte = source.find(marker).unwrap();
            let pos = index.position(byte);
            let position = Position::new(pos.line, pos.character);
            let hover = snap
                .prepare_literal_products(&docs, &owner, role)
                .unwrap()
                .publish(&docs, Some(position))
                .unwrap()
                .binding_hover;
            assert_eq!(hover.is_none(), project, "{marker}");
            requests.push(json!({"position":position,"hover":hover,"project":project}));
        }
        cases.push(json!({"source":source,"requests":requests}));
    }
    let output = std::env::var_os("VIZE_LITERAL_HOVERS").unwrap();
    std::fs::write(output, serde_json::to_vec(&cases).unwrap()).unwrap();
}
