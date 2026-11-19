//! Actual inline Vapor Model helpers, with original lexical write custody.

use super::{VaporError, VaporErrorKind, error, setup::helper};
use crate::{
    runtime::{Runtime, vocabulary},
    write::{LinkSink, Writer},
};
use vize_l0::{String, cstr, id::NodeId};
use vize_l2::{file::NativeFileModelKind, lang::js::NativeSelectedSetup, op::ElementOp};
use vize_l3::decision::vapor::{NativeSelectedSetupVaporAnalysis, VaporModel};

pub(super) fn write<'owner, 'arena, L: LinkSink>(
    prelude: &mut Writer<L>,
    body: &mut Writer<L>,
    analysis: &NativeSelectedSetupVaporAnalysis<'_, 'owner, 'arena>,
    node: NodeId,
    element: &'owner ElementOp<'arena>,
    model: VaporModel<'owner, 'arena>,
) -> Result<(), VaporError> {
    let fail = || error(VaporErrorKind::ForeignRoot);
    let original = model.original().ok_or_else(fail)?;
    let kind = original.kind().ok_or_else(fail)?;
    let resolution = model.resolution();
    let occurrence = model.occurrence().ok_or_else(fail)?;
    let authored = model.authored_span().ok_or_else(fail)?;
    let binding = model.binding().ok_or_else(fail)?;
    if !core::ptr::eq(resolution.file(), analysis.file())
        || !original.accepts_element(node, element)
        || analysis.setup().binding(binding).is_err()
        || !resolution.accepts(binding)
        || analysis.vapor().and_then(|facts| facts.inherit_attrs()) != Some(node)
        || !analysis
            .tables()
            .nodes
            .get(node)
            .is_some_and(|row| row.dynamic_bindings.as_slice() == [resolution.node()])
    {
        return Err(fail());
    }
    let runtime = vocabulary(Runtime::VueVapor);
    let helpers = fresh(analysis.setup(), "__vizeModelRuntime")?;
    let value = fresh(analysis.setup(), "__vizeModelValue")?;
    let apply = match kind {
        NativeFileModelKind::Text => "applyTextModel",
        NativeFileModelKind::Checkbox => "applyCheckboxModel",
    };
    for name in [apply, "unref", "isRef", "template"] {
        helper(prelude, runtime, name)?;
    }
    // Import aliases are resolved outside the original inline setup scope.
    // Only these fresh generated identifiers enter that scope; an authored
    // `_isRef`, `_applyTextModel` or callback-looking name cannot shadow them.
    prelude.push("const ");
    prelude.push(helpers.as_str());
    prelude.push(" = { apply: _");
    prelude.push(apply);
    prelude.push(", read: _unref, isRef: _isRef }\n");
    if kind == NativeFileModelKind::Checkbox {
        let value = analysis
            .file()
            .native_attribute_value_for(0, element, 0)
            .ok_or_else(fail)?;
        let observation = value.observation().ok_or_else(fail)?;
        prelude.push("const t0 = _template(\"<input ");
        prelude.push_named("type", observation.name_span(), "type");
        prelude.push("=\\\"");
        prelude.push_named("checkbox", observation.source().span(), "checkbox");
        prelude.push("\\\">\", 1)\n");
    } else {
        prelude.push("const t0 = _template(\"<input>\", 1)\n");
    }
    body.newline();
    body.anchor(element.span.start);
    body.push("const n0 = t0()");
    body.newline();
    body.push(helpers.as_str());
    body.push(".apply(n0, () => ");
    body.push(helpers.as_str());
    body.push(".read(");
    body.push_named(occurrence.name, authored, occurrence.name);
    body.push("), ");
    body.push(value.as_str());
    body.push(" => (");
    body.push(helpers.as_str());
    body.push(".isRef(");
    body.push_named(occurrence.name, authored, occurrence.name);
    body.push(") ? (");
    body.push_named_parts([occurrence.name, ".value"], authored, occurrence.name);
    body.push(" = ");
    body.push(value.as_str());
    body.push(") : (");
    body.push_named(occurrence.name, authored, occurrence.name);
    body.push(" = ");
    body.push(value.as_str());
    body.push("))");
    let operand = original.input().operand();
    let modifier = operand
        .trim_span()
        .map(|span| ("trim", span))
        .or_else(|| operand.lazy_span().map(|span| ("lazy", span)))
        .or_else(|| operand.number_span().map(|span| ("number", span)));
    if let Some((name, span)) = modifier {
        body.push(", { ");
        body.push_named(name, span, name);
        body.push(": true }");
    }
    body.push(")");
    Ok(())
}

fn fresh(setup: &NativeSelectedSetup<'_, '_>, base: &str) -> Result<String, VaporError> {
    let occupied = |name: &str| {
        setup
            .bindings()
            .any(|binding| binding.declaration().is_some_and(|row| row.name == name))
    };
    if !occupied(base) {
        return Ok(base.into());
    }
    // N original declarations occupy at most N-1 suffixed alternatives once
    // the base is occupied. Membership, not hash iteration, selects the name.
    (1..=setup.bindings().count())
        .find_map(|suffix| {
            let candidate = cstr!("{base}{suffix}");
            (!occupied(candidate.as_str())).then_some(candidate)
        })
        .ok_or_else(|| error(VaporErrorKind::GeneratedBindingCollision))
}
