//! Measure the original editor request through guarded publication on both sides.
use super::{SourceSnapshotCache, TemplateReport, whole_file_report};
use crate::{runtime::block_on, server::ServerState};
use davinci_harness::{alloc, paired::assert_paired_time};
use std::hint::black_box;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

fn publish(
    state: &ServerState,
    cache: &SourceSnapshotCache,
    uri: &Url,
    resident: bool,
) -> TemplateReport {
    let (query, _) = cache.begin_query(&state.documents, uri).unwrap();
    block_on(query.run(&state.documents, |snapshot| async move {
        if resident {
            snapshot.template_report(state).unwrap().unwrap()
        } else {
            whole_file_report(snapshot.source()).unwrap()
        }
    }))
    .unwrap()
    .publish(&state.documents, |answer| answer)
    .unwrap()
}

#[test]
#[ignore = "paired open, edit, query and publication on separate original document stores"]
fn complete_editor_requests() {
    alloc::mark_installed();
    let uri = Url::parse("file:///editor/Measure.vue").unwrap();
    for (name, body) in [
        ("one", "<p v-if='a && b'>雪</p>".to_owned()),
        ("wide", "<p v-if='a && b'>雪</p>".repeat(64)),
        ("damaged", "<p :x=>雪</p>".to_owned()),
    ] {
        let initial = format!("<template>{body}</template>");
        let moved = format!("<!--🧭-->\r\n{initial}");
        let edited = initial
            .replace("a && b", "b || ready")
            .replace(":x=>", ":x='ok'>");
        for (mode, next) in [
            ("open", &initial),
            ("stable", &initial),
            ("prefix", &moved),
            ("body", &edited),
        ] {
            let hosts = [ServerState::new(), ServerState::new()];
            let caches = [
                SourceSnapshotCache::default(),
                SourceSnapshotCache::default(),
            ];
            let mut versions = [1_i32; 2];
            for host in &hosts {
                host.documents
                    .open(uri.clone(), initial.clone(), 1, "vue".into());
            }
            let mut request = |resident: bool| {
                let side = usize::from(resident);
                let host = &hosts[side];
                let version = &mut versions[side];
                *version += 1;
                if mode == "open" {
                    host.documents
                        .open(uri.clone(), initial.clone(), *version, "vue".into());
                } else if mode != "stable" {
                    let source = if *version % 2 == 0 { next } else { &initial };
                    assert!(host.documents.apply_changes(
                        &uri,
                        vec![TextDocumentContentChangeEvent {
                            range: None,
                            range_length: None,
                            text: source.clone(),
                        }],
                        *version
                    ));
                }
                publish(host, &caches[side], &uri, resident)
            };
            for _ in 0..4 {
                assert_eq!(request(false), request(true));
            }
            let old = alloc::measure(|| request(false)).unwrap();
            let new = alloc::measure(|| request(true)).unwrap();
            assert!(old.calls > 0 && old.peak_bytes_over_start > 0);
            eprintln!(
                "editor_template {name} {mode} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start
            );
            assert_paired_time(&format!("editor_template {name} {mode}"), 16, |resident| {
                drop(black_box(request(resident)));
            });
        }
    }
}
