//! Component asset preamble collection and emission.

use alloc::vec::Vec as StdVec;

use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{Op, Region};

use super::{Buf, EmitCx, asset_ident, builtin};

pub(in crate::emit) fn collect_names<'a>(root: &Region<'a>) -> StdVec<&'a str> {
    let mut names = StdVec::new();
    collect_from(root, &mut names);
    names
}

/// `generate_assets` over the components: tags that resolve to a script
/// binding are skipped (they are pushed as `$setup.Name` at the call).
/// Returns whether any `resolveComponent` line was written.
pub(in crate::emit) fn emit_resolves(cx: &mut EmitCx<'_>, names: &[&str]) -> bool {
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
        cx.buf.push(asset_ident("component", name).as_str());
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
    vize_s0::capitalize(&vize_s0::camelize(component)).as_str() == own
}

fn collect_from<'a>(region: &Region<'a>, names: &mut StdVec<&'a str>) {
    ensure_sufficient_stack(|| collect_from_guarded(region, names));
}

fn collect_from_guarded<'a>(region: &Region<'a>, names: &mut StdVec<&'a str>) {
    for op in region.ops.iter() {
        match op {
            Op::Element(element) => collect_from(&element.children, names),
            Op::Component(component) => {
                collect_from(&component.children, names);
                if !builtin::is_reserved_name(component.name)
                    && !builtin::is_dynamic_component(component)
                    && !names.contains(&component.name)
                {
                    names.push(component.name);
                }
            }
            Op::Slot(slot) => collect_from(&slot.fallback, names),
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    collect_from(&branch.region, names);
                }
            }
            Op::For(for_op) => collect_from(&for_op.region, names),
            _ => {}
        }
    }
}
