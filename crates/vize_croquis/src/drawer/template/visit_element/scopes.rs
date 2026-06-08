use crate::drawer::Drawer;
use crate::drawer::helpers::{ConditionalKind, VForScopeAliases};
use crate::scope::{ParamNames, VForScopeData, VSlotScopeData};
use vize_carton::{CompactString, SmallVec};
use vize_relief::ast::ElementNode;

use super::bounds::element_subtree_end;
use super::v_for_scope::v_for_scope_bindings;

pub(super) type SlotScopeInfo = (
    CompactString,
    SmallVec<[CompactString; 4]>,
    Option<CompactString>,
    u32,
);

pub(super) type ForScopeInfo = (
    VForScopeAliases,
    SmallVec<[(CompactString, u32); 4]>,
    u32,
    u32,
);

#[derive(Default)]
pub(super) struct ElementDirectiveState {
    pub(super) slot_scope: Option<SlotScopeInfo>,
    pub(super) for_scope: Option<ForScopeInfo>,
    pub(super) key_expression: Option<CompactString>,
    pub(super) conditional: Option<(ConditionalKind, Option<CompactString>)>,
}

impl Drawer {
    pub(super) fn enter_element_slot_scope(
        &mut self,
        slot_scope: Option<SlotScopeInfo>,
        is_component: bool,
        tag: &str,
        el: &ElementNode<'_>,
        subtree_end: &mut Option<u32>,
        scope_vars: &mut Vec<CompactString>,
    ) -> usize {
        let Some((slot_name, prop_names, props_pattern, offset)) = slot_scope else {
            return 0;
        };

        let count = prop_names.len();
        if count > 0 || self.options.analyze_template_scopes {
            self.croquis.scopes.enter_v_slot_scope(
                VSlotScopeData {
                    name: slot_name,
                    props_pattern,
                    prop_names: prop_names.iter().cloned().collect(),
                    component: is_component.then(|| CompactString::new(tag)),
                },
                offset,
                *subtree_end.get_or_insert_with(|| element_subtree_end(el)),
            );

            for name in prop_names {
                scope_vars.push(name);
            }
        }

        count
    }

    pub(super) fn enter_element_for_scope(
        &mut self,
        for_scope: Option<ForScopeInfo>,
        key_expression: Option<CompactString>,
        scope_vars: &mut Vec<CompactString>,
    ) -> usize {
        let Some((aliases, alias_offsets, start, end)) = for_scope else {
            return 0;
        };

        let scope_bindings = v_for_scope_bindings(&aliases);
        let count = scope_bindings.len();
        if count == 0 {
            return 0;
        }

        self.croquis.scopes.enter_v_for_scope(
            VForScopeData {
                value_alias: aliases.value_pattern,
                value_bindings: aliases.value_bindings,
                key_alias: aliases.key_alias,
                index_alias: aliases.index_alias,
                source: aliases.source,
                key_expression,
            },
            start,
            end,
        );
        // Entering a v-for scope: O(1) flag read by `is_in_vfor_scope`.
        self.vfor_depth += 1;

        let scope = self.croquis.scopes.current_scope_mut();
        for (name, offset) in &alias_offsets {
            if let Some(binding) = scope.get_binding_mut(name.as_str()) {
                binding.declaration_offset = *offset;
            }
        }

        for var in &scope_bindings {
            scope_vars.push(var.clone());
        }

        count
    }

    pub(super) fn exit_element_for_scope(
        &mut self,
        for_vars_count: usize,
        scope_vars: &mut Vec<CompactString>,
    ) {
        if for_vars_count == 0 {
            return;
        }

        for _ in 0..for_vars_count {
            scope_vars.pop();
        }
        self.croquis.scopes.exit_scope();
        // Pairs with the increment at v-for scope enter above.
        self.vfor_depth -= 1;
    }

    pub(super) fn exit_element_slot_scope(
        &mut self,
        slot_vars_count: usize,
        scope_vars: &mut Vec<CompactString>,
    ) {
        if slot_vars_count == 0 {
            return;
        }

        for _ in 0..slot_vars_count {
            scope_vars.pop();
        }
        self.croquis.scopes.exit_scope();
    }
}

fn _param_names_type_anchor(_: &ParamNames) {}
