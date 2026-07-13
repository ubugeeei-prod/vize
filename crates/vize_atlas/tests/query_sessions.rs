use std::{
    sync::{
        Barrier,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

use vize_atlas::{
    Compilation, Product, ProductStatus, Provider, ProviderContext, ProviderError, Shared,
};
use vize_carton::cstr;

struct WordCount;

impl Product for WordCount {
    type Value = usize;
    const NAME: &'static str = "test.query-session-word-count";
}

struct WordCountProvider(Shared<AtomicUsize>);

impl Provider for WordCountProvider {
    type Product = WordCount;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<usize, ProviderError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(context.source().text().split_whitespace().count())
    }
}

fn compilation(calls: &Shared<AtomicUsize>) -> Compilation {
    let mut compilation = Compilation::new();
    compilation
        .register_provider(WordCountProvider(Shared::clone(calls)))
        .unwrap();
    compilation
}

#[test]
fn sessions_share_cache_while_compilation_and_forks_stay_isolated() {
    let calls = Shared::new(AtomicUsize::new(0));
    let mut compilation = compilation(&calls);
    let source = compilation.add_source("words.txt", "one two").unwrap();
    let snapshot = compilation.snapshot();
    let mut left_fork = snapshot.fork();
    let mut right_fork = snapshot.fork();
    let first_session = snapshot.query_session();
    let second_session = snapshot.query_session();

    let (
        (first_status, first_executions, first_hits),
        (second_status, second_executions, second_hits),
    ) = thread::scope(|scope| {
        let (ready_tx, ready_rx) = mpsc::channel();
        let first = scope.spawn(move || {
            let mut session = first_session;
            let status = session.query::<WordCount>(source).unwrap().status();
            ready_tx.send(()).unwrap();
            let counters = session.counters().for_product::<WordCount>();
            (status, counters.executions(), counters.cache_hits())
        });
        let second = scope.spawn(move || {
            let mut session = second_session;
            ready_rx.recv().unwrap();
            let status = session.query::<WordCount>(source).unwrap().status();
            let counters = session.counters().for_product::<WordCount>();
            (status, counters.executions(), counters.cache_hits())
        });
        (first.join().unwrap(), second.join().unwrap())
    });

    assert_eq!(first_status, ProductStatus::Executed);
    assert_eq!((first_executions, first_hits), (1, 0));
    assert_eq!(second_status, ProductStatus::CacheHit);
    assert_eq!((second_executions, second_hits), (0, 1));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(snapshot.cache().contains::<WordCount>(source));
    assert!(compilation.cache().is_empty());
    assert!(left_fork.cache().is_empty());
    assert!(right_fork.cache().is_empty());

    compilation.update_source(source, "one two three").unwrap();
    assert_eq!(*compilation.query::<WordCount>(source).unwrap().value(), 3);
    let mut later_session = snapshot.query_session();
    let cached = later_session.query::<WordCount>(source).unwrap();
    assert_eq!(*cached.value(), 2);
    assert_eq!(cached.status(), ProductStatus::CacheHit);

    left_fork
        .update_source(source, "one two three four")
        .unwrap();
    assert_eq!(*left_fork.query::<WordCount>(source).unwrap().value(), 4);
    let right = right_fork.query::<WordCount>(source).unwrap();
    assert_eq!(*right.value(), 2);
    assert_eq!(right.status(), ProductStatus::Executed);
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    assert_eq!(snapshot.source(source).unwrap().text(), "one two");
}

#[test]
fn parallel_sessions_preserve_all_shared_cache_entries() {
    const WORKERS: usize = 12;
    let calls = Shared::new(AtomicUsize::new(0));
    let mut compilation = compilation(&calls);
    let sources: Vec<_> = (0..WORKERS)
        .map(|index| {
            compilation
                .add_source(cstr!("worker-{index}.txt"), "parallel words")
                .unwrap()
        })
        .collect();
    let snapshot = compilation.snapshot();
    let barrier = Shared::new(Barrier::new(WORKERS));

    thread::scope(|scope| {
        let handles: Vec<_> = sources
            .iter()
            .copied()
            .map(|source| {
                let barrier = Shared::clone(&barrier);
                let mut session = snapshot.query_session();
                scope.spawn(move || {
                    barrier.wait();
                    session.query::<WordCount>(source).unwrap()
                })
            })
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap().status(), ProductStatus::Executed);
        }
    });

    assert_eq!(calls.load(Ordering::SeqCst), WORKERS);
    assert_eq!(snapshot.cache().len(), WORKERS);
    let mut verifier = snapshot.query_session();
    for source in sources {
        assert_eq!(
            verifier.query::<WordCount>(source).unwrap().status(),
            ProductStatus::CacheHit
        );
    }
    let counters = verifier.counters().for_product::<WordCount>();
    assert_eq!(counters.executions(), 0);
    assert_eq!(counters.cache_hits(), WORKERS as u64);
}
