//! Reporting and analysis output for the reactivity tracker.
//!
//! Provides the [`to_markdown`](super::ReactivityTracker::to_markdown) method
//! that renders a human-readable Markdown report of tracked bindings and violations.

use super::tracker::ReactivityTracker;
use super::{BindingState, ViolationKind, ViolationSeverity};
use vize_carton::append;
use vize_carton::String;

impl ReactivityTracker {
    /// Generate markdown report.
    pub fn to_markdown(&self) -> String {
        let mut md = String::default();

        md.push_str("# Reactivity Analysis Report\n\n");

        // Summary
        let error_count = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Error)
            .count();
        let warning_count = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Warning)
            .count();
        let info_count = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Info)
            .count();

        md.push_str("## Summary\n\n");
        append!(md, "- **Tracked Bindings**: {}\n", self.bindings.len());
        append!(
            md,
            "- **Violations**: {error_count} errors, {warning_count} warnings, {info_count} info\n\n",
        );

        // Bindings table
        if !self.bindings.is_empty() {
            md.push_str("## Tracked Reactive Bindings\n\n");
            md.push_str("| Name | Origin | State | Scope |\n");
            md.push_str("|------|--------|-------|-------|\n");

            for binding in self.bindings.values() {
                let state = match binding.state {
                    BindingState::Active => "✓ Active",
                    BindingState::ReactivityLost => "✗ Lost",
                    BindingState::Moved => "→ Moved",
                    BindingState::Escaped => "↗ Escaped",
                    BindingState::Reassigned => "⟲ Reassigned",
                };
                append!(
                    md,
                    "| `{}` | {:?} | {} | {} |\n",
                    binding.name,
                    binding.origin,
                    state,
                    binding.scope_depth
                );
            }
            md.push('\n');
        }

        // Violations
        if !self.violations.is_empty() {
            md.push_str("## Violations\n\n");

            for violation in &self.violations {
                let icon = match violation.severity {
                    ViolationSeverity::Error => "❌",
                    ViolationSeverity::Warning => "⚠️",
                    ViolationSeverity::Info => "ℹ️",
                    ViolationSeverity::Hint => "💡",
                };

                append!(md, "### {icon} {}\n\n", violation.message);
                append!(
                    md,
                    "**Location**: offset {}..{}\n\n",
                    violation.start,
                    violation.end
                );

                if let Some(ref suggestion) = violation.suggestion {
                    append!(md, "**Suggestion**: {suggestion}\n\n");
                }

                // Add detailed explanation for specific violation kinds
                match &violation.kind {
                    ViolationKind::DestructuringLoss { extracted_props } => {
                        md.push_str("```\n");
                        md.push_str("// ❌ Reactivity is lost:\n");
                        append!(
                            md,
                            "const {{ {} }} = reactiveObj\n",
                            extracted_props.join(", ")
                        );
                        md.push_str("\n// ✓ Keep reactivity:\n");
                        append!(
                            md,
                            "const {{ {} }} = toRefs(reactiveObj)\n",
                            extracted_props.join(", ")
                        );
                        md.push_str("```\n\n");
                    }
                    ViolationKind::SpreadLoss => {
                        md.push_str("```\n");
                        md.push_str("// ❌ Creates non-reactive copy:\n");
                        md.push_str("const copy = { ...reactiveObj }\n");
                        md.push_str("\n// ✓ If intentional, use toRaw:\n");
                        md.push_str("const copy = { ...toRaw(reactiveObj) }\n");
                        md.push_str("```\n\n");
                    }
                    _ => {}
                }
            }
        }

        md
    }
}
