//! Resolve an actual configured instance without legacy parsing or semantics.

use super::registry::{RULE_NO_GET_CURRENT_INSTANCE, RULE_NUXT_CONFIG_KEYS_ORDER};
use super::{Linter, builtin_script_rule_entry, resolved_rule};
use crate::rules::script::ScriptRule;

pub(crate) fn configured<'o>(linter: &'o Linter, name: &str) -> Option<&'o dyn ScriptRule> {
    let entry = builtin_script_rule_entry(name)?;
    let rule = resolved_rule(linter, entry);
    (rule.meta().name == entry.rule_name).then_some(rule)
}

/// These genuine catalog keys have original filename/activation policy that
/// this new host has not authenticated. Offered callbacks cannot bypass it.
pub(crate) fn requires_invocation_policy(instance: &dyn ScriptRule) -> bool {
    matches!(
        instance.meta().name,
        RULE_NUXT_CONFIG_KEYS_ORDER | RULE_NO_GET_CURRENT_INSTANCE
    )
}
