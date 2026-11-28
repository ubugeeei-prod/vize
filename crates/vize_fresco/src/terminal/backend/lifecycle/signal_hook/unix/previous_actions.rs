//! Bounded publication of the saved actions a signal handler reads.
//!
//! A failed installation can restore every action while an earlier handler is
//! still running. Reusing one mutable table after rollback therefore races.
//! Each publication now fills the inactive generation, after its readers have
//! drained, then publishes it. A reader reserves the observed generation and
//! validates it before borrowing any bytes. Only the installer waits; the
//! handler uses lock-free atomics and releases its slot before any foreign
//! action is invoked.

use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use super::SUPERVISED_SIGNALS;

#[cfg(test)]
mod tests;

type Actions = [Option<libc::sigaction>; SUPERVISED_SIGNALS.len()];

struct Generation {
    actions: UnsafeCell<Actions>,
    readers: AtomicUsize,
}

impl Generation {
    const fn new() -> Self {
        Self {
            actions: UnsafeCell::new([None; SUPERVISED_SIGNALS.len()]),
            readers: AtomicUsize::new(0),
        }
    }
}

pub(super) struct PreviousActions {
    first: Generation,
    second: Generation,
    current: AtomicBool,
}

impl PreviousActions {
    pub(super) const fn new() -> Self {
        Self {
            first: Generation::new(),
            second: Generation::new(),
            current: AtomicBool::new(false),
        }
    }

    fn generation(&self, current: bool) -> &Generation {
        if current { &self.second } else { &self.first }
    }

    /// Only the installer calls this, under `SIGNAL_HOOK_INSTALLATION`.
    /// Published generations have a fixed address and use bounded storage.
    pub(super) fn publish(&self, previous: &[libc::sigaction]) {
        let mut actions = [None; SUPERVISED_SIGNALS.len()];
        for (slot, saved) in actions.iter_mut().zip(previous) {
            *slot = Some(*saved);
        }
        let next = !self.current.load(Ordering::SeqCst);
        let generation = self.generation(next);
        let mut spins = 0_u8;
        while generation.readers.load(Ordering::SeqCst) != 0 {
            if spins < 64 {
                spins = spins.saturating_add(1);
                std::hint::spin_loop();
            } else {
                std::thread::yield_now();
            }
        }
        // SAFETY: the installation mutex serializes publishers. This is the
        // inactive generation, and every previously validated reader drained.
        // A reader that reserves it after the zero check must validate again:
        // it either observes the other generation and reads no bytes, or it
        // observes the publication below, after this complete write. The
        // sequentially consistent order makes those cases exhaustive.
        unsafe { *generation.actions.get() = actions };
        self.current.store(next, Ordering::SeqCst);
    }

    fn acquire(&self) -> Reader<'_> {
        loop {
            let current = self.current.load(Ordering::SeqCst);
            if let Some(reader) = self.reserve(current) {
                return reader;
            }
        }
    }

    #[inline]
    fn reserve(&self, current: bool) -> Option<Reader<'_>> {
        let generation = self.generation(current);
        generation.readers.fetch_add(1, Ordering::SeqCst);
        if self.current.load(Ordering::SeqCst) != current {
            generation.readers.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(Reader { generation })
    }

    pub(super) fn read(&self, index: usize) -> Option<PreviousAction> {
        let reader = self.acquire();
        reader
            .actions()
            .get(index)?
            .as_ref()
            .map(PreviousAction::from)
    }
}

// SAFETY: only serialized publishers mutate an inactive, drained generation.
// Validated readers keep their generation immutable until their slot drops;
// see `publish` and `acquire`. `sigaction` is plain Send + Sync C data.
unsafe impl Sync for PreviousActions {}

struct Reader<'a> {
    generation: &'a Generation,
}

impl Reader<'_> {
    fn actions(&self) -> &Actions {
        // SAFETY: `acquire` reserved this generation before validating its
        // publication, in the same sequentially consistent order as the
        // publisher's drain check. Publication follows the complete write.
        // Reuse waits for this reader's slot. The returned borrow is tied to
        // `self`, so it ends before Drop releases that slot; Drop itself
        // contains no reference to the action bytes.
        unsafe { &*self.generation.actions.get() }
    }
}

impl Drop for Reader<'_> {
    fn drop(&mut self) {
        self.generation.readers.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Copy only what chaining needs. A custom or ignored action needs no signal
/// mask; the default path retains the complete original action for restoration.
/// This value owns its data, so no reader slot crosses a foreign callback.
pub(super) enum PreviousAction {
    Ignore,
    Default(libc::sigaction),
    Simple(usize),
    Siginfo(usize),
}

impl From<&libc::sigaction> for PreviousAction {
    fn from(action: &libc::sigaction) -> Self {
        if action.sa_sigaction == libc::SIG_IGN {
            Self::Ignore
        } else if action.sa_sigaction == libc::SIG_DFL {
            Self::Default(*action)
        } else {
            #[expect(
                unused_assignments,
                reason = "the dead initializer infers sa_flags' platform integer type"
            )]
            let mut siginfo = action.sa_flags;
            siginfo = libc::SA_SIGINFO as _;
            if action.sa_flags & siginfo == 0 {
                Self::Simple(action.sa_sigaction)
            } else {
                Self::Siginfo(action.sa_sigaction)
            }
        }
    }
}
