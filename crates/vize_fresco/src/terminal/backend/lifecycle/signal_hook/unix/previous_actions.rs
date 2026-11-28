//! Bounded publication of the saved actions a signal handler reads.
//!
//! A failed installation can restore every action while an earlier handler is
//! still running. Reusing one mutable table after rollback therefore races.
//! Publications reuse two bounded table allocations. Reusing an older
//! generation waits for readers to drain before replacing it. A reader
//! reserves the observed publication and validates it before borrowing any
//! bytes. Successful installation seals the table permanently, so subsequent
//! handlers read immutable storage without updating a reader counter. Only
//! the installer waits, and no guard crosses a foreign callback.

use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

use super::SUPERVISED_SIGNALS;

#[cfg(test)]
mod tests;

type Actions = [libc::sigaction; SUPERVISED_SIGNALS.len()];

const CURRENT: u8 = 1;
const FIRST: u8 = 2;
const SECOND: u8 = 4;
const SEALED: u8 = 8;

struct Generation {
    actions: UnsafeCell<Option<Box<Actions>>>,
}

impl Generation {
    const fn new() -> Self {
        Self {
            actions: UnsafeCell::new(None),
        }
    }
}

pub(super) struct PreviousActions {
    first: Generation,
    second: Generation,
    readers: AtomicUsize,
    state: AtomicU8,
}

impl PreviousActions {
    pub(super) const fn new() -> Self {
        Self {
            first: Generation::new(),
            second: Generation::new(),
            readers: AtomicUsize::new(0),
            state: AtomicU8::new(0),
        }
    }

    fn generation(&self, current: bool) -> &Generation {
        if current { &self.second } else { &self.first }
    }

    /// Publication and sealing share `SIGNAL_HOOK_INSTALLATION`.
    /// Reuse an inactive allocation; at most two fixed allocations exist.
    pub(super) fn publish(&self, previous: Actions) -> Reader<'_> {
        let state = self.state.load(Ordering::SeqCst);
        assert_eq!(
            state & SEALED,
            0,
            "sealed signal actions cannot be republished"
        );
        let initialized = if state & CURRENT == 0 { SECOND } else { FIRST };
        let previous = match self.replace_inactive(state, None) {
            Some(mut allocation) => {
                *allocation = previous;
                allocation
            }
            None => Box::new(previous),
        };
        drop(self.replace_inactive(state, Some(previous)));
        self.state
            .store((state | initialized) ^ CURRENT, Ordering::SeqCst);
        self.acquire()
    }

    /// Seal under the installation mutex, then free the drained older allocation.
    pub(super) fn seal(&self) {
        let state = self.state.fetch_or(SEALED, Ordering::SeqCst) | SEALED;
        drop(self.replace_inactive(state, None));
    }

    fn replace_inactive(&self, state: u8, previous: Option<Box<Actions>>) -> Option<Box<Actions>> {
        let next = state & CURRENT == 0;
        let initialized = if next { SECOND } else { FIRST };
        if state & initialized != 0 {
            let mut spins = 0_u8;
            while self.readers.load(Ordering::SeqCst) != 0 {
                if spins < 64 {
                    spins = spins.saturating_add(1);
                    std::hint::spin_loop();
                } else {
                    std::thread::yield_now();
                }
            }
        }
        // SAFETY: the installation mutex serializes publication and sealing.
        // The inactive generation has never been published (initial readers
        // borrow no generation), or every previously validated reader drained.
        // Late reservations validate the publication again before reading: they
        // reject an inactive generation, or see the complete publication below.
        // Sealing rejects all future publishers before uncounted readers can
        // start, and reclaims only the inactive allocation after draining.
        let generation = self.generation(next);
        unsafe { std::mem::replace(&mut *generation.actions.get(), previous) }
    }

    fn acquire(&self) -> Reader<'_> {
        loop {
            let state = self.state.load(Ordering::SeqCst);
            if let Some(reader) = self.reserve(state) {
                return reader;
            }
        }
    }

    #[inline]
    fn reserve(&self, state: u8) -> Option<Reader<'_>> {
        if state & (FIRST | SECOND) == 0 {
            return Some(Reader {
                generation: None,
                readers: &self.readers,
                reserved: false,
            });
        }
        let generation = Some(self.generation(state & CURRENT != 0));
        let reserved = state & SEALED == 0;
        if reserved {
            self.readers
                .try_update(Ordering::SeqCst, Ordering::SeqCst, |readers| {
                    readers.checked_add(1)
                })
                .ok()?;
            if self.state.load(Ordering::SeqCst) != state {
                self.readers.fetch_sub(1, Ordering::SeqCst);
                return None;
            }
        }
        Some(Reader {
            generation,
            readers: &self.readers,
            reserved,
        })
    }

    pub(super) fn read(&self, index: usize) -> Option<PreviousAction> {
        let reader = self.acquire();
        Some(PreviousAction::from(reader.actions().get(index)?))
    }
}

// SAFETY: serialized publishers mutate an inactive, unpublished or drained
// generation. Validated readers pin their generation until their slot drops.
// Sealing forbids future publication before readers stop reserving slots,
// then reclaims only the inactive allocation after reserved readers drain.
// See `publish` and `acquire`; `sigaction` is plain Send + Sync C data.
unsafe impl Sync for PreviousActions {}

pub(super) struct Reader<'a> {
    generation: Option<&'a Generation>,
    readers: &'a AtomicUsize,
    reserved: bool,
}

impl Reader<'_> {
    pub(super) fn actions(&self) -> &[libc::sigaction] {
        let Some(generation) = self.generation else {
            return &[];
        };
        // SAFETY: `acquire` reserved this generation before validating its
        // publication, in the same sequentially consistent order as the
        // publisher's drain check, or observed a sealed table that forbids
        // subsequent writes to this current generation. Publication follows
        // the complete write.
        // Reuse waits for reserved readers; the returned borrow ends before
        // Drop releases that slot. Drop holds no reference to the action bytes.
        let actions = unsafe { &*generation.actions.get() };
        actions.as_deref().map_or(&[], |actions| actions.as_slice())
    }
}

impl Drop for Reader<'_> {
    fn drop(&mut self) {
        if self.reserved {
            self.readers.fetch_sub(1, Ordering::SeqCst);
        }
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
