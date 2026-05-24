//! Linter config model.

use serde::{Deserialize, Serialize};

use crate::{FxHashMap, String};

/// Per-rule lint severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintRuleSeverity {
    Off,
    Warn,
    Error,
}

/// Linter settings shared by CLI and IDE linting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LinterConfig {
    pub enabled: bool,
    pub preset: Option<String>,
    pub rules: FxHashMap<String, LintRuleSeverity>,
}

impl LinterConfig {
    /// Returns true when the config matches the built-in defaults.
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    /// Rule names explicitly disabled by config.
    pub fn disabled_rules(&self) -> Vec<String> {
        let mut rules = self
            .rules
            .iter()
            .filter(|(_, severity)| matches!(severity, LintRuleSeverity::Off))
            .map(|(rule, _)| rule.clone())
            .collect::<Vec<_>>();
        rules.sort();
        rules
    }

    /// Rule names explicitly enabled by config.
    pub fn enabled_rules(&self) -> Vec<String> {
        let mut rules = self
            .rules
            .iter()
            .filter(|(_, severity)| !matches!(severity, LintRuleSeverity::Off))
            .map(|(rule, _)| rule.clone())
            .collect::<Vec<_>>();
        rules.sort();
        rules
    }
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: None,
            rules: FxHashMap::default(),
        }
    }
}
