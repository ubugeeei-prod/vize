//! The Vue 2 template-sugar desugars, mirrored from the shipped
//! `desugar_legacy_template` (P2-9 series 7; split from
//! [`super`] under the source budget): `.sync` expansion and the
//! `slot-scope`/`scope` conversion, in the shipped order (sync first,
//! products appended after the authored bindings).

use vize_carton::{Box, String, Vec, cstr};
use vize_sinopia::Element;

use vize_disegno::expr::ExprRef;
use vize_disegno::op::{BindingOp, DynamicName, OnOp};

use super::super::binding::slot_content_params;
use super::super::cx::{Cx, attr_slice, attr_span};
use super::super::directive::{Arg, AttrForm, Head};
use super::super::element::{Analyzed, attr_value_text};

/// One `.sync` expansion the element-level desugar planned: the bind's
/// position and the pieces of the listener it appends.
struct SyncExpansion<'a> {
    /// The authored attribute's index (the bind op to strip `sync` from
    /// is the one at this attribute's span).
    index: usize,
    /// The static prop name (`foo` of `:foo.sync`).
    name: &'a str,
    /// The authored value text, verbatim (the live desugar embeds the
    /// parser's content unchanged).
    value: &'a str,
}

/// Mirror of the shipped `desugar_sync_modifiers`, applied after the
/// element's authored bindings lowered: every `:foo.sync="bar"` strips
/// its `sync` modifier and appends an `@update:foo="$event => ((bar) =
/// $event)"` listener — the exact handler shape the live expansion
/// emits. A dynamic-argument or valueless `.sync` is left untouched,
/// exactly as the live desugar leaves it.
pub(crate) fn desugar_sync<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    bindings: &mut Vec<'a, BindingOp<'a>>,
) {
    if !cx.legacy.scoped_slot_attrs {
        return;
    }
    let mut expansions: alloc::vec::Vec<SyncExpansion<'a>> = alloc::vec::Vec::new();
    for (index, form) in analyzed.forms.iter().enumerate() {
        let AttrForm::Directive(directive) = form else {
            continue;
        };
        if directive.head != Head::Bind || !directive.modifiers.contains(&"sync") {
            continue;
        }
        let Some(Arg::Static(name)) = directive.arg else {
            continue;
        };
        let value = match attr_value_text(element, index) {
            // An authored-blank value materializes no expression in the
            // shipped parser, and the live desugar skips an
            // expressionless `.sync` — mirrored.
            Some(text) if text.trim().is_empty() => continue,
            Some(text) => text,
            // No value position at all: the shipped parser's Vue 3.4
            // same-name shorthand fills the expression with the
            // camelized argument *before* the live desugar reads it, so
            // `:foo-bar.sync` expands with value `fooBar` — mirrored.
            None => {
                let camel = vize_carton::camelize(name);
                if camel.as_str() == name {
                    name
                } else {
                    cx.allocator.alloc_str(camel.as_str())
                }
            }
        };
        expansions.push(SyncExpansion { index, name, value });
    }
    for expansion in expansions {
        let attr = &element.open.attrs[expansion.index];
        let span = attr_span(cx, attr);
        // Strip the first `sync` modifier from the lowered bind op (the
        // op at this attribute's span), mirroring the in-place removal.
        for binding in bindings.iter_mut() {
            if let BindingOp::Bind(bind) = binding
                && bind.span == span
                && let Some(position) = bind.modifiers.iter().position(|m| *m == "sync")
            {
                let mut kept: Vec<'a, &'a str> = Vec::new_in(&cx.allocator);
                for (i, modifier) in bind.modifiers.iter().enumerate() {
                    if i != position {
                        kept.push(modifier);
                    }
                }
                bind.modifiers = kept;
                break;
            }
        }
        let node = cx.mint_op();
        let event = cx
            .allocator
            .alloc_str(cstr!("update:{}", expansion.name).as_str());
        let handler_text = cx
            .allocator
            .alloc_str(cstr!("$event => (({}) = $event)", expansion.value).as_str());
        let handler = ExprRef::parse_js_in(cx.allocator, handler_text, span);
        cx.record(
            "normalize.legacy.sync",
            node,
            attr_slice(cx, attr),
            cstr!("ui.on \"{event}\""),
            span,
        );
        bindings.push(BindingOp::On(Box::new_in(
            OnOp {
                name: Some(DynamicName::Static(event)),
                modifiers: Vec::new_in(&cx.allocator),
                handler: Some(handler),
                span,
            },
            &cx.allocator,
        )));
    }
}

/// The consumed surface of one `slot-scope`/`scope` desugar.
pub(crate) struct ScopedSlotPlan<'a> {
    /// The scoped-slot attribute's index (`slot-scope` preferred by
    /// position; `scope` is the 2.1 alias — the live desugar's first
    /// match wins).
    pub scope_idx: usize,
    /// The companion `slot="name"` attribute's index, when present —
    /// consumed whether or not it carries a value, exactly as the live
    /// desugar removes it.
    pub slot_idx: Option<usize>,
    /// The target slot name (`None` — the implicit default — when the
    /// companion attribute is absent or valueless).
    pub name: Option<&'a str>,
    /// The slot-props expression text (`None` for a valueless or blank
    /// spelling — the `ui.slot-content` params rule).
    pub params: Option<&'a str>,
}

/// Mirror of the shipped `desugar_scoped_slot_attrs`' recognition: the
/// first `slot-scope`/`scope` attribute converts, its companion `slot`
/// attribute is consumed as the name, and an element already carrying a
/// `v-slot` spelling is left alone (the malformed old-and-new mix).
pub(crate) fn scoped_slot_plan<'a>(
    cx: &Cx<'a>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
) -> Option<ScopedSlotPlan<'a>> {
    if !cx.legacy.scoped_slot_attrs {
        return None;
    }
    let is_static = |index: usize| matches!(analyzed.forms[index], AttrForm::Static);
    let scope_idx = element
        .open
        .attrs
        .iter()
        .enumerate()
        .position(|(i, attr)| is_static(i) && matches!(attr.name.text, "slot-scope" | "scope"))?;
    if analyzed
        .forms
        .iter()
        .any(|form| matches!(form, AttrForm::Directive(directive) if directive.head == Head::Slot))
    {
        return None;
    }
    let slot_idx = element
        .open
        .attrs
        .iter()
        .enumerate()
        .position(|(i, attr)| is_static(i) && attr.name.text == "slot");
    let name = slot_idx
        .and_then(|i| attr_value_text(element, i))
        .filter(|text| !text.is_empty());
    let params = attr_value_text(element, scope_idx).filter(|text| !text.trim().is_empty());
    Some(ScopedSlotPlan {
        scope_idx,
        slot_idx,
        name,
        params,
    })
}

/// Whether the plan consumes the attribute at `index` (skipped by the
/// owner's attribute loop instead of lowering as a plain attribute).
pub(crate) fn consumed_by_scoped_slot(plan: &Option<ScopedSlotPlan<'_>>, index: usize) -> bool {
    plan.as_ref()
        .is_some_and(|plan| plan.scope_idx == index || plan.slot_idx == Some(index))
}

/// Mirror of the shipped desugar's construction: the consumed spelling
/// becomes an appended `ui.slot-content` binding — name from the
/// companion attribute, params from the scoped-slot value through the
/// same one params/scope rule every `v-slot` spelling uses.
pub(crate) fn desugar_scoped_slot<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    plan: &ScopedSlotPlan<'a>,
    bindings: &mut Vec<'a, BindingOp<'a>>,
) {
    let scope_attr = &element.open.attrs[plan.scope_idx];
    let span = attr_span(cx, scope_attr);
    if let Some(slot_idx) = plan.slot_idx {
        let slot_attr = &element.open.attrs[slot_idx];
        cx.record(
            "consume.legacy.slot-name",
            None,
            attr_slice(cx, slot_attr),
            String::default(),
            attr_span(cx, slot_attr),
        );
    }
    let node = cx.mint_op();
    let name = plan.name.map(DynamicName::Static);
    let after = match plan.name {
        None => String::from("ui.slot-content"),
        Some(text) => cstr!("ui.slot-content \"{text}\""),
    };
    cx.record(
        "normalize.legacy.slot-scope",
        node,
        attr_slice(cx, scope_attr),
        after,
        span,
    );
    let params = plan.params.map(|text| slot_content_params(cx, node, text));
    bindings.push(BindingOp::SlotContent(Box::new_in(
        vize_disegno::op::SlotContentOp {
            name,
            modifiers: Vec::new_in(&cx.allocator),
            params,
            span,
        },
        &cx.allocator,
    )));
}
