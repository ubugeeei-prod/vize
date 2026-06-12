//! vue/no-undefined-refs
//!
//! Disallow undefined variable references in templates.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::cstr;
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-undefined-refs",
    description: "Disallow undefined variable references in templates",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No undefined refs rule.
#[derive(Default)]
pub struct NoUndefinedRefs;

impl Rule for NoUndefinedRefs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let Some(analysis) = ctx.analysis() else {
            return;
        };

        let undefined_refs: Vec<_> = analysis
            .undefined_refs
            .iter()
            .map(|undefined| {
                (
                    undefined.name.clone(),
                    undefined.offset,
                    undefined.offset + undefined.name.len() as u32,
                )
            })
            .collect();

        for (name, start, end) in undefined_refs {
            ctx.report(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    cstr!("Variable '{name}' is not defined"),
                    start,
                    end,
                )
                .with_help("Define in <script setup> or ensure it's imported"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoUndefinedRefs;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoUndefinedRefs;
        assert_eq!(rule.meta().name, "vue/no-undefined-refs");
        assert_eq!(rule.meta().category, RuleCategory::Recommended);
    }
}
