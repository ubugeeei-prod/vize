//! Defaults match typescript-eslint `no-unused-vars` and ESLint `prefer-const`.

use regex::Regex;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum Args {
    All,
    AfterUsed,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum Vars {
    All,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum Caught {
    All,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
            source: source.to_string(),
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
}

impl Default for UnusedOptions {
    fn default() -> Self {
        Self {
            args: Args::AfterUsed,
            args_ignore: Pattern::compile("^_"),
            vars: Vars::All,
            vars_ignore: Pattern::compile("^_"),
            caught: Caught::All,
            caught_ignore: Pattern::compile("^_"),
            array_ignore: Pattern::compile("^_"),
            ignore_rest_siblings: false,
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

pub(super) fn allowance(kind: &str, pattern: Option<&Pattern>) -> String {
    let Some(pattern) = pattern else {
        return String::new();
    };
    format!(" Allowed unused {kind} must match /{}/u", pattern.source())
}
