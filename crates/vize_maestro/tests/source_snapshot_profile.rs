//! Direct document snapshot producer controls; no checker, backend or filesystem.
#![expect(clippy::expect_used, reason = "protocol fixture failures panic")]
#![expect(
    clippy::disallowed_types,
    reason = "document snapshots own protocol strings"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};
use vize_maestro::document::DocumentStore;

#[cfg(feature = "micro-profiling")]
#[global_allocator]
static ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =
    vize_l0::profiler::ProfilingAllocator::from_allocator(std::alloc::System);

type Snapshot = (Arc<[Url]>, Vec<(Url, String)>);

#[derive(Clone, Copy)]
struct Expected {
    items: u64,
    uri_bytes: u64,
    vue_items: u64,
    vue_uri_bytes: u64,
    text_bytes: u64,
}

fn capture(store: &DocumentStore, workers: usize, expected: Expected) -> Vec<Snapshot> {
    #[cfg(feature = "micro-profiling")]
    let profiler = {
        let profiler = vize_l0::profiler::global_profiler();
        profiler.clear();
        profiler.enable();
        profiler
    };
    let results = std::thread::scope(|scope| {
        let handles = (0..workers)
            .map(|_| scope.spawn(move || (store.uris(), store.vue_texts())))
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|worker| worker.join().expect("snapshot worker completes"))
            .collect::<Vec<_>>()
    });
    #[cfg(feature = "micro-profiling")]
    {
        profiler.disable();
        let entries = vize_l0::profiler::micro::snapshot();
        let count = workers as u64;
        for (name, units) in [
            ("lsp.document_store.uris.calls", count),
            ("lsp.document_store.uris.clone_snapshot", 0),
            (
                "lsp.document_store.uris.returned_items",
                count * expected.items,
            ),
            (
                "lsp.document_store.uris.returned_uri_utf8_bytes",
                count * expected.uri_bytes,
            ),
            ("lsp.document_store.vue_texts.calls", count),
            ("lsp.document_store.vue_texts.clone_snapshot", 0),
            (
                "lsp.document_store.vue_texts.returned_items",
                count * expected.vue_items,
            ),
            (
                "lsp.document_store.vue_texts.returned_uri_utf8_bytes",
                count * expected.vue_uri_bytes,
            ),
            (
                "lsp.document_store.vue_texts.returned_text_utf8_bytes",
                count * expected.text_bytes,
            ),
        ] {
            let matching = entries
                .iter()
                .filter(|entry| entry.name == name)
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{name} has one source site");
            assert_eq!(
                (matching[0].calls, matching[0].units),
                (count, units),
                "{name}"
            );
        }
        println!(
            "document_store_micro={}",
            serde_json::to_string(&entries).expect("counts serialize")
        );
    }
    for (uris, texts) in &results {
        assert_eq!(uris.len() as u64, expected.items);
        assert_eq!(
            uris.iter()
                .map(|uri| uri.as_str().len() as u64)
                .sum::<u64>(),
            expected.uri_bytes
        );
        assert_eq!(texts.len() as u64, expected.vue_items);
        assert_eq!(
            texts
                .iter()
                .map(|(uri, _)| uri.as_str().len() as u64)
                .sum::<u64>(),
            expected.vue_uri_bytes
        );
        assert_eq!(
            texts.iter().map(|(_, text)| text.len() as u64).sum::<u64>(),
            expected.text_bytes
        );
    }
    results
}

fn uri(value: &str) -> Url {
    Url::parse(value).expect("fixture URI is valid")
}

#[cfg(feature = "micro-profiling")]
fn assert_shared_text_snapshot_allocation_law() {
    use vize_l0::profiler::{
        allocation_snapshot, reset_allocation_counters, set_allocation_tracking_enabled,
    };

    const READS: u64 = 128;
    let segment = "<template>\r\n<div title=\"日本😀\">value</div>\r\n</template>\r\n";
    let source = segment.repeat(512 * 1024 / segment.len() + 1);
    let payload_bytes = source.len() as u64;
    let store = DocumentStore::new();
    let document = uri("file:///fixture/Large.vue");
    store.open(document.clone(), source, 1, String::from("vue"));

    // Seed the revision's OnceLock outside both measured windows.
    let resident = store
        .text_snapshot(&document)
        .expect("open document has a shared snapshot");

    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    for _ in 0..READS {
        let owned = store.text(&document).expect("open document has owned text");
        std::hint::black_box(&owned);
    }
    let owned = allocation_snapshot();
    set_allocation_tracking_enabled(false);

    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    for _ in 0..READS {
        let shared = store
            .text_snapshot(&document)
            .expect("open document has a shared snapshot");
        assert!(Arc::ptr_eq(&resident, &shared));
        std::hint::black_box(&shared);
    }
    let shared = allocation_snapshot();
    set_allocation_tracking_enabled(false);

    assert!(owned.allocation_calls() >= READS);
    assert!(owned.requested_bytes() >= READS.saturating_mul(payload_bytes));
    assert_eq!(shared.allocation_calls(), 0);
    assert_eq!(shared.requested_bytes(), 0);
    assert_eq!(shared.allocation_failures(), 0);
    println!(
        "document_text_snapshot_allocation={}",
        serde_json::json!({
            "reads": READS,
            "payloadBytes": payload_bytes,
            "owned": {
                "allocationCalls": owned.allocation_calls(),
                "requestedBytes": owned.requested_bytes(),
            },
            "shared": {
                "allocationCalls": shared.allocation_calls(),
                "requestedBytes": shared.requested_bytes(),
            },
            "law": "warm shared Arc clones allocate zero bytes",
        })
    );
}

#[cfg(feature = "micro-profiling")]
fn assert_shared_uri_snapshot_allocation_law() {
    use vize_l0::profiler::{
        allocation_snapshot, reset_allocation_counters, set_allocation_tracking_enabled,
    };

    const DOCUMENTS: usize = 2_500;
    const READS: u64 = 338;
    let store = DocumentStore::new();
    for index in 0..DOCUMENTS {
        let path = vize_l0::cstr!("file:///fixture/uri-snapshot/{index:04}-%E6%97%A5.vue");
        store.open(
            Url::parse(path.as_str()).expect("generated URI is valid"),
            String::new(),
            1,
            String::from("vue"),
        );
    }

    // Build and register the cache and counter sites outside the measured window.
    let resident = store.uris();
    assert_eq!(resident.len(), DOCUMENTS);

    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    for _ in 0..READS {
        let owned = resident.iter().cloned().collect::<Vec<Url>>();
        assert_eq!(owned.len(), DOCUMENTS);
        assert_eq!(owned.as_slice(), resident.as_ref());
        std::hint::black_box(&owned);
    }
    let legacy_owned = allocation_snapshot();
    set_allocation_tracking_enabled(false);

    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    for _ in 0..READS {
        let shared = store.uris();
        assert!(Arc::ptr_eq(&resident, &shared));
        std::hint::black_box(&shared);
    }
    let shared = allocation_snapshot();
    set_allocation_tracking_enabled(false);

    assert!(legacy_owned.allocation_calls() > 0);
    assert!(legacy_owned.requested_bytes() > 0);
    assert_eq!(legacy_owned.allocation_failures(), 0);
    assert_eq!(shared.allocation_calls(), 0);
    assert_eq!(shared.requested_bytes(), 0);
    assert_eq!(shared.allocation_failures(), 0);
    println!(
        "document_uri_snapshot_allocation={}",
        serde_json::json!({
            "documents": DOCUMENTS,
            "reads": READS,
            "legacyOwned": {
                "allocationCalls": legacy_owned.allocation_calls(),
                "requestedBytes": legacy_owned.requested_bytes(),
            },
            "shared": {
                "allocationCalls": shared.allocation_calls(),
                "requestedBytes": shared.requested_bytes(),
            },
            "law": "warm URI Arc clones allocate zero bytes after the same-process owned baseline",
        })
    );
}

#[test]
fn snapshot_producers_keep_owned_results_and_exact_utf8_payloads() {
    let store = DocumentStore::new();
    let empty = capture(
        &store,
        1,
        Expected {
            items: 0,
            uri_bytes: 0,
            vue_items: 0,
            vue_uri_bytes: 0,
            text_bytes: 0,
        },
    );
    assert_eq!(empty.len(), 1);
    assert!(empty[0].0.is_empty());
    assert!(empty[0].1.is_empty());
    for (path, text, language) in [
        ("file:///fixture/A.vue", "abc\r\n", "vue"),
        ("file:///fixture/Empty.vue", "", "vue"),
        (
            "file:///fixture/%E6%97%A5%E6%9C%AC.vue",
            "日本😀\r\n",
            "vue",
        ),
        ("file:///fixture/B.ts", "const x = 1", "typescript"),
        ("file:///fixture/C.Vue", "notvue", "vue"),
    ] {
        store.open(uri(path), String::from(text), 1, String::from(language));
    }
    // 125 serialized URI bytes; Vue has 84 URI bytes and 17 UTF8 text bytes.
    let original = capture(
        &store,
        4,
        Expected {
            items: 5,
            uri_bytes: 125,
            vue_items: 3,
            vue_uri_bytes: 84,
            text_bytes: 17,
        },
    );
    for (uris, texts) in &original {
        let mut names = uris.iter().map(Url::as_str).collect::<Vec<_>>();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "file:///fixture/%E6%97%A5%E6%9C%AC.vue",
                "file:///fixture/A.vue",
                "file:///fixture/B.ts",
                "file:///fixture/C.Vue",
                "file:///fixture/Empty.vue",
            ]
        );
        assert_eq!(
            texts
                .iter()
                .map(|(uri, text)| (uri.as_str(), text.as_str()))
                .collect::<Vec<_>>(),
            [
                ("file:///fixture/%E6%97%A5%E6%9C%AC.vue", "日本😀\r\n"),
                ("file:///fixture/A.vue", "abc\r\n"),
                ("file:///fixture/Empty.vue", ""),
            ]
        );
    }

    assert!(store.apply_changes(
        &uri("file:///fixture/A.vue"),
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: String::from("A")
        }],
        2
    ));
    assert!(store.rename(
        &uri("file:///fixture/%E6%97%A5%E6%9C%AC.vue"),
        uri("file:///fixture/Renamed.txt")
    ));
    store.close(&uri("file:///fixture/B.ts"));
    let changed = capture(
        &store,
        1,
        Expected {
            items: 4,
            uri_bytes: 94,
            vue_items: 2,
            vue_uri_bytes: 46,
            text_bytes: 1,
        },
    );
    assert_eq!(
        changed[0]
            .1
            .iter()
            .map(|(uri, text)| (uri.as_str(), text.as_str()))
            .collect::<Vec<_>>(),
        [
            ("file:///fixture/A.vue", "A"),
            ("file:///fixture/Empty.vue", ""),
        ]
    );
    // The earlier returned strings remain owned after edits and renames.
    assert_eq!(original[0].1[0].1, "日本😀\r\n");
    assert_eq!(original[0].1[1].1, "abc\r\n");

    #[cfg(feature = "micro-profiling")]
    {
        assert_shared_text_snapshot_allocation_law();
        assert_shared_uri_snapshot_allocation_law();
    }
}
