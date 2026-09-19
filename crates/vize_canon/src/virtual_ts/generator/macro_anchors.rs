//! Bindings consumed by the component's public compiler-macro signature.

use super::setup_helpers::SetupHelperPlan;
use vize_carton::{String, append};
use vize_croquis::Croquis;

pub(super) fn emit_setup_scope_macro_anchors(
    ts: &mut String,
    summary: &Croquis,
    plan: &SetupHelperPlan,
) {
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

    if let Some(destructure) = summary.macros.props_destructure()
        && !destructure.bindings.is_empty()
    {
        ts.push_str("\n  // Reference destructured props (prevent TS6133)\n  ");
        let mut first = true;
        for binding in destructure.bindings.values() {
            if !first {
                ts.push(' ');
            }
            append!(*ts, "void {};", binding.local);
            first = false;
        }
        if let Some(ref rest) = destructure.rest_id {
            if !first {
                ts.push(' ');
            }
            append!(*ts, "void {};", rest);
        }
        ts.push('\n');
    }
}
