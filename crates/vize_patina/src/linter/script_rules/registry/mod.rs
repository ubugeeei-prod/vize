//! The growable built-in script-rule registry.
//!
//! All per-rule data lives in this module so a new `script/*` (or `ecosystem/*`)
//! rule can be added by editing only small, data-only files here, leaving the
//! larger dispatch logic in `script_rules.rs` untouched. Each file is kept well
//! under the repository source-length guard so it has room to grow.
//!
//! ## Adding a script rule
//!
//! 1. Implement the rule in `crate::rules::script`.
//! 2. In `names.rs`, add `pub(crate) const RULE_<NAME>: &str = "<cat>/<kebab>";`
//!    and append the name to `ALL_BUILTIN_SCRIPT_RULE_NAMES` (and, unless it is
//!    one of the first three always-on rules, to the test-only
//!    `OPT_IN_SCRIPT_RULE_NAMES`).
//! 3. In `rules.rs`, import the rule type, add a configured `static <NAME>_RULE`
//!    instance if it needs parameters, and append a [`BuiltinScriptRuleEntry`]
//!    to `BUILTIN_SCRIPT_RULES` in the same order.
//!
//! `BUILTIN_SCRIPT_RULES` is a `&'static [_]` slice, so no count constant has to
//! be bumped when a rule is added.

mod names;
mod rules;

pub(in crate::linter::script_rules) use names::ALL_BUILTIN_SCRIPT_RULE_NAMES;
#[cfg(test)]
pub(in crate::linter::script_rules) use names::OPT_IN_SCRIPT_RULE_NAMES;
pub(crate) use names::{
    RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};
pub(in crate::linter::script_rules) use rules::BUILTIN_SCRIPT_RULES;

use crate::rules::script::ScriptRule;

/// A built-in script rule paired with its registry name and profiling label.
///
/// AST-based rules share a single oxc parse per script block. Byte-based rules
/// run directly against the source via [`ScriptRule::check`].
pub(in crate::linter::script_rules) struct BuiltinScriptRuleEntry {
    pub(in crate::linter::script_rules) rule_name: &'static str,
    pub(in crate::linter::script_rules) profile_name: &'static str,
    pub(in crate::linter::script_rules) category: &'static str,
    pub(in crate::linter::script_rules) fixable: bool,
    pub(in crate::linter::script_rules) presets: &'static [&'static str],
    pub(in crate::linter::script_rules) rule: &'static (dyn ScriptRule + 'static),
}

impl BuiltinScriptRuleEntry {
    #[inline]
    pub(in crate::linter::script_rules) fn meta(&self) -> BuiltinScriptRuleMeta {
        let meta = self.rule.meta();
        BuiltinScriptRuleMeta {
            name: meta.name,
            description: meta.description,
            category: self.category,
            fixable: self.fixable,
            default_severity: meta.default_severity,
            presets: self.presets,
        }
    }
}

pub struct BuiltinScriptRuleMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub fixable: bool,
    pub default_severity: crate::Severity,
    pub presets: &'static [&'static str],
}

const OPINIONATED_SCRIPT_PRESETS: &[&str] = &["opinionated", "nuxt"];
const ECOSYSTEM_SCRIPT_PRESETS: &[&str] = &["ecosystem"];
const OPT_IN_SCRIPT_PRESETS: &[&str] = &[];
