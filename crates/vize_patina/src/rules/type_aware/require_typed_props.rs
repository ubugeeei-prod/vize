//! type/require-typed-props
//!
//! Require type definition for defineProps macro.
//!
//! This rule requires semantic analysis to check if defineProps uses
//! TypeScript type parameter or runtime declaration.
//!
//! ## Examples
//!
//! ### Invalid (no type information)
//! ```vue
//! <script setup>
//! // Runtime-only props without type
//! const props = defineProps(['msg', 'count'])
//! </script>
//! ```
//!
//! ### Valid (with type parameter)
//! ```vue
//! <script setup lang="ts">
//! interface Props {
//!   msg: string
//!   count?: number
//! }
//! const props = defineProps<Props>()
//! </script>
//! ```
//!
//! ### Valid (with withDefaults)
//! ```vue
//! <script setup lang="ts">
//! interface Props {
//!   msg: string
//!   count?: number
//! }
//! const props = withDefaults(defineProps<Props>(), {
//!   count: 0
//! })
//! </script>
//! ```
//!
//! ### Valid (runtime declaration with type annotation)
//! ```vue
//! <script setup>
//! const props = defineProps({
//!   msg: { type: String, required: true },
//!   count: { type: Number, default: 0 }
//! })
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "type/require-typed-props",
    description: "Require type definition for defineProps",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require typed props rule
#[derive(Default)]
pub struct RequireTypedProps {
    /// Whether to allow runtime-only array syntax (e.g., defineProps(['msg']))
    pub allow_array_syntax: bool,
}

impl RequireTypedProps {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow runtime-only array syntax
    pub fn allow_array_syntax(mut self) -> Self {
        self.allow_array_syntax = true;
        self
    }
}

impl Rule for RequireTypedProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            return;
        }

        let analysis = ctx.analysis().unwrap();
        let props = analysis.macros.props();

        // Check if there are any props
        if props.is_empty() {
            return;
        }

        // Find the defineProps macro call to check if it has type args
        let define_props_call = analysis
            .macros
            .all_calls()
            .iter()
            .find(|c| matches!(c.kind, vize_croquis::macros::MacroKind::DefineProps));

        // If defineProps has type arguments, all props are typed
        if let Some(call) = define_props_call {
            if call.type_args.is_some() {
                return;
            }
        }

        // Check each prop for runtime type information
        for prop in props {
            // Skip if runtime type is specified
            if prop.prop_type.is_some() {
                continue;
            }

            // Allow array syntax if configured
            if self.allow_array_syntax {
                continue;
            }

            // Get position from the defineProps call if available
            if let Some(call) = define_props_call {
                ctx.report(
                    crate::diagnostic::LintDiagnostic::warn(
                        ctx.current_rule,
                        "Prop should have a type definition",
                        call.start,
                        call.end,
                    )
                    .with_help("Use TypeScript type parameter: defineProps<{ msg: string }>()"),
                );
                // Only report once per defineProps call
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequireTypedProps;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = RequireTypedProps::default();
        assert_eq!(rule.meta().name, "type/require-typed-props");
        assert_eq!(rule.meta().category, RuleCategory::TypeAware);
    }
}
