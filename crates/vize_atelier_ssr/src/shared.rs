//! What this crate shares with its `composed` copy:
//! the host context a compile takes, the result it returns, and the lane
//! that owns it. The copy names these items through this module, so a host
//! hands either compile the same values.

use vize_l0::String;

/// SSR codegen result
#[derive(Debug, Default)]
pub struct SsrCodegenResult {
    /// Generated render function code
    pub code: String,
    /// Import preamble
    pub preamble: String,
    /// Source Map v3 JSON for the generated render code.
    pub map: Option<String>,
}

/// Host-owned SFC metadata, separate from user compiler options. Existing
/// direct-template entry points retain their production helper spelling.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SsrSfcContext {
    #[doc(hidden)]
    pub slotted: bool,
    #[doc(hidden)]
    pub development: bool,
    #[doc(hidden)]
    pub jsx_closure: bool,
}

impl SsrSfcContext {
    /// Preserve direct-template production metadata with explicit scope ownership.
    #[must_use]
    pub const fn new(slotted: bool) -> Self {
        Self {
            slotted,
            development: false,
            jsx_closure: false,
        }
    }

    /// A JSX component owns lexical bindings directly, rather than the SFC
    /// `$setup` object. This host role does not alter template defaults.
    #[must_use]
    pub const fn with_jsx_closure(mut self) -> Self {
        self.jsx_closure = true;
        self
    }

    /// The adapter's explicit development target; never read the environment
    /// from an emitter or infer it from comment preservation.
    /// This target is independent of upstream JavaScript module-load mode.
    #[must_use]
    pub const fn with_development(mut self, development: bool) -> Self {
        self.development = development;
        self
    }
}

/// Which emitter owns a compile. Production always asks the L4 selector;
/// the differential battery pins the legacy walker on the same input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrLane {
    Selected,
    #[cfg(any(test, feature = "legacy-differential"))]
    LegacyOnly,
}
