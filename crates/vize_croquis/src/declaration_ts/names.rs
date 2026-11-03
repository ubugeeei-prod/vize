//! Generated declaration names, allocated against the SFC's type parameters.

use super::type_exists;
use crate::Croquis;
use vize_carton::{String, append, cstr};

/// Every name written inside the SFC's type parameter scopes takes a spelling
/// no parameter binds, or `generic="Props"` would capture it (#11485). A type
/// the SFC declares itself keeps its own spelling.
pub(super) struct Names<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) bound: String,
}

impl Names<'_> {
    pub(super) fn local(&self, base: &str) -> String {
        let bound = |name: &str| self.bound.split(", ").any(|bound| bound == name);
        if !bound(base) || type_exists(self.summary, base) {
            return base.into();
        }
        (1usize..)
            .map(|suffix| cstr!("{base}_{suffix}"))
            .find(|name| !bound(name) && !type_exists(self.summary, name))
            .unwrap_or_else(|| base.into())
    }

    /// `export type Props`, or `type Props_1` re-exported by [`Self::reexport`].
    pub(super) fn export_head(&self, public: &str) -> String {
        let local = self.local(public);
        if local == public {
            cstr!("export type {local}")
        } else {
            cstr!("type {local}")
        }
    }

    pub(super) fn reexport(&self, ts: &mut String, public: &str) {
        let local = self.local(public);
        if local != public {
            append!(*ts, "export type {{ {local} as {public} }};\n");
        }
    }
}
