//! Resolve an actual configured instance without legacy parsing or semantics.

use super::{Linter, builtin_script_rule_entry, resolved_rule};
use crate::rules::script::ScriptRule;

pub(crate) fn configured<'o>(linter: &'o Linter, name: &str) -> Option<&'o dyn ScriptRule> {
    let entry = builtin_script_rule_entry(name)?;
    let rule = resolved_rule(linter, entry);
    (rule.meta().name == entry.rule_name).then_some(rule)
}
