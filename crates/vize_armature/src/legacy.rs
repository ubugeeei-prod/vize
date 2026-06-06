//! Legacy Vue (v0 / v1 / v2) support surface.
//!
//! This module is the consolidation point for pre-Vue-3 ("legacy") parsing and
//! tokenization in `vize_armature`. It is gated behind the `legacy` cargo
//! feature and is **not** compiled into the default Vue 3 (`vize`) binary:
//! legacy support is strictly opt-in.
//!
//! The companion `legacy` modules in `vize_canon` (type checking) and
//! `vize_maestro` (editor / LSP) build on the [`LegacyVueVersion`] model
//! defined here, so all three layers agree on which legacy line is in play.

/// A legacy (pre-Vue-3) major line that Vize can opt into supporting.
///
/// Vize targets Vue 3 by default. These variants name the older runtimes whose
/// template syntax, reactivity, and component model differ enough from Vue 3 to
/// require a dedicated parsing / analysis path behind the `legacy` feature:
///
/// - [`V0`](Self::V0): the Vue 0.x (0.11-era) line, predating the modern SFC
///   toolchain.
/// - [`V1`](Self::V1): Vue 1.x (`v-repeat`, `track-by`, `$index`, `v-el`, …).
/// - [`V2`](Self::V2): Vue 2.x, including 2.7 (Options-API-first; `slot-scope`,
///   `.sync`, filters, functional components, single root, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyVueVersion {
    /// Vue 0.x (the 0.11-era line).
    V0,
    /// Vue 1.x.
    V1,
    /// Vue 2.x (including 2.7).
    V2,
}

impl LegacyVueVersion {
    /// Every supported legacy line, oldest first.
    pub const ALL: [LegacyVueVersion; 3] = [Self::V0, Self::V1, Self::V2];

    /// A short, stable identifier for the line (`"v0"`, `"v1"`, `"v2"`).
    ///
    /// Kept stable so it can be used in config values, diagnostics, and feature
    /// reporting without churning across releases.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V0 => "v0",
            Self::V1 => "v1",
            Self::V2 => "v2",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_round_trips_all_variants() {
        assert_eq!(LegacyVueVersion::ALL.len(), 3);
        assert_eq!(LegacyVueVersion::V0.as_str(), "v0");
        assert_eq!(LegacyVueVersion::V1.as_str(), "v1");
        assert_eq!(LegacyVueVersion::V2.as_str(), "v2");
    }
}
