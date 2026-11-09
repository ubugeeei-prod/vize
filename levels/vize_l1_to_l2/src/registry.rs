//! L1→L2 conversion-pattern registry.
//!
//! Conversion owners ask this module for the versioned pattern set they may
//! apply. Concrete variants stay out of the neutral orchestration modules.

use vize_l0::{FileDialect, pass::Pipeline};

use crate::dialect::LegacyCaps;

/// Conditional legalization hooks selected from the shared L1 capability
/// descriptor. Baseline `.sync` and `slot-scope` rewrites remain part of the
/// existing pass; this table owns the capability-dependent branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegalizationPatterns {
    filters: bool,
    vue2_events: bool,
}

impl LegalizationPatterns {
    #[must_use]
    pub(crate) const fn filters(self) -> bool {
        self.filters
    }

    #[must_use]
    pub(crate) const fn vue2_events(self) -> bool {
        self.vue2_events
    }
}

/// Select the capability-dependent legalization patterns once per file.
#[must_use]
pub(crate) const fn legalization_patterns(caps: LegacyCaps) -> LegalizationPatterns {
    LegalizationPatterns {
        filters: caps.supports_filters,
        vue2_events: caps.v2_event_sugar,
    }
}

/// Select the existing conversion pipeline from the registered patterns.
#[must_use]
pub const fn legalization_pipeline(caps: LegacyCaps) -> Pipeline {
    if caps.needs_sugar() {
        crate::dialect::legalize::LEGACY
    } else {
        crate::pass::TRANSFORM
    }
}

/// Whether the scoped-style selected-SFC conversion is registered.
#[must_use]
pub fn selected_scoped_sfc(dialect: FileDialect) -> bool {
    vize_l1::registry::is_standard_vue3_component(dialect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::{Lang, TemplateSyntaxMode, config::VueVersion, dialect::VueDialect};

    #[test]
    fn scoped_pattern_is_registered_only_for_the_existing_profile() {
        let vue3 = FileDialect::component(
            VueVersion::V3,
            VueDialect::Vue,
            TemplateSyntaxMode::Standard,
            Lang::Js,
        );
        let vue2 = FileDialect::component(
            VueVersion::V2,
            VueDialect::Vue,
            TemplateSyntaxMode::Standard,
            Lang::Js,
        );
        assert!(selected_scoped_sfc(vue3));
        assert!(!selected_scoped_sfc(vue2));
    }

    #[test]
    fn legalization_patterns_keep_the_existing_version_matrix() {
        let patterns = |version| legalization_patterns(LegacyCaps::for_version(version));
        for version in [VueVersion::V0_10, VueVersion::V0_11, VueVersion::V1] {
            assert_eq!(
                patterns(version),
                LegalizationPatterns {
                    filters: true,
                    vue2_events: false,
                },
                "{version:?}"
            );
        }
        for version in [VueVersion::V2, VueVersion::V2_7] {
            assert_eq!(
                patterns(version),
                LegalizationPatterns {
                    filters: true,
                    vue2_events: true,
                },
                "{version:?}"
            );
        }
        assert_eq!(
            patterns(VueVersion::V3),
            LegalizationPatterns {
                filters: false,
                vue2_events: false,
            }
        );
    }
}
