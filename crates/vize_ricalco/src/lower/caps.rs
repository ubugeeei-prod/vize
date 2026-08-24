//! Vue dialect capabilities the S1→S2 lowering consults.
//!
//! A copy of the three template-sugar bits
//! [`vize_armature::legacy::LegacyDialectCapabilities`] names, kept
//! here so this crate never grows an armature edge. Resolved once per
//! file from [`vize_s0::config::VueVersion`]. Vue 3 is every flag
//! off — a single field-read short-circuit, the same zero-cost shape
//! the shipped `desugar_legacy_template` uses.

use vize_s0::config::VueVersion;

/// The three Vue 2 template-sugar surfaces P2-9 installment 7 ports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LegacyCaps {
    /// Pipe filters (`{{ msg | capitalize }}`). Every pre-Vue-3 line.
    pub supports_filters: bool,
    /// `slot-scope` / `scope` and `:foo.sync`. Vue 2 / 2.7 only.
    pub scoped_slot_attrs: bool,
    /// `.native` and numeric keyCodes on `v-on`. Vue 2 / 2.7 only.
    pub v2_event_sugar: bool,
}

impl LegacyCaps {
    /// The default dialect: every sugar surface off.
    pub const VUE3: Self = Self {
        supports_filters: false,
        scoped_slot_attrs: false,
        v2_event_sugar: false,
    };

    /// Resolve a config-selected dialect. Vue 3 (and any future
    /// non-legacy line) is [`Self::VUE3`].
    #[must_use]
    pub const fn for_version(version: VueVersion) -> Self {
        match version {
            VueVersion::V3 => Self::VUE3,
            VueVersion::V2 | VueVersion::V2_7 => Self {
                supports_filters: true,
                scoped_slot_attrs: true,
                v2_event_sugar: true,
            },
            VueVersion::V1 | VueVersion::V0_11 | VueVersion::V0_10 => Self {
                supports_filters: true,
                scoped_slot_attrs: false,
                v2_event_sugar: false,
            },
        }
    }

    /// Whether the legalizing pass has any work. Vue 3 is a single
    /// false — the tree is never walked for sugar.
    #[must_use]
    pub const fn needs_sugar(self) -> bool {
        self.supports_filters || self.scoped_slot_attrs || self.v2_event_sugar
    }
}

#[cfg(test)]
mod tests {
    use super::LegacyCaps;
    use vize_s0::config::VueVersion;

    #[test]
    fn vue3_is_every_flag_off() {
        let caps = LegacyCaps::for_version(VueVersion::V3);
        assert_eq!(caps, LegacyCaps::VUE3);
        assert!(!caps.needs_sugar());
    }

    #[test]
    fn vue2_and_vue2_7_share_the_template_sugar() {
        let v2 = LegacyCaps::for_version(VueVersion::V2);
        let v2_7 = LegacyCaps::for_version(VueVersion::V2_7);
        assert_eq!(v2, v2_7);
        assert!(v2.supports_filters && v2.scoped_slot_attrs && v2.v2_event_sugar);
        assert!(v2.needs_sugar());
    }

    #[test]
    fn vue1_keeps_filters_only() {
        let v1 = LegacyCaps::for_version(VueVersion::V1);
        assert!(v1.supports_filters);
        assert!(!v1.scoped_slot_attrs && !v1.v2_event_sugar);
        assert!(v1.needs_sugar());
    }
}
