//! The last-close callback never races a published open or waits on mutation locks.
use super::*;
use std::{sync::mpsc, time::Duration};
use tower_lsp::lsp_types::Url;
const BOUND: Duration = Duration::from_secs(10);

#[test]
fn open_or_uncommitted_sources_cannot_enter_the_callback() {
    let store = DocumentStore::new();
    let uri = Url::parse("file:///closed/App.vue").unwrap();
    store.open(uri.clone(), "<template />".into(), 1, "vue".into());
    assert_eq!(store.try_while_closed(|| 9), None);
    store.close(&uri);
    assert_eq!(store.try_while_closed(|| 9), Some(9));
    let guard = store.uri_snapshots.mutation(&uri, None);
    assert_eq!(
        store.try_while_closed(|| -> usize { panic!("busy callback ran") }),
        None
    );
    drop(guard);
    let mutation = store.source_set_mutation();
    assert_eq!(
        store.try_while_closed(|| -> usize { panic!("unstable callback ran") }),
        None
    );
    drop(mutation);
    assert_eq!(store.try_while_closed(|| 10), Some(10));
}

#[test]
fn an_open_started_inside_the_closed_boundary_publishes_only_after_it_returns() {
    let store = DocumentStore::new();
    let uri = Url::parse("file:///closed/Reopened.vue").unwrap();
    let writer_store = store.clone();
    let writer_uri = uri.clone();
    let (start, begin) = mpsc::channel();
    let (attempted, attempt) = mpsc::channel();
    let (finished, done) = mpsc::channel();
    let writer = std::thread::spawn(move || {
        begin.recv_timeout(BOUND).unwrap();
        attempted.send(()).unwrap();
        writer_store.open(
            writer_uri,
            "<template>new</template>".into(),
            1,
            "vue".into(),
        );
        finished.send(()).unwrap();
    });
    assert_eq!(
        store.try_while_closed(|| {
            start.send(()).unwrap();
            attempt.recv_timeout(BOUND).unwrap();
            assert!(store.uris().is_empty());
            assert!(done.try_recv().is_err());
            7
        }),
        Some(7)
    );
    done.recv_timeout(BOUND).unwrap();
    writer.join().unwrap();
    assert_eq!(store.version(&uri), Some(1));
    assert_eq!(store.uris().as_ref(), &[uri]);
    assert_eq!(
        store.try_while_closed(|| -> usize { panic!("reopened callback ran") }),
        None
    );
}
