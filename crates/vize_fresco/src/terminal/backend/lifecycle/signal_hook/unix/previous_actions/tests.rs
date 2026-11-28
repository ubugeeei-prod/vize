use super::{Actions, PreviousAction, PreviousActions, SUPERVISED_SIGNALS};
use std::{
    mem::MaybeUninit,
    sync::{Barrier, atomic::Ordering},
    thread,
};

fn actions(generation: usize) -> [libc::sigaction; SUPERVISED_SIGNALS.len()] {
    std::array::from_fn(|index| {
        // SAFETY: zero is a valid sigaction representation, including the
        // empty mask and optional restorer. These test words are never
        // installed or invoked; the actual cell only stores and copies them.
        let mut action = unsafe { MaybeUninit::<libc::sigaction>::zeroed().assume_init() };
        action.sa_sigaction = 0x1000 + generation * 16 + index;
        action.sa_flags = generation as _;
        action
    })
}

fn assert_snapshot(snapshot: &Actions, generation: usize) {
    for (index, saved) in snapshot.iter().enumerate() {
        let saved = saved.as_ref().unwrap();
        assert_eq!(saved.sa_sigaction, 0x1000 + generation * 16 + index);
        assert_eq!(saved.sa_flags as usize, generation);
    }
}

#[test]
fn unpublished_and_out_of_range_actions_are_absent() {
    let table = PreviousActions::new();
    assert!(table.read(0).is_none());
    table.publish(&actions(1));
    assert!(table.read(usize::MAX).is_none());
    assert_eq!(table.first.readers.load(Ordering::SeqCst), 0);
    assert_eq!(table.second.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn a_late_reservation_rejects_inactive_storage_and_accepts_complete_republication() {
    let table = PreviousActions::new();
    table.publish(&actions(1));
    // Pause the actual acquisition between its publication load and reserve.
    let observed = table.current.load(Ordering::SeqCst);
    table.publish(&actions(2));
    assert!(table.reserve(observed).is_none());
    table.publish(&actions(3));
    // The publication bit has returned to its former value. Validation now
    // accepts the complete new generation, rather than reading old bytes.
    let reader = table.reserve(observed).unwrap();
    assert_snapshot(reader.actions(), 3);
}

#[test]
fn same_thread_nested_readers_keep_independent_slots() {
    let table = PreviousActions::new();
    table.publish(&actions(1));
    let outer = table.acquire();
    let inner = table.acquire();
    assert_eq!(table.second.readers.load(Ordering::SeqCst), 2);
    drop(outer);
    assert_eq!(table.second.readers.load(Ordering::SeqCst), 1);
    table.publish(&actions(2));
    assert_snapshot(inner.actions(), 1);
    drop(inner);
    table.publish(&actions(3));
    assert_snapshot(table.acquire().actions(), 3);
}

#[test]
fn a_retained_reader_keeps_its_generation_while_the_installer_retries() {
    let table = PreviousActions::new();
    table.publish(&actions(1));
    let reader = table.acquire();
    let ready = Barrier::new(2);
    thread::scope(|scope| {
        let writer = scope.spawn(|| {
            table.publish(&actions(2));
            ready.wait();
            // This reuses the first reader's generation. It must wait until
            // that reader releases its slot, including after rollback.
            table.publish(&actions(3));
        });
        ready.wait();
        for _ in 0..16 {
            thread::yield_now();
            assert_snapshot(reader.actions(), 1);
        }
        drop(reader);
        writer.join().unwrap();
    });
    assert_snapshot(table.acquire().actions(), 3);
}

#[test]
fn concurrent_readers_observe_complete_immutable_generations() {
    let table = PreviousActions::new();
    table.publish(&actions(1));
    thread::scope(|scope| {
        for _ in 0..3 {
            let table = &table;
            scope.spawn(move || {
                for _ in 0..24 {
                    let reader = table.acquire();
                    let generation = reader
                        .actions()
                        .first()
                        .and_then(Option::as_ref)
                        .unwrap()
                        .sa_flags as usize;
                    thread::yield_now();
                    assert_snapshot(reader.actions(), generation);
                }
            });
        }
        // This is the sole publisher, as the production installation mutex
        // requires. Repeated retries exercise reuse of both generations.
        for generation in 2..=24 {
            table.publish(&actions(generation));
            thread::yield_now();
        }
    });
    assert_eq!(table.first.readers.load(Ordering::SeqCst), 0);
    assert_eq!(table.second.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn owned_chaining_values_release_the_reader_before_foreign_dispatch() {
    let table = PreviousActions::new();
    let mut saved = actions(1);
    let first = saved.first_mut().unwrap();
    first.sa_sigaction = libc::SIG_IGN;
    table.publish(&saved);
    assert!(matches!(table.read(0), Some(PreviousAction::Ignore)));
    let first = saved.first_mut().unwrap();
    first.sa_sigaction = libc::SIG_DFL;
    let flags = first.sa_flags;
    table.publish(&saved);
    let default = table
        .read(0)
        .and_then(|saved| match saved {
            PreviousAction::Default(action) => Some(action),
            _ => None,
        })
        .expect("the complete default action is retained");
    assert_eq!(default.sa_sigaction, libc::SIG_DFL);
    assert_eq!(default.sa_flags, flags);
    assert_eq!(table.first.readers.load(Ordering::SeqCst), 0);
    assert_eq!(table.second.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn custom_actions_preserve_the_handler_address_and_selected_abi() {
    let table = PreviousActions::new();
    let mut saved = actions(1);
    let first = saved.first_mut().unwrap();
    first.sa_flags = 0;
    table.publish(&saved);
    assert!(matches!(
        table.read(0),
        Some(PreviousAction::Simple(0x1010))
    ));
    let first = saved.first_mut().unwrap();
    first.sa_flags = libc::SA_SIGINFO as _;
    table.publish(&saved);
    assert!(matches!(
        table.read(0),
        Some(PreviousAction::Siginfo(0x1010))
    ));
    assert_eq!(table.first.readers.load(Ordering::SeqCst), 0);
    assert_eq!(table.second.readers.load(Ordering::SeqCst), 0);
}
