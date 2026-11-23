//! Exercise the same original cursor locations through cold open and didChange.
use super::{
    tests::{options, replace},
    *,
};
use crate::source_project::SourceSnapshotCache;
use serde_json::json;
use tower_lsp::lsp_types::Url;
use vize_l0::line_index::LineIndex;
use vize_l1::container::Vue;

#[test]
#[ignore = "writes cold and warm typed hovers for the stdio comparison"]
fn compare_damaged_binding_hovers_on_the_wire() {
    let mut cases = Vec::new();
    for (setup, role) in [("", ScriptRole::Ordinary), (" setup", ScriptRole::Setup)] {
        let original = format!(
            "<script{setup} lang=ts>\r\nconst crab: string='🦀'; let 雪: number=false; var ok: boolean=true;\r\nconst ref: number=2; var reused: number=1; var reused: string='x';\r\nlet unfinished: number = 3;\r\n</script>"
        );
        let damaged = format!("<!--新-->\r\n{}", original.replace("= 3;", "= ;"));
        for warm in [false, true] {
            let docs = DocumentStore::new();
            let uri = Url::parse("file:///src/App.vue").unwrap();
            let first = if warm { &original } else { &damaged };
            docs.open(uri.clone(), first.clone(), 1, "vue".into());
            let cache = SourceSnapshotCache::default();
            let initial = cache.capture(&docs, &uri).unwrap();
            let arena = Allocator::new();
            let before = Vue.observe_descriptor(&arena, initial.source(), options());
            let saved = initial.open_literal_prefix(&docs, &before, role).unwrap();
            if warm {
                replace(&docs, &uri, None, &damaged, 2);
            }
            let next = cache.capture(&docs, &uri).unwrap();
            let owner = Vue.observe_descriptor(&arena, next.source(), options());
            let local = saved.recover(&docs, &next, &owner).unwrap();
            let index = LineIndex::new(&damaged);
            let mut requests = Vec::new();
            for (marker, project) in [
                ("crab:", false),
                ("雪:", false),
                ("ok:", false),
                ("ref:", false),
                ("reused: number", true),
                ("reused: string", true),
                ("unfinished:", true),
            ] {
                let at = index.position(damaged.find(marker).unwrap());
                let position = Position::new(at.line, at.character);
                let answer = local.publish(&docs, position).unwrap();
                assert_eq!(answer.hover.is_none(), project, "{marker}");
                assert_eq!(answer.version, if warm { 2 } else { 1 });
                assert_eq!(answer.assignment.is_some(), marker != "unfinished:");
                requests.push(json!({"position":position,"hover":answer.hover,"project":project}));
            }
            let mut case = json!({"source":damaged,"requests":requests});
            if warm {
                case["previous"] = json!(original);
            }
            cases.push(case);
        }
    }
    let output = std::env::var_os("VIZE_RECOVERED_HOVERS").unwrap();
    std::fs::write(output, serde_json::to_vec(&cases).unwrap()).unwrap();
}
