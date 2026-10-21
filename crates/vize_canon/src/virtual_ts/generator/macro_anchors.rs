//! Bindings consumed by the component's public compiler-macro signature.

use super::setup_helpers::SetupHelperPlan;
use vize_carton::{String, append};

pub(super) fn emit_setup_scope_macro_anchors(ts: &mut String, plan: &SetupHelperPlan) {
    if !plan.macro_results.is_empty() {
        ts.push_str("\n  // Compiler-macro results contribute to the component signature\n  ");
        for (index, name) in plan.macro_results.iter().enumerate() {
            if index > 0 {
                ts.push(' ');
            }
            append!(*ts, "void {name};");
        }
        ts.push('\n');
    }
}
