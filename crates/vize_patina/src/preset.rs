//! Built-in lint presets for Patina.

use std::sync::OnceLock;

/// Named lint presets exposed across Rust, CLI, and bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LintPreset {
    /// General-purpose defaults that focus on the common Vue happy path.
    HappyPath,
    /// Happy path rules plus stricter conventions and framework assumptions.
    Opinionated,
    /// Error-focused correctness checks.
    Essential,
    /// Starts with no built-in bundle so hosts can opt in rule-by-rule.
    Incremental,
    /// Broad defaults plus ecosystem integration rules, without opinionated conventions.
    #[default]
    Ecosystem,
    /// Opinionated rules adjusted for Nuxt auto-import conventions.
    Nuxt,
}

impl LintPreset {
    pub const ALL: [Self; 6] = [
        Self::HappyPath,
        Self::Opinionated,
        Self::Essential,
        Self::Incremental,
        Self::Ecosystem,
        Self::Nuxt,
    ];

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HappyPath => "happy-path",
            Self::Opinionated => "opinionated",
            Self::Essential => "essential",
            Self::Incremental => "incremental",
            Self::Ecosystem => "ecosystem",
            Self::Nuxt => "nuxt",
        }
    }

    #[inline]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "happy-path" | "happy_path" | "happy" => Some(Self::HappyPath),
            "default" | "recommended" => Some(Self::Ecosystem),
            "opinionated" | "strict" | "all" => Some(Self::Opinionated),
            "essential" => Some(Self::Essential),
            "incremental" => Some(Self::Incremental),
            "ecosystem" => Some(Self::Ecosystem),
            "nuxt" => Some(Self::Nuxt),
            _ => None,
        }
    }
}

const NO_SCRIPT_RULE_NAMES: &[&str] = &[];
static ECOSYSTEM_SCRIPT_RULE_NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
static OPINIONATED_SCRIPT_RULE_NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
static NUXT_SCRIPT_RULE_NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();

pub(crate) fn builtin_script_rule_names(preset: LintPreset) -> &'static [&'static str] {
    match preset {
        LintPreset::HappyPath | LintPreset::Essential | LintPreset::Incremental => {
            NO_SCRIPT_RULE_NAMES
        }
        LintPreset::Ecosystem => script_rule_names_for_preset(
            &ECOSYSTEM_SCRIPT_RULE_NAMES,
            LintPreset::Ecosystem.as_str(),
        ),
        LintPreset::Opinionated => script_rule_names_for_preset(
            &OPINIONATED_SCRIPT_RULE_NAMES,
            LintPreset::Opinionated.as_str(),
        ),
        LintPreset::Nuxt => {
            script_rule_names_for_preset(&NUXT_SCRIPT_RULE_NAMES, LintPreset::Nuxt.as_str())
        }
    }
}

pub(crate) fn ecosystem_builtin_script_rule_names() -> &'static [&'static str] {
    builtin_script_rule_names(LintPreset::Ecosystem)
}

fn script_rule_names_for_preset(
    cache: &'static OnceLock<Vec<&'static str>>,
    preset: &'static str,
) -> &'static [&'static str] {
    cache
        .get_or_init(|| crate::linter::script_rules::builtin_script_rule_names_for_preset(preset))
        .as_slice()
}

/// The opinionated `css/*` rule set, enabled by the strict presets.
const OPINIONATED_CSS_RULE_NAMES: &[&str] = &[
    "css/no-important",
    "css/no-id-selectors",
    "css/prefer-logical-properties",
    "css/require-font-display",
    "css/prefer-nested-selectors",
    "css/no-display-none",
    "css/no-v-bind-performance",
    "css/no-hardcoded-values",
    "css/no-utility-classes",
    "css/prefer-slotted",
];

/// Built-in `css/*` rules enabled by a preset.
///
/// The `css/*` rules are opt-in: only the strict `opinionated`/`nuxt` presets
/// enable them. The default `ecosystem` preset and the lighter presets enable
/// none, so existing users are unaffected. Hosts can still enable any `css/*`
/// rule by name via `with_enabled_rules`/`with_additional_rules`.
pub(crate) const fn builtin_css_rule_names(preset: LintPreset) -> &'static [&'static str] {
    match preset {
        LintPreset::Opinionated | LintPreset::Nuxt => OPINIONATED_CSS_RULE_NAMES,
        LintPreset::HappyPath
        | LintPreset::Essential
        | LintPreset::Incremental
        | LintPreset::Ecosystem => &[],
    }
}

#[cfg(test)]
mod tests;
