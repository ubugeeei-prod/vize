//! type/require-typed-emits
//!
//! Require type definition for defineEmits macro.
//!
//! This rule requires semantic analysis to check if defineEmits uses
//! TypeScript type parameter or runtime declaration.
//!
//! ## Examples
//!
//! ### Invalid (no type information)
//! ```vue
//! <script setup>
//! // Array-only emits without type
//! const emit = defineEmits(['click', 'update'])
//! </script>
//! ```
//!
//! ### Valid (with type parameter)
//! ```vue
//! <script setup lang="ts">
//! interface Emits {
//!   (e: 'click', value: MouseEvent): void
//!   (e: 'update', value: string): void
//! }
//! const emit = defineEmits<Emits>()
//! </script>
//! ```
//!
//! ### Valid (with call signature type)
//! ```vue
//! <script setup lang="ts">
//! const emit = defineEmits<{
//!   click: [value: MouseEvent]
//!   update: [value: string]
//! }>()
//! </script>
//! ```
//!
//! ### Valid (runtime declaration with validation)
//! ```vue
//! <script setup>
//! const emit = defineEmits({
//!   click: (value) => value instanceof MouseEvent,
//!   update: (value) => typeof value === 'string'
//! })
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "type/require-typed-emits",
    description: "Require type definition for defineEmits",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require typed emits rule
#[derive(Default)]
pub struct RequireTypedEmits {
    /// Whether to allow runtime-only array syntax (e.g., defineEmits(['click']))
    pub allow_array_syntax: bool,
}

impl RequireTypedEmits {
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

impl Rule for RequireTypedEmits {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            return;
        }

        let analysis = ctx.analysis().unwrap();
        let emits = analysis.macros.emits();

        // Check if there are any emits
        if emits.is_empty() {
            return;
        }

        // Find the defineEmits macro call to check if it has type args
        let define_emits_call = analysis
            .macros
            .all_calls()
            .iter()
            .find(|c| matches!(c.kind, vize_croquis::macros::MacroKind::DefineEmits));

        // If defineEmits has type arguments, all emits are typed
        if let Some(call) = define_emits_call {
            if call.type_args.is_some() {
                return;
            }
        }

        // Check each emit for payload type information
        for emit in emits {
            // Skip if payload type is specified
            if emit.payload_type.is_some() {
                continue;
            }

            // Allow array syntax if configured
            if self.allow_array_syntax {
                continue;
            }

            // Get position from the defineEmits call if available
            if let Some(call) = define_emits_call {
                ctx.report(
                    crate::diagnostic::LintDiagnostic::warn(
                        ctx.current_rule,
                        "Emit should have a type definition",
                        call.start,
                        call.end,
                    )
                    .with_help(
                        "Use TypeScript type parameter: defineEmits<{ click: [value: MouseEvent] }>()",
                    ),
                );
                // Only report once per defineEmits call
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequireTypedEmits;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = RequireTypedEmits::default();
        assert_eq!(rule.meta().name, "type/require-typed-emits");
        assert_eq!(rule.meta().category, RuleCategory::TypeAware);
    }
}
