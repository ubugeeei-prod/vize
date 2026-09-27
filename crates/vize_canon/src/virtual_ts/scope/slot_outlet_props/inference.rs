//! Export slot payloads from the lexical scopes that infer their types.

use vize_carton::{CompactString, String, append, cstr};
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

/// One entry of an inferred slots type: a dynamic-name outlet stands alone,
/// while every static outlet of one name shares a single entry.
enum OutletEntry {
    Dynamic(u32),
    Static {
        name: CompactString,
        starts: Vec<u32>,
    },
}

impl SlotOutletChecks {
    /// The outlets whose payloads the scope at `scope_id` returns, in source
    /// order, with same-named static outlets merged into one entry.
    ///
    /// Emitting one `{ name?: (props) => any }` per outlet intersects same-named
    /// entries into an overloaded slot function, and the parent's payload
    /// inference then reads only the last overload: which outlet it was
    /// depended on template order, and a bare `<slot name="x" />` after a
    /// bound one erased every prop the parent destructures.
    fn outlet_entries(&self, summary: &Croquis, scope_id: Option<ScopeId>) -> Vec<OutletEntry> {
        let mut outlets: Vec<_> = self.by_scope.values().flatten().collect();
        outlets.sort_by_key(|outlet| outlet.start);
        let mut entries: Vec<OutletEntry> = Vec::new();
        for outlet in outlets {
            if boundary(summary, ScopeId::new(outlet.scope_id)) != scope_id {
                continue;
            }
            if outlet.name_is_dynamic {
                entries.push(OutletEntry::Dynamic(outlet.start));
                continue;
            }
            let existing = entries.iter_mut().find_map(|entry| match entry {
                OutletEntry::Static { name, starts } if *name == outlet.name => Some(starts),
                _ => None,
            });
            match existing {
                Some(starts) => starts.push(outlet.start),
                None => entries.push(OutletEntry::Static {
                    name: outlet.name.clone(),
                    starts: vec![outlet.start],
                }),
            }
        }
        entries
    }

    /// Whether any inferred slots type merges same-named outlets, which is when
    /// `__VizeSlotOutletUnion` is referenced and must be declared.
    pub(super) fn merges_static_outlets(&self, summary: &Croquis) -> bool {
        if !self.infers_slots() {
            return false;
        }
        let mut boundaries: Vec<Option<ScopeId>> = self
            .by_scope
            .keys()
            .map(|scope| boundary(summary, ScopeId::new(*scope)))
            .collect();
        boundaries.sort_unstable();
        boundaries.dedup();
        boundaries.into_iter().any(|scope_id| {
            self.outlet_entries(summary, scope_id).iter().any(
                |entry| matches!(entry, OutletEntry::Static { starts, .. } if starts.len() > 1),
            )
        })
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

    /// The template scope returns the values of `record` (what a generic
    /// component forwards to its root, the ref instances it instantiates) next
    /// to the slots it infers, each under its own key.
    pub(in crate::virtual_ts::scope) fn emit_root_result(
        &self,
        ts: &mut String,
        summary: &Croquis,
        record: &[(&str, String)],
    ) {
        if record.is_empty() {
            return self.emit_result(ts, summary, None, "  ");
        }
        ts.push_str("  return { ");
        if self.infers_slots() {
            append!(
                *ts,
                "{}: {{}} as {}, ",
                crate::virtual_ts::scope::SLOTS_RETURN_KEY,
                self.result_type(summary, None)
            );
        }
        for (key, value) in record {
            append!(*ts, "{key}: {value}, ");
        }
        ts.push_str("};\n");
    }

    fn result_type(&self, summary: &Croquis, scope_id: Option<ScopeId>) -> String {
        let mut types = Vec::new();
        for entry in self.outlet_entries(summary, scope_id) {
            let mut slot = String::default();
            match entry {
                OutletEntry::Dynamic(start) => {
                    append!(
                        slot,
                        "{{ [K in NonNullable<typeof __vize_slot_name_{start}>]?: (props: typeof __vize_slot_payload_{start}) => any }}"
                    );
                }
                OutletEntry::Static { name, starts } => {
                    slot.push_str("{ ");
                    push_ts_string_literal(&mut slot, name.as_str());
                    slot.push_str("?: (props: ");
                    if let [start] = starts.as_slice() {
                        append!(slot, "typeof __vize_slot_payload_{start}");
                    } else {
                        // Every outlet of the name can render the slot, so its
                        // payload is what all of them agree on plus what only
                        // some of them pass, the latter optional.
                        slot.push_str("__VizeSlotOutletUnion<");
                        for (index, start) in starts.iter().enumerate() {
                            if index > 0 {
                                slot.push_str(" | ");
                            }
                            append!(slot, "typeof __vize_slot_payload_{start}");
                        }
                        slot.push('>');
                    }
                    slot.push_str(") => any }");
                }
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
