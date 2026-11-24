//! Component asset preamble collection and emission.

use alloc::vec::Vec as StdVec;

use vize_l2::op::ComponentOp;

use super::{Buf, EmitCx, builtin};
use vize_l2::op::vue::asset_identifier;

/// Record `component` as a `resolveComponent` asset, in `generate_assets`
/// order: first occurrence wins, reserved and dynamic tags never resolve.
///
/// The one home of the rule. The emitter applies it inside the helper
/// preference walk (`helper_preference`), after the component's children,
/// so the preamble costs no walk of its own.
pub(in crate::targets::dom::artifact) fn note<'a>(
    component: &ComponentOp<'a>,
    source: &str,
    names: &mut StdVec<&'a str>,
) {
    if !builtin::is_reserved_name(component, source)
        && !builtin::is_dynamic_component(component, source)
        && !names.contains(&component.name)
    {
        names.push(component.name);
    }
}

/// `generate_assets` over the components: tags that resolve to a script
/// binding are skipped (they are pushed as `$setup["Name"]` at the call).
/// Returns whether any `resolveComponent` line was written.
pub(in crate::targets::dom::artifact) fn emit_resolves(
    cx: &mut EmitCx<'_>,
    names: &[&str],
) -> bool {
    let mut resolved = false;
    for name in names {
        if super::binding::resolves(cx, name) {
            continue;
        }
        if !resolved {
            cx.buf.use_resolve_component();
            resolved = true;
        }
        cx.buf.push("const ");
        cx.buf.push(asset_identifier("component", name).as_str());
        cx.buf.push(" = ");
        cx.buf.push(Buf::resolve_component_alias());
        cx.buf.push("(\"");
        cx.buf.push(name);
        cx.buf.push("\"");
        if cx
            .component_name
            .is_some_and(|own| is_self_reference(name, own))
        {
            cx.buf.push(", true");
        }
        cx.buf.push(")");
        cx.buf.newline();
    }
    resolved
}

/// `is_self_component_reference`: the tag verbatim, or its PascalCased
/// spelling, equals the component's own name.
fn is_self_reference(component: &str, own: &str) -> bool {
    if component == own {
        return true;
    }
    vize_l0::capitalize(&vize_l0::camelize(component))
        == vize_l0::capitalize(&vize_l0::camelize(own))
}
