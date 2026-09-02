//! Builder methods for configurable built-in `musea/*` rules.

use super::config::Linter;
use crate::rules::musea::PreferDesignTokensConfig;
use vize_s0::String;

impl Linter {
    /// Configure design tokens for `musea/prefer-design-tokens`.
    ///
    /// Each entry is `(value, path, tier)`. Enabling the rule itself is still
    /// governed by the usual rule-enable configuration; this only supplies the
    /// token data the rule needs to avoid becoming a silent no-op.
    #[inline]
    pub fn with_musea_design_tokens(mut self, tokens: Vec<(String, String, String)>) -> Self {
        if tokens.is_empty() {
            return self;
        }
        let mut config = PreferDesignTokensConfig::default();
        for (value, path, tier) in tokens {
            config.add_token(&value, &path, &tier);
        }
        self.musea_design_tokens = Some(config);
        self
    }
}
