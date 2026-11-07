//! vue/no-unused-vars
//!
//! Disallow unused variable definitions in `v-for` and `v-slot` directives.
//!
//! This rule reports variables that are defined in `v-for` or `v-slot` directives
//! but never used in the template. Uses croquis semantic analysis for accurate
//! tracking of variable usage.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <!-- 'index' is defined but never used -->
//! <li v-for="(item, index) in items" :key="item.id">{{ item }}</li>
//!
//! <!-- 'foo' is defined but never used -->
//! <template v-slot="{ foo }">
//!   <span>Hello</span>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <li v-for="(item, index) in items" :key="index">{{ item }}</li>
//! <li v-for="item in items" :key="item.id">{{ item.name }}</li>
//!
//! <template v-slot="{ data }">
//!   <span>{{ data }}</span>
//! </template>
//! ```
//!
//! ## Options
//!
//! An ignore pattern can be configured explicitly; the default matches
//! eslint-plugin-vue and reports underscore-prefixed aliases too.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::UnusedVarContext;
use vize_l0::String;
use vize_l0::cstr;
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-vars",
    description: "Disallow unused variable definitions in v-for and v-slot directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow unused v-for and v-slot variables
#[derive(Default)]
pub struct NoUnusedVars {
    /// Optional pattern for variables to ignore.
    ignore_pattern: Option<String>,
}

impl NoUnusedVars {
    /// Check if a variable name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        // Check custom ignore pattern
        if let Some(pattern) = &self.ignore_pattern {
            if pattern == "^_" {
                return name.starts_with('_');
            }
            if name.starts_with(pattern.as_str()) {
                return true;
            }
        }

        false
    }
}

impl Rule for NoUnusedVars {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // Skip if no analysis available (croquis semantic analysis required)
        if !ctx.has_analysis() {
            return;
        }

        let Some(analysis) = ctx.analysis() else {
            return;
        };
        let unused_vars = analysis.unused_template_vars();

        for unused in unused_vars {
            // Skip if the variable should be ignored
            if self.should_ignore(unused.name.as_str()) {
                continue;
            }

            let (message, help) = match &unused.context {
                UnusedVarContext::VForValue => (
                    cstr!(
                        "Variable '{}' is defined by v-for but never used",
                        unused.name
                    ),
                    "If the variable is intentionally unused, prefix it with underscore: _item",
                ),
                UnusedVarContext::VForKey => (
                    cstr!(
                        "Key variable '{}' is defined by v-for but never used",
                        unused.name
                    ),
                    "Consider removing the key variable or prefix it with underscore: _key",
                ),
                UnusedVarContext::VForIndex => (
                    cstr!(
                        "Index variable '{}' is defined by v-for but never used",
                        unused.name
                    ),
                    "Consider removing the index variable or prefix it with underscore: _index",
                ),
                UnusedVarContext::VSlot { slot_name } => (
                    cstr!(
                        "Slot prop '{}' from slot '{}' is defined but never used",
                        unused.name,
                        slot_name
                    ),
                    "Consider removing unused slot props or prefix with underscore",
                ),
            };

            ctx.report(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    message,
                    unused.offset,
                    unused.offset + unused.name.len() as u32,
                )
                .with_help(help),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoUnusedVars;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoUnusedVars::default();
        assert_eq!(rule.meta().name, "vue/no-unused-vars");
        assert_eq!(rule.meta().category, RuleCategory::Essential);
    }

    #[test]
    fn test_default_does_not_ignore_underscore_prefix() {
        let rule = NoUnusedVars::default();
        assert!(!rule.should_ignore("_item"));
        assert!(!rule.should_ignore("_"));
        assert!(!rule.should_ignore("item"));
        assert!(!rule.should_ignore("index"));
    }
}
