//! Batch-private return factoring; owner intent is distinct from semantic authority.

use std::path::{Path, PathBuf};

use super::VirtualProject;

mod edit;
mod materialization;
mod syntax;

/// The sole real external module artifact. Construction remains private to the
/// canonical project owner; no public caller can supply a certificate or path.
pub(super) struct HelperFile {
    pub(super) root: PathBuf,
    pub(super) path: PathBuf,
    pub(super) body: &'static str,
}

/// Exact body exercised by the pure held-SHARED/TS6 consumer controls. All five
/// callee references remain global and their independent distribution stays put.
pub(super) const HELPER_BODY: &str = "type _PrivateReturn<P, D, A, I> = $UsePublicInstance<A> extends true ? I : $ReplaceComponentProps<I, $AuthoredKeyWitness<A> & $ComponentInputProps<P, D> & $ComponentInputGuard<P, D, A>>;\nexport type { _PrivateReturn as default };\n";

/// Only the canonical owner constructs an active plan after all witness checks.
/// A public terminal intent cannot mint this value.
pub(super) struct ActiveReturn {
    helper: HelperFile,
    fingerprint: vize_carton::String,
}

impl VirtualProject {
    pub(super) fn private_return_fingerprint(&self) -> Option<&str> {
        self.private_return
            .as_ref()
            .map(|active| active.fingerprint.as_str())
    }

    pub(crate) fn owned_cli_support_path(&self) -> Option<&Path> {
        self.private_return
            .as_ref()
            .map(|active| active.helper.path.as_path())
    }
}
