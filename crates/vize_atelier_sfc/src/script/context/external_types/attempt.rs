//! A value one worker computes for every worker, unless computing it fails.

use std::sync::{Mutex, OnceLock, PoisonError};

/// A cell filled by the first successful attempt. Workers that ask while an
/// attempt runs wait for it instead of repeating it. A failed attempt leaves
/// the cell empty, so the next worker that asks tries again: a file that
/// could not be read once is not unreadable for the rest of the batch.
pub(super) struct Attempted<T> {
    value: OnceLock<T>,
    attempting: Mutex<()>,
}

impl<T> Default for Attempted<T> {
    fn default() -> Self {
        Self {
            value: OnceLock::new(),
            attempting: Mutex::new(()),
        }
    }
}

impl<T> Attempted<T> {
    pub(super) fn get(&self) -> Option<&T> {
        self.value.get()
    }

    /// The kept value, or the outcome of `attempt`: its value is kept, its
    /// error goes to this caller alone.
    pub(super) fn get_or_attempt<E>(
        &self,
        attempt: impl FnOnce() -> Result<T, E>,
    ) -> Result<&T, E> {
        if let Some(value) = self.value.get() {
            return Ok(value);
        }
        let _attempting = self
            .attempting
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(value) = self.value.get() {
            return Ok(value);
        }
        let value = attempt()?;
        Ok(self.value.get_or_init(|| value))
    }
}
