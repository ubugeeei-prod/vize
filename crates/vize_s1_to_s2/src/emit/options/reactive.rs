//! Script reactivity facts on the binding table (P3-17).
//!
//! The shipped transform lane consults the Croquis reactivity tracker before
//! the binding kind when it decides how an inline render closure reads or
//! writes a setup binding (`IdentifierCollector::is_ref_binding` /
//! `needs_unref`, `is_ref_binding_simple`). The tracker is keyed by name
//! alone — nested declarations included, last registration wins — so the
//! facts carried here are exactly its `lookup(name)` answers, projected by
//! the backend that owns the Croquis summary. The codegen-side rewrites never
//! see the tracker and keep reading the binding kind.

use vize_s0::String;

use super::BindingTable;

/// How the script's reactivity tracker says a name is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveRead {
    /// `ref` / `shallowRef` / `computed` / `toRef`: an inline render closure
    /// reads and writes it through `.value` (`ReactiveKind::needs_value_access`).
    Value,
    /// Any other tracked kind (`reactive`, `readonly`, `toRefs`, …): read
    /// directly, never through `.value` or `_unref`.
    Direct,
}

impl BindingTable {
    /// Attach the reactivity tracker's facts; later duplicates of a name win,
    /// as the tracker's own name index does.
    #[must_use]
    pub fn with_reactive_reads<'n>(
        mut self,
        reads: impl IntoIterator<Item = (&'n str, ReactiveRead)>,
    ) -> Self {
        for (name, read) in reads {
            match self
                .reactive
                .binary_search_by(|(entry, _)| entry.as_str().cmp(name))
            {
                Ok(index) => self.reactive[index].1 = read,
                Err(index) => self.reactive.insert(index, (String::from(name), read)),
            }
        }
        self
    }

    /// The tracker's read of `name`, if the script tracked it.
    #[must_use]
    pub fn reactive_read(&self, name: &str) -> Option<ReactiveRead> {
        self.reactive
            .binary_search_by(|(entry, _)| entry.as_str().cmp(name))
            .ok()
            .map(|index| self.reactive[index].1)
    }
}
