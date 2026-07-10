use super::config::Linter;
use vize_carton::{String, config::VueVersion};

impl Linter {
    /// Apply project-wide Vue dialect compatibility to lint rules.
    #[inline]
    pub fn with_vue_version(mut self, version: Option<VueVersion>) -> Self {
        if version.is_some_and(VueVersion::is_legacy) {
            self.disabled_rules
                .insert(String::from("vue/no-v-for-template-key-on-child"));
        }
        self
    }

    /// Apply project-wide SFC Vapor mode to lint rules.
    #[inline]
    pub fn with_vapor_mode(mut self, enabled: Option<bool>) -> Self {
        if enabled == Some(false) {
            self.disabled_rules
                .insert(String::from("script/no-get-current-instance"));
            self.disabled_rules
                .insert(String::from("script/no-next-tick"));
        }
        self
    }
}
