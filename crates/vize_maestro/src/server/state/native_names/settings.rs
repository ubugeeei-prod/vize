//! Values copied from actual server settings under their coherent gate.
use vize_l0::config::{VueDialect, VueVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeNamesSettings {
    version: VueVersion,
    configured_dialect: Option<VueDialect>,
    legacy: bool,
    patterned: bool,
    native_linked: bool,
    rename: bool,
}
impl NativeNamesSettings {
    pub(super) fn new(
        version: VueVersion,
        configured_dialect: Option<VueDialect>,
        legacy: bool,
        patterned: bool,
        native_linked: bool,
        rename: bool,
    ) -> Self {
        Self {
            version,
            configured_dialect,
            legacy,
            patterned,
            native_linked,
            rename,
        }
    }
    pub(crate) fn version(self) -> VueVersion {
        self.version
    }
    pub(crate) fn configured_dialect(self) -> Option<VueDialect> {
        self.configured_dialect
    }
    pub(crate) fn legacy(self) -> bool {
        self.legacy
    }
    pub(crate) fn patterned(self) -> bool {
        self.patterned
    }
    pub(super) fn linked_enabled(self) -> bool {
        self.native_linked && self.rename
    }
    pub(super) fn parser_settings(self) -> (VueVersion, Option<VueDialect>, bool, bool) {
        (
            self.version,
            self.configured_dialect,
            self.legacy,
            self.patterned,
        )
    }
    pub(super) fn linked_settings(self) -> (bool, bool) {
        (self.native_linked, self.rename)
    }
}
