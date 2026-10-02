use super::*;

#[test]
fn close_reopen_with_same_editor_version_cancels_old_work_and_retains_new_identity() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///reopen.ts");
    project.open(uri.clone(), "same".into(), 1, "typescript".into());
    let (old, old_cancel) = project.begin_query(&uri).unwrap();
    let original = Arc::clone(old.snapshot());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &uri, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    project.close(&uri);
    assert!(old_cancel.is_aborted());
    assert!(matches!(
        poll(work.as_mut(), &wake),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    project.open(uri.clone(), "same".into(), 1, "typescript".into());
    let (new, _) = project.begin_query(&uri).unwrap();
    assert_eq!(new.snapshot().version(), original.version());
    assert_eq!(new.snapshot().source(), original.source());
    assert_ne!(new.snapshot().revision(), original.revision());
    assert!(!Arc::ptr_eq(new.snapshot(), &original));
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn rename_cancels_old_uri_work_and_keeps_authored_source_at_real_new_uri() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let old = uri("file:///old.ts");
    let new = uri("file:///new.ts");
    project.open(old.clone(), "authored".into(), 8, "typescript".into());
    let (query, cancel) = project.begin_query(&old).unwrap();
    let original = Arc::clone(query.snapshot());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &old, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(project.rename(&old, new.clone()));
    assert!(cancel.is_aborted());
    assert!(matches!(
        poll(work.as_mut(), &wake),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    assert!(!documents.contains(&old));
    let (renamed, _) = project.begin_query(&new).unwrap();
    assert_eq!(renamed.snapshot().uri(), &new);
    assert_eq!(renamed.snapshot().source(), original.source());
    assert_ne!(renamed.snapshot().revision(), original.revision());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn rejected_rename_collision_and_same_uri_rename_preserve_live_query() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let old = uri("file:///occupied-source.ts");
    let occupied = uri("file:///occupied-target.ts");
    project.open(old.clone(), "first".into(), 1, "typescript".into());
    project.open(occupied.clone(), "second".into(), 1, "typescript".into());
    let (query, cancel) = project.begin_query(&old).unwrap();
    let revision = query.snapshot().revision();
    assert!(!project.rename(&old, occupied.clone()));
    assert!(project.rename(&old, old.clone()));
    assert!(!cancel.is_aborted());
    assert_eq!(query.snapshot().check_current(&documents), Ok(()));
    assert_eq!(documents.get(&old).unwrap().revision(), revision);
    assert_eq!(documents.text(&occupied).as_deref(), Some("second"));
    assert_eq!(
        block_on(query.run(|_| async { 3 }))
            .unwrap()
            .publish(|value| value),
        Ok(3)
    );
}

#[test]
fn one_document_event_does_not_cancel_another_document_or_a_fresh_query() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let first = uri("file:///first.ts");
    let second = uri("file:///second.ts");
    project.open(first.clone(), "first".into(), 1, "typescript".into());
    project.open(second.clone(), "second".into(), 1, "typescript".into());
    let (old, old_cancel) = project.begin_query(&first).unwrap();
    let (other, other_cancel) = project.begin_query(&second).unwrap();
    project.open(first.clone(), "first".into(), 1, "typescript".into());
    assert!(old_cancel.is_aborted());
    assert!(!other_cancel.is_aborted());
    let (fresh, fresh_cancel) = project.begin_query(&first).unwrap();
    assert!(!fresh_cancel.is_aborted());
    assert_ne!(old.snapshot().revision(), fresh.snapshot().revision());
    assert_eq!(
        block_on(other.run(|_| async { 2 }))
            .unwrap()
            .publish(|value| value),
        Ok(2)
    );
    assert_eq!(
        block_on(fresh.run(|_| async { 3 }))
            .unwrap()
            .publish(|value| value),
        Ok(3)
    );
}

#[test]
fn dropping_project_wakes_suspended_owned_work_without_retaining_registry_entries() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///shutdown.ts");
    project.open(uri.clone(), "source".into(), 1, "typescript".into());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &uri, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    let before = wake.0.load(Ordering::Relaxed);
    drop(project);
    assert!(wake.0.load(Ordering::Relaxed) > before);
    assert!(matches!(
        poll(work.as_mut(), &wake),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

struct RegistryReentryOnDrop {
    active: Arc<super::super::ActiveQueries>,
    drops: Arc<AtomicUsize>,
}

impl ArcWake for RegistryReentryOnDrop {
    fn wake_by_ref(_: &Arc<Self>) {}
}

impl Drop for RegistryReentryOnDrop {
    fn drop(&mut self) {
        // A nonblocking re-entry detects the original deadlock without hanging
        // the runner. This destructor belongs to the genuine stored Waker.
        assert!(self.active.0.try_lock().is_some());
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn final_cancel_handle_drops_stored_waker_after_releasing_registry_lock() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///waker-drop.ts");
    project.open(uri.clone(), "source".into(), 1, "typescript".into());
    let computation_drops = Arc::new(AtomicUsize::new(0));
    let waker_drops = Arc::new(AtomicUsize::new(0));
    let mut work = Box::pin(pending(&project, &uri, &computation_drops));
    let waker = futures::task::waker(Arc::new(RegistryReentryOnDrop {
        active: Arc::clone(&project.active),
        drops: Arc::clone(&waker_drops),
    }));
    let mut context = Context::from_waker(&waker);
    assert!(work.as_mut().poll(&mut context).is_pending());
    assert!(work.as_mut().poll(&mut context).is_pending());
    drop(waker);
    assert_eq!(waker_drops.load(Ordering::Relaxed), 0);
    drop(work);
    assert_eq!(computation_drops.load(Ordering::Relaxed), 1);
    assert_eq!(waker_drops.load(Ordering::Relaxed), 1);
    assert!(project.active.0.lock().is_empty());
}

#[test]
fn accepted_empty_change_list_keeps_real_key_physical_buffer_and_pending_work() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///empty-changes.ts");
    project.open(uri.clone(), "source".into(), 3, "typescript".into());
    let (query, cancel) = project.begin_query(&uri).unwrap();
    let original = Arc::clone(query.snapshot());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &uri, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(project.apply_changes(&uri, Vec::new(), 4));
    assert!(!cancel.is_aborted());
    assert!(poll(work.as_mut(), &wake).is_pending());
    let (current, _) = project.begin_query(&uri).unwrap();
    assert_eq!(current.snapshot().key(), original.key());
    assert_eq!(current.snapshot().version(), 3);
    assert!(Arc::ptr_eq(current.snapshot(), &original));
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    drop(work);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}
