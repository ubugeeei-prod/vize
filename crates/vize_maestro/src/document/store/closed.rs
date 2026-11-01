//! One synchronous last-close boundary over the existing URI mutation stripes.

use super::DocumentStore;

impl DocumentStore {
    /// No callback may mutate documents, await, retire native workers or wake tasks.
    /// A new open may start while this runs, but cannot publish its URI until it returns.
    pub(crate) fn try_while_closed<T>(&self, apply: impl FnOnce() -> T) -> Option<T> {
        self.uri_snapshots
            .try_while_empty(|| {
                self.stable_revision()?;
                self.documents.is_empty().then(apply)
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests;
