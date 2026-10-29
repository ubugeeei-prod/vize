#![expect(
    clippy::disallowed_types,
    reason = "concurrency fixtures share barriers and immutable protocol URLs"
)]

use std::sync::atomic::Ordering;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use dashmap::try_result::TryResult;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::DocumentStore;

fn uri(value: &str) -> Url {
    Url::parse(value).expect("fixture URI is valid")
}

fn replace(text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: String::from(text),
    }
}

fn names(snapshot: &Arc<[Url]>) -> Vec<&str> {
    snapshot.iter().map(Url::as_str).collect()
}

fn wait_until(message: &str, mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(Instant::now() < deadline, "{message}");
        std::thread::yield_now();
    }
}

fn wait_for_mutation(store: &DocumentStore, document: &Url, sources: usize) {
    wait_until("writer never reached the expected mutation state", || {
        store.mutations.load(Ordering::Acquire) >= sources
            && store.uri_snapshots.mutation_is_locked(document)
    });
}

fn independent_uri(store: &DocumentStore, guarded: &Url) -> Url {
    let guarded_stripe = store.uri_snapshots.stripe_index(guarded);
    for index in 0..10_000 {
        let value = vize_l0::cstr!("file:///workspace/independent-{index}.vue");
        let candidate = uri(value.as_str());
        if store.uri_snapshots.stripe_index(&candidate) != guarded_stripe
            && matches!(store.documents.try_get_mut(&candidate), TryResult::Absent)
        {
            return candidate;
        }
    }
    panic!("fixture finds independent URI and DashMap stripes");
}

#[test]
fn content_only_edits_reuse_the_exact_uri_snapshot() {
    let store = DocumentStore::new();
    let document = uri("file:///workspace/%E6%97%A5%E6%9C%AC%F0%9F%98%80.vue");
    store.open(
        document.clone(),
        String::from("before"),
        1,
        String::from("vue"),
    );
    let before = store.uris();

    assert!(store.apply_changes(&document, vec![replace("after")], 2));
    let after_change = store.uris();
    assert!(Arc::ptr_eq(&before, &after_change));
    drop(store.get_mut(&document).expect("document remains open"));
    assert!(Arc::ptr_eq(&before, &store.uris()));

    // didOpen is a source-lifetime boundary even when its key is unchanged.
    store.open(document, String::from("reopened"), 3, String::from("vue"));
    let after_reopen = store.uris();
    assert_eq!(names(&after_reopen), names(&before));
    assert!(!Arc::ptr_eq(&before, &after_reopen));
}

#[test]
fn key_set_mutations_publish_sorted_complete_snapshots_and_keep_stale_arcs() {
    let store = DocumentStore::new();
    let alpha = uri("file:///workspace/A.vue");
    let unicode = uri("file:///workspace/%E6%97%A5%E6%9C%AC.vue");
    let zeta = uri("file:///workspace/Z.vue");
    store.open(zeta.clone(), String::new(), 1, String::from("vue"));
    store.open(unicode.clone(), String::new(), 1, String::from("vue"));
    let stale = store.uris();
    assert_eq!(names(&stale), [unicode.as_str(), zeta.as_str()]);

    store.open(alpha.clone(), String::new(), 1, String::from("vue"));
    let opened = store.uris();
    assert!(!Arc::ptr_eq(&stale, &opened));
    assert_eq!(
        names(&opened),
        [unicode.as_str(), alpha.as_str(), zeta.as_str()]
    );

    store.close(&unicode);
    let closed = store.uris();
    assert!(!Arc::ptr_eq(&opened, &closed));
    assert_eq!(names(&closed), [alpha.as_str(), zeta.as_str()]);

    let renamed = uri("file:///workspace/%F0%9F%98%80.vue");
    assert!(store.rename(&zeta, renamed.clone()));
    let current = store.uris();
    assert!(!Arc::ptr_eq(&closed, &current));
    assert_eq!(names(&current), [renamed.as_str(), alpha.as_str()]);
    assert_eq!(names(&stale), [unicode.as_str(), zeta.as_str()]);
}

#[test]
fn failed_close_and_rename_do_not_publish_a_generation() {
    let store = DocumentStore::new();
    let first = uri("file:///workspace/first.vue");
    let occupied = uri("file:///workspace/occupied.vue");
    let missing = uri("file:///workspace/missing.vue");
    store.open(first.clone(), String::new(), 1, String::from("vue"));
    store.open(occupied.clone(), String::new(), 1, String::from("vue"));
    let seeded = store.uris();

    store.close(&missing);
    assert!(Arc::ptr_eq(&seeded, &store.uris()));
    assert!(!store.rename(&missing, uri("file:///workspace/new.vue")));
    assert!(Arc::ptr_eq(&seeded, &store.uris()));
    assert!(!store.rename(&first, occupied));
    assert!(Arc::ptr_eq(&seeded, &store.uris()));
}

#[test]
fn in_flight_mutation_returns_the_complete_prior_snapshot_without_lock_inversion() {
    let store = DocumentStore::new();
    let document = uri("file:///workspace/held.vue");
    store.open(document.clone(), String::new(), 1, String::from("vue"));
    let seeded = store.uris();

    let document_guard = store.get(&document).expect("document remains open");
    let mutation = store.uri_snapshots.mutation(&document, None);
    let during_mutation = store.uris();
    assert!(Arc::ptr_eq(&seeded, &during_mutation));
    drop(mutation);
    drop(document_guard);
}

#[test]
fn completed_nonconflicting_mutation_is_visible_while_another_writer_is_blocked() {
    let store = DocumentStore::new();
    let blocked = uri("file:///workspace/blocked.vue");
    store.open(blocked.clone(), String::new(), 1, String::from("vue"));
    let stale = store.uris();
    let blocked_guard = store.get(&blocked).expect("blocked document is open");
    let completed = independent_uri(&store, &blocked);

    let closing_store = store.clone();
    let closing_uri = blocked.clone();
    let closer = std::thread::spawn(move || closing_store.close(&closing_uri));
    wait_for_mutation(&store, &blocked, 1);

    store.open(
        completed.clone(),
        String::from("completed"),
        1,
        String::from("vue"),
    );
    assert_eq!(store.mutations.load(Ordering::Acquire), 1);
    let during_blocked_close = store.uris();
    assert!(!Arc::ptr_eq(&stale, &during_blocked_close));
    assert_eq!(
        names(&during_blocked_close),
        [blocked.as_str(), completed.as_str()]
    );
    assert_eq!(names(&stale), [blocked.as_str()]);

    drop(blocked_guard);
    closer.join().expect("blocked close completes");
    let current = store.uris();
    assert_eq!(names(&current), [completed.as_str()]);
}

#[test]
fn overlapping_conflicting_writers_commit_in_uri_stripe_order() {
    let store = DocumentStore::new();
    let document = uri("file:///workspace/conflicting.vue");
    store.open(
        document.clone(),
        String::from("before"),
        1,
        String::from("vue"),
    );
    let seeded = store.uris();
    let document_guard = store.get(&document).expect("document is open");

    let closing_store = store.clone();
    let closing_uri = document.clone();
    let closer = std::thread::spawn(move || closing_store.close(&closing_uri));
    wait_for_mutation(&store, &document, 1);

    let opening_store = store.clone();
    let opening_uri = document.clone();
    let opener = std::thread::spawn(move || {
        opening_store.open(
            opening_uri,
            String::from("reopened"),
            2,
            String::from("vue"),
        );
    });
    wait_until("second writer never reached the mutation state", || {
        store.mutations.load(Ordering::Acquire) >= 2
    });
    assert!(Arc::ptr_eq(&seeded, &store.uris()));

    drop(document_guard);
    closer.join().expect("first conflicting writer completes");
    opener.join().expect("second conflicting writer completes");
    let current = store.uris();
    assert!(!Arc::ptr_eq(&seeded, &current));
    assert_eq!(names(&current), [document.as_str()]);
    assert_eq!(store.text(&document).as_deref(), Some("reopened"));
}

#[test]
fn concurrent_renames_never_publish_a_remove_insert_intermediate() {
    const ROUNDS: usize = 2_000;
    let store = DocumentStore::new();
    let anchor = uri("file:///workspace/anchor.vue");
    let old = uri("file:///workspace/old.vue");
    let new = uri("file:///workspace/new.vue");
    store.open(anchor.clone(), String::new(), 1, String::from("vue"));
    store.open(old.clone(), String::new(), 1, String::from("vue"));
    let seeded = store.uris();
    assert_eq!(seeded.len(), 2);

    let barrier = Arc::new(Barrier::new(3));
    let writer_store = store.clone();
    let writer_barrier = Arc::clone(&barrier);
    let writer_old = old.clone();
    let writer_new = new.clone();
    let writer = std::thread::spawn(move || {
        writer_barrier.wait();
        for _ in 0..ROUNDS {
            assert!(writer_store.rename(&writer_old, writer_new.clone()));
            assert!(writer_store.rename(&writer_new, writer_old.clone()));
        }
    });

    let reader_store = store.clone();
    let reader_barrier = Arc::clone(&barrier);
    let reader_anchor = anchor.clone();
    let reader_old = old.clone();
    let reader_new = new.clone();
    let reader = std::thread::spawn(move || {
        reader_barrier.wait();
        for _ in 0..ROUNDS * 2 {
            let snapshot = reader_store.uris();
            assert_eq!(snapshot.len(), 2);
            assert_eq!(snapshot[0], reader_anchor);
            let expected_renamed = if snapshot[1] == reader_old {
                &reader_old
            } else {
                &reader_new
            };
            assert_eq!(&snapshot[1], expected_renamed);
        }
    });

    barrier.wait();
    writer.join().expect("writer completes");
    reader.join().expect("reader completes");
    let final_snapshot = store.uris();
    assert_eq!(names(&final_snapshot), [anchor.as_str(), old.as_str()]);
}
