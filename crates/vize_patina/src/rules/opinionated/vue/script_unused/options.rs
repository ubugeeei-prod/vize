//! Rule options in the typescript-eslint `no-unused-vars` and ESLint
//! `prefer-const` shapes.
//!
//! Defaults match typescript-eslint except that `vars` and `args` ignore
//! `^_`, as the rule documents. Caught errors have no default pattern.

use regex::Regex;
use vize_l0::config::{
    NoUnusedScriptVarsOptions as ConfigUnused, PreferConstDestructuring,
    PreferConstOptions as ConfigPreferConst, UnusedArgs, UnusedCaughtErrors, UnusedVarsScope,
};
use vize_l0::{String, ToCompactString, cstr};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Args {
    All,
    AfterUsed,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Vars {
    All,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Caught {
    All,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Destructuring {
    Any,
    All,
}

pub(super) struct Pattern {
    source: String,
    regex: Regex,
}

impl Pattern {
    fn compile(source: &str) -> Option<Self> {
        let source = source.trim();
        if source.is_empty() {
            return None;
        }
        Regex::new(source).ok().map(|regex| Self {
            source: source.to_compact_string(),
            regex,
        })
    }

    pub(super) fn matches(&self, name: &str) -> bool {
        self.regex.is_match(name)
    }

    pub(super) fn source(&self) -> &str {
        &self.source
    }
}

pub(super) struct UnusedOptions {
    pub(super) args: Args,
    pub(super) args_ignore: Option<Pattern>,
    pub(super) vars: Vars,
    pub(super) vars_ignore: Option<Pattern>,
    pub(super) caught: Caught,
    pub(super) caught_ignore: Option<Pattern>,
    pub(super) array_ignore: Option<Pattern>,
    pub(super) ignore_rest_siblings: bool,
    pub(super) ignore_static_block_classes: bool,
    pub(super) report_used_ignore_pattern: bool,
}

impl Default for UnusedOptions {
    fn default() -> Self {
        Self {
            args: Args::AfterUsed,
            args_ignore: Pattern::compile("^_"),
            vars: Vars::All,
            vars_ignore: Pattern::compile("^_"),
            caught: Caught::All,
            caught_ignore: None,
            array_ignore: None,
            ignore_rest_siblings: false,
            ignore_static_block_classes: false,
            report_used_ignore_pattern: false,
        }
    }
}

impl UnusedOptions {
    /// Options from `linter.ruleOptions["vue/no-unused-script-vars"]`.
    pub(super) fn from_config(config: &ConfigUnused) -> Self {
        let defaults = Self::default();
        let pattern = |value: &Option<String>, fallback: Option<Pattern>| match value {
            Some(source) => Pattern::compile(source),
            None => fallback,
        };
        Self {
            args: match config.args {
                Some(UnusedArgs::All) => Args::All,
                Some(UnusedArgs::None) => Args::None,
                Some(UnusedArgs::AfterUsed) | None => Args::AfterUsed,
            },
            args_ignore: pattern(&config.args_ignore_pattern, defaults.args_ignore),
            vars: match config.vars {
                Some(UnusedVarsScope::Local) => Vars::Local,
                Some(UnusedVarsScope::All) | None => Vars::All,
            },
            vars_ignore: pattern(&config.vars_ignore_pattern, defaults.vars_ignore),
            caught: match config.caught_errors {
                Some(UnusedCaughtErrors::None) => Caught::None,
                Some(UnusedCaughtErrors::All) | None => Caught::All,
            },
            caught_ignore: pattern(&config.caught_errors_ignore_pattern, None),
            array_ignore: pattern(&config.destructured_array_ignore_pattern, None),
            ignore_rest_siblings: config.ignore_rest_siblings.unwrap_or(false),
            ignore_static_block_classes: config
                .ignore_class_with_static_init_block
                .unwrap_or(false),
            report_used_ignore_pattern: config.report_used_ignore_pattern.unwrap_or(false),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PreferConstOptions {
    pub(super) destructuring: Destructuring,
    pub(super) ignore_read_before_assign: bool,
}

impl Default for PreferConstOptions {
    fn default() -> Self {
        Self {
            destructuring: Destructuring::Any,
            ignore_read_before_assign: false,
        }
    }
}

impl PreferConstOptions {
    /// Options from `linter.ruleOptions["vue/prefer-const"]`.
    pub(super) fn from_config(config: &ConfigPreferConst) -> Self {
        Self {
            destructuring: match config.destructuring {
                Some(PreferConstDestructuring::All) => Destructuring::All,
                Some(PreferConstDestructuring::Any) | None => Destructuring::Any,
            },
            ignore_read_before_assign: config.ignore_read_before_assign.unwrap_or(false),
        }
    }
}

/// `. Allowed unused vars must match /^_/u`, without the leading period.
pub(super) fn allowance(kind: &str, pattern: Option<&Pattern>) -> String {
    let Some(pattern) = pattern else {
        return String::default();
    };
    cstr!(" Allowed unused {kind} must match /{}/u", pattern.source())
}

/// The `reportUsedIgnorePattern` suffix.
pub(super) fn used_allowance(kind: &str, pattern: &Pattern) -> String {
    cstr!(" Used {kind} must not match /{}/u", pattern.source())
}
