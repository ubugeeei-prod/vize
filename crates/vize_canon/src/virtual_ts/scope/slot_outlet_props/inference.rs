//! Export slot payloads from the lexical scopes that infer their types.

use vize_carton::{String, append, cstr};
use vize_croquis::{Croquis, ScopeId, ScopeKind};
use vize_relief::{RootNode, TemplateChildNode};

use super::SlotOutletChecks;
use crate::virtual_ts::helpers::push_ts_string_literal;

pub(crate) fn has_inferred_slots(summary: &Croquis, root: Option<&RootNode<'_>>) -> bool {
    summary.macros.define_slots().is_none()
        && root.is_some_and(|root| has_outlet(root.children.as_slice()))
}

fn has_outlet(children: &[TemplateChildNode<'_>]) -> bool {
    children.iter().any(|child| match child {
        TemplateChildNode::Element(element) => {
            element.tag == "slot" || has_outlet(element.children.as_slice())
        }
        TemplateChildNode::If(node) => node
            .branches
            .iter()
            .any(|branch| has_outlet(branch.children.as_slice())),
        TemplateChildNode::For(node) => has_outlet(node.children.as_slice()),
        _ => false,
    })
}

fn boundary(summary: &Croquis, mut id: ScopeId) -> Option<ScopeId> {
    loop {
        let scope = summary.scopes.get_scope(id)?;
        if matches!(scope.kind, ScopeKind::VFor | ScopeKind::VSlot) {
            return Some(id);
        }
        id = scope.parent()?;
    }
}

impl SlotOutletChecks {
    pub(super) fn infers_slots(&self) -> bool {
        self.infer && !self.by_scope.is_empty()
    }

    pub(in crate::virtual_ts::scope) fn captures_scope(
        &self,
        summary: &Croquis,
        id: ScopeId,
    ) -> bool {
        self.infers_slots()
            && self.by_scope.keys().any(|outlet_scope| {
                let mut cursor = Some(ScopeId::new(*outlet_scope));
                while let Some(current) = cursor {
                    if current == id {
                        return true;
                    }
                    cursor = summary
                        .scopes
                        .get_scope(current)
                        .and_then(|scope| scope.parent());
                }
                false
            })
    }

    pub(in crate::virtual_ts::scope) fn emit_result(
        &self,
        ts: &mut String,
        summary: &Croquis,
        scope_id: Option<ScopeId>,
        indent: &str,
    ) {
        if !self.infers_slots() {
            return;
        }
        let ty = self.result_type(summary, scope_id);
        if scope_id.is_some_and(|id| {
            summary
                .scopes
                .get_scope(id)
                .is_some_and(|scope| scope.kind == ScopeKind::VFor)
        }) {
            append!(*ts, "{indent}return [{{}} as {ty}];\n");
        } else {
            append!(*ts, "{indent}return {{}} as {ty};\n");
        }
    }

    /// The template scope returns what a generic component forwards to its
    /// root next to the slots it infers, each under its own key.
    pub(in crate::virtual_ts::scope) fn emit_root_result(
        &self,
        ts: &mut String,
        summary: &Croquis,
        forwarded: Option<&str>,
    ) {
        let Some(forwarded) = forwarded else {
            return self.emit_result(ts, summary, None, "  ");
        };
        ts.push_str("  return { ");
        if self.infers_slots() {
            append!(
                *ts,
                "{}: {{}} as {}, ",
                crate::virtual_ts::scope::SLOTS_RETURN_KEY,
                self.result_type(summary, None)
            );
        }
        append!(
            *ts,
            "{}: {{}} as {forwarded} }};\n",
            crate::virtual_ts::scope::FORWARDED_RETURN_KEY
        );
    }

    fn result_type(&self, summary: &Croquis, scope_id: Option<ScopeId>) -> String {
        let mut types = Vec::new();
        let mut outlets: Vec<_> = self.by_scope.values().flatten().collect();
        outlets.sort_by_key(|outlet| outlet.start);
        for outlet in outlets {
            if boundary(summary, ScopeId::new(outlet.scope_id)) != scope_id {
                continue;
            }
            let mut slot = String::default();
            if outlet.name_is_dynamic {
                append!(
                    slot,
                    "{{ [K in NonNullable<typeof __vize_slot_name_{}>]?: (props: typeof __vize_slot_payload_{}) => any }}",
                    outlet.start,
                    outlet.start
                );
            } else {
                slot.push_str("{ ");
                push_ts_string_literal(&mut slot, outlet.name.as_str());
                append!(
                    slot,
                    "?: (props: typeof __vize_slot_payload_{}) => any }}",
                    outlet.start
                );
            }
            types.push(slot);
        }
        for scope in summary.scopes.iter() {
            if !matches!(scope.kind, ScopeKind::VFor | ScopeKind::VSlot)
                || !self.captures_scope(summary, scope.id)
                || scope.parent().and_then(|parent| boundary(summary, parent)) != scope_id
            {
                continue;
            }
            let id = scope.id.as_u32();
            types.push(if matches!(scope.kind, ScopeKind::VFor) {
                cstr!("NonNullable<typeof __vize_slot_scope_{id}[number]>")
            } else {
                cstr!("ReturnType<typeof __vize_slot_scope_{id}>")
            });
        }
        if types.is_empty() {
            String::from("{}")
        } else {
            types.join(" & ").into()
        }
    }
}
