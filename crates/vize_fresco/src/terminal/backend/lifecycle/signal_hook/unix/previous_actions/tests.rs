use super::{PreviousAction, PreviousActions, SUPERVISED_SIGNALS};
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

fn assert_snapshot(snapshot: &[libc::sigaction], generation: usize) {
    for (index, saved) in snapshot.iter().enumerate() {
        assert_eq!(saved.sa_sigaction, 0x1000 + generation * 16 + index);
        assert_eq!(saved.sa_flags as usize, generation);
    }
}

#[test]
fn unpublished_and_out_of_range_actions_are_absent() {
    let table = PreviousActions::new();
    assert!(table.read(0).is_none());
    table.publish(actions(1));
    assert!(table.read(usize::MAX).is_none());
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn a_late_reservation_rejects_inactive_storage_and_accepts_complete_republication() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    table.publish(actions(2));
    // Pause the actual acquisition between its publication load and reserve.
    let observed = table.state.load(Ordering::SeqCst);
    table.publish(actions(3));
    assert!(table.reserve(observed).is_none());
    table.publish(actions(4));
    // The publication bit has returned to its former value. Validation now
    // accepts the complete new generation, rather than reading old bytes.
    let reader = table.reserve(observed).unwrap();
    assert_snapshot(reader.actions(), 4);
}

#[test]
fn same_thread_nested_readers_keep_independent_slots() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    let outer = table.acquire();
    let inner = table.acquire();
    assert_eq!(table.readers.load(Ordering::SeqCst), 2);
    drop(outer);
    assert_eq!(table.readers.load(Ordering::SeqCst), 1);
    table.publish(actions(2));
    assert_snapshot(inner.actions(), 1);
    drop(inner);
    table.publish(actions(3));
    assert_snapshot(table.acquire().actions(), 3);
}

#[test]
fn a_retained_reader_keeps_its_generation_while_the_installer_retries() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    let reader = table.acquire();
    let ready = Barrier::new(2);
    thread::scope(|scope| {
        let writer = scope.spawn(|| {
            table.publish(actions(2));
            ready.wait();
            // This reuses the first reader's generation. It must wait until
            // that reader releases its slot, including after rollback.
            table.publish(actions(3));
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
    table.publish(actions(1));
    thread::scope(|scope| {
        for _ in 0..3 {
            let table = &table;
            scope.spawn(move || {
                for _ in 0..24 {
                    let reader = table.acquire();
                    let generation = reader.actions().first().unwrap().sa_flags as usize;
                    thread::yield_now();
                    assert_snapshot(reader.actions(), generation);
                }
            });
        }
        // This is the sole publisher, as the production installation mutex
        // requires. Repeated retries exercise reuse of both generations.
        for generation in 2..=24 {
            table.publish(actions(generation));
            thread::yield_now();
        }
    });
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn owned_chaining_values_release_the_reader_before_foreign_dispatch() {
    let table = PreviousActions::new();
    let mut saved = actions(1);
    let first = saved.first_mut().unwrap();
    first.sa_sigaction = libc::SIG_IGN;
    table.publish(saved);
    assert!(matches!(table.read(0), Some(PreviousAction::Ignore)));
    let first = saved.first_mut().unwrap();
    first.sa_sigaction = libc::SIG_DFL;
    let flags = first.sa_flags;
    table.publish(saved);
    let default = table
        .read(0)
        .and_then(|saved| match saved {
            PreviousAction::Default(action) => Some(action),
            _ => None,
        })
        .expect("the complete default action is retained");
    assert_eq!(default.sa_sigaction, libc::SIG_DFL);
    assert_eq!(default.sa_flags, flags);
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn custom_actions_preserve_the_handler_address_and_selected_abi() {
    let table = PreviousActions::new();
    let mut saved = actions(1);
    let first = saved.first_mut().unwrap();
    first.sa_flags = 0;
    table.publish(saved);
    assert!(matches!(
        table.read(0),
        Some(PreviousAction::Simple(0x1010))
    ));
    let first = saved.first_mut().unwrap();
    first.sa_flags = libc::SA_SIGINFO as _;
    table.publish(saved);
    assert!(matches!(
        table.read(0),
        Some(PreviousAction::Siginfo(0x1010))
    ));
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn an_unpublished_reader_never_borrows_a_later_generation() {
    let table = PreviousActions::new();
    let reader = table.acquire();
    table.publish(actions(1));
    table.publish(actions(2));
    assert!(reader.actions().is_empty());
    assert_snapshot(table.acquire().actions(), 2);
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
}

#[test]
fn sealing_keeps_reserved_readers_and_removes_later_reader_updates() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    let reserved = table.acquire();
    table.seal();
    let sealed = table.acquire();
    assert_eq!(table.readers.load(Ordering::SeqCst), 1);
    assert_snapshot(reserved.actions(), 1);
    assert_snapshot(sealed.actions(), 1);
    drop(reserved);
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
    assert_snapshot(sealed.actions(), 1);
}

#[test]
#[should_panic(expected = "sealed signal actions cannot be republished")]
fn a_sealed_table_rejects_republication_before_mutation() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    table.seal();
    let reader = table.acquire();
    assert_snapshot(reader.actions(), 1);
    table.publish(actions(2));
}

#[test]
fn an_exhausted_reader_counter_rejects_a_reservation_without_wrapping() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    table.readers.store(usize::MAX, Ordering::SeqCst);
    let state = table.state.load(Ordering::SeqCst);
    assert!(table.reserve(state).is_none());
    assert_eq!(table.readers.load(Ordering::SeqCst), usize::MAX);
    table.readers.store(0, Ordering::SeqCst);
    assert_snapshot(table.acquire().actions(), 1);
}

#[test]
fn sealing_drains_old_readers_before_reclaiming_the_inactive_allocation() {
    let table = PreviousActions::new();
    table.publish(actions(1));
    let retained = table.acquire();
    let old_state = table.state.load(Ordering::SeqCst);
    table.publish(actions(2));
    thread::scope(|scope| {
        let installer = scope.spawn(|| table.seal());
        while table.state.load(Ordering::SeqCst) & super::SEALED == 0 {
            thread::yield_now();
        }
        assert_snapshot(retained.actions(), 1);
        assert_snapshot(table.acquire().actions(), 2);
        assert!(table.reserve(old_state).is_none());
        drop(retained);
        installer.join().unwrap();
    });
    assert_eq!(table.readers.load(Ordering::SeqCst), 0);
    assert_snapshot(table.acquire().actions(), 2);
    // The inactive allocation was reclaimed: use the actual checked borrow
    // with its reserved generation to observe the empty cell after sealing.
    let inactive = super::Reader {
        generation: Some(&table.second),
        readers: &table.readers,
        reserved: false,
    };
    assert!(inactive.actions().is_empty());
}

#[test]
fn retries_reuse_each_generation_allocation() {
    let table = PreviousActions::new();
    let first = table.publish(actions(1)).actions().as_ptr();
    let second = table.publish(actions(2)).actions().as_ptr();
    assert_ne!(first, second);
    assert_eq!(table.publish(actions(3)).actions().as_ptr(), first);
    assert_eq!(table.publish(actions(4)).actions().as_ptr(), second);
}
