use super::*;

fn uris() -> [Url; 4] {
    std::array::from_fn(|index| {
        Url::parse(&vize_l0::cstr!("file:///workspace/Idle{index}.vue")).unwrap()
    })
}

fn contexts<'a>(state: &'a ServerState, uris: &'a [Url; 4]) -> [IdeContext<'a>; 4] {
    std::array::from_fn(|index| IdeContext::testing(state, &uris[index], 0, SOURCE.into()))
}

#[test]
fn a_third_document_replaces_the_oldest_idle_publication() {
    let state = ServerState::new();
    let uris = uris();
    let contexts = contexts(&state, &uris);
    let budget = Arc::new(Budget::default());
    let owners = contexts
        .each_ref()
        .map(|ctx| local_owner(ctx, Arc::clone(&budget)));
    let first = publish(&owners[0], &contexts[0]);
    let retired = Arc::downgrade(&first);
    drop(first);
    drop(publish(&owners[1], &contexts[1]));
    let admitted = publish(&owners[2], &contexts[2]);
    assert!(admitted.lease.is_some());
    assert!(!owners[0].cell.retained());
    assert!(retired.upgrade().is_none());
    assert!(owners[1].cell.retained());
    assert!(owners[2].cell.retained());
    assert_eq!(budget.entries.load(Ordering::Acquire), 2);
    assert_eq!(
        packet(&contexts[2], Some("renamed"), &admitted),
        uncached::resolved_packet(&contexts[2], Some("renamed"))
    );
    drop(admitted);
    // Registration remains bounded while each later document can get a turn.
    for index in [3, 0, 1, 2, 3, 0] {
        let entry = publish(&owners[index], &contexts[index]);
        assert!(entry.lease.is_some());
        assert_eq!(budget.entries.load(Ordering::Acquire), 2);
        assert_eq!(
            budget.registry.lock().snapshot().iter().flatten().count(),
            2
        );
    }
}

#[test]
fn active_and_retired_readers_keep_the_cap_while_idle_entries_can_rotate() {
    let state = ServerState::new();
    let uris = uris();
    let contexts = contexts(&state, &uris);
    let budget = Arc::new(Budget::default());
    let owners = contexts
        .each_ref()
        .map(|ctx| local_owner(ctx, Arc::clone(&budget)));
    let active = publish(&owners[0], &contexts[0]);
    drop(publish(&owners[1], &contexts[1]));
    let newer = publish(&owners[2], &contexts[2]);
    assert!(newer.lease.is_some());
    assert!(owners[0].cell.retained(), "the oldest entry has a reader");
    assert!(!owners[1].cell.retained(), "skip to the oldest idle entry");
    let refused = publish(&owners[3], &contexts[3]);
    assert!(refused.lease.is_none());
    assert!(!owners[3].cell.retained());
    assert_eq!(
        packet(&contexts[3], Some("renamed"), &refused),
        uncached::resolved_packet(&contexts[3], Some("renamed"))
    );
    owners[0].cell.entry.lock().take();
    assert_eq!(budget.entries.load(Ordering::Acquire), 2);
    assert!(publish(&owners[3], &contexts[3]).lease.is_none());
    assert!(owners[2].cell.retained());
    drop(active);
    let admitted = publish(&owners[3], &contexts[3]);
    assert!(admitted.lease.is_some());
    assert_eq!(budget.entries.load(Ordering::Acquire), 2);
    drop(newer);
    drop(admitted);
    drop(owners);
    assert_eq!(budget.entries.load(Ordering::Acquire), 0);
    assert_eq!(budget.known_bytes.load(Ordering::Acquire), 0);
}

#[test]
fn stale_publication_identities_cannot_evict_a_replacement_or_reopened_cell() {
    let state = ServerState::new();
    let uris = uris();
    let contexts = contexts(&state, &uris);
    let budget = Arc::new(Budget::default());
    let owner = local_owner(&contexts[0], Arc::clone(&budget));
    drop(publish(&owner, &contexts[0]));
    let stale = budget.registry.lock().snapshot();
    owner.cell.entry.lock().take();
    let replacement = publish(&owner, &contexts[0]);
    assert!(!Registry::evict_oldest_idle(stale));
    assert!(Arc::ptr_eq(
        owner.cell.entry.lock().as_ref().unwrap(),
        &replacement
    ));
    let stale = budget.registry.lock().snapshot();
    drop(replacement);
    drop(owner);
    let reopened = local_owner(&contexts[0], Arc::clone(&budget));
    let current = publish(&reopened, &contexts[0]);
    assert!(!Registry::evict_oldest_idle(stale));
    assert!(Arc::ptr_eq(
        reopened.cell.entry.lock().as_ref().unwrap(),
        &current
    ));
    assert_eq!(budget.entries.load(Ordering::Acquire), 1);
    assert_eq!(
        budget.registry.lock().snapshot().iter().flatten().count(),
        1
    );
}

#[test]
fn concurrent_publications_preserve_reader_leases_and_bounded_registration() {
    let budget = Arc::new(Budget::default());
    let barrier = Arc::new(std::sync::Barrier::new(5));
    let admitted = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..4)
        .map(|index| {
            let budget = Arc::clone(&budget);
            let barrier = Arc::clone(&barrier);
            let admitted = Arc::clone(&admitted);
            std::thread::spawn(move || {
                let state = ServerState::new();
                let uri =
                    Url::parse(&vize_l0::cstr!("file:///workspace/Concurrent{index}.vue")).unwrap();
                let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
                let owner = local_owner(&ctx, Arc::clone(&budget));
                let entry = publish(&owner, &ctx);
                admitted.fetch_add(usize::from(entry.lease.is_some()), Ordering::AcqRel);
                barrier.wait();
                barrier.wait();
                drop(entry);
            })
        })
        .collect();
    barrier.wait();
    assert_eq!(admitted.load(Ordering::Acquire), 2);
    assert_eq!(budget.entries.load(Ordering::Acquire), 2);
    assert_eq!(
        budget.registry.lock().snapshot().iter().flatten().count(),
        2
    );
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(budget.entries.load(Ordering::Acquire), 0);
    assert_eq!(budget.known_bytes.load(Ordering::Acquire), 0);
}
