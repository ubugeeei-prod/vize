//! Apply `linter.ruleOptions["vue/v-bind-style"]`.

use super::super::config::Linter;
use crate::markup::SameNameSpelling;
use crate::rules::vue::{PreferPropsShorthand, VBindStyle, VBindStyleOption};
use vize_l0::config::{VBindSameNameShorthand, VBindStyle as ConfigVBindStyle, VBindStyleConfig};

pub(super) fn apply(linter: Linter, config: VBindStyleConfig) -> Linter {
    linter.with_v_bind_style_policy(
        match config.style() {
            ConfigVBindStyle::Shorthand => VBindStyleOption::Shorthand,
            ConfigVBindStyle::Longform => VBindStyleOption::Longform,
        },
        config.same_name_shorthand(),
    )
}

impl Linter {
    /// Configure the spelling and the same-name policy of `vue/v-bind-style`
    /// without enabling an unselected rule.
    ///
    /// `always` and `never` decide how a binding that reads its own argument
    /// is written. `vue/prefer-props-shorthand` asks for the shorthand too, so
    /// it stays quiet while `vue/v-bind-style` runs with either policy: one
    /// binding gets one report and one fix.
    pub fn with_v_bind_style_policy(
        mut self,
        style: VBindStyleOption,
        same_name: VBindSameNameShorthand,
    ) -> Self {
        let required = match same_name {
            VBindSameNameShorthand::Ignore => return self.with_v_bind_style(style),
            VBindSameNameShorthand::Always => SameNameSpelling::Shorthand,
            VBindSameNameShorthand::Never => SameNameSpelling::Explicit,
        };
        self.registry
            .replace(VBindStyle { style }.requiring_same_name(required));
        if self.registry.has_rule("vue/prefer-props-shorthand") {
            self.registry
                .replace(PreferPropsShorthand::deferring_to_v_bind_style());
        }
        self
    }
}

#[cfg(test)]
mod tests;
