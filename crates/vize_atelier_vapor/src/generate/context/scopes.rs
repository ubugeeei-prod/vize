//! For-loop and slot scope entries the Vapor generate context stacks.

use vize_atelier_core::options::BindingType;
use vize_carton::{String, ToCompactString, cstr};

use super::GenerateContext;
use crate::generate::destructure::{
    DestructureBinding, needs_parsed_reads, parse_destructure_bindings, resolve_props_binding,
};

impl GenerateContext<'_> {
    /// Resolve a name through active loop/slot scopes and prop metadata.
    pub(in crate::generate) fn resolve_scope_binding_inner(&self, name: &str) -> Option<String> {
        // Innermost scope first. Loop and slot scopes interleave, so a slot
        // parameter declared inside a loop shadows the loop's alias.
        let (mut loops, mut slots) = (self.for_scopes.len(), self.slot_scopes.len());
        while loops > 0 || slots > 0 {
            if let Some(scope) = (slots.checked_sub(1))
                .and_then(|slot| self.slot_scopes.get(slot))
                .filter(|scope| scope.for_depth >= loops)
            {
                slots -= 1;
                if scope
                    .names
                    .iter()
                    .any(|slot_name| name == slot_name.as_str())
                {
                    // Default-bearing reads are recorded after the plain key
                    // paths, so the last override for a name wins.
                    if let Some(binding) = scope
                        .read_overrides
                        .iter()
                        .rev()
                        .find(|binding| name == binding.local.as_str())
                    {
                        return Some(binding.path.clone());
                    }
                    return Some(if scope.whole {
                        scope.slot_props_var.clone()
                    } else {
                        cstr!("{}.{}", scope.slot_props_var, name)
                    });
                }
                continue;
            }
            // Every slot scope sits inside `loops >= 0` loop scopes, so a
            // remaining slot scope is taken above and `loops > 0` here.
            loops -= 1;
            let Some(scope) = self.for_scopes.get(loops) else {
                continue;
            };
            if let Some(ref value_alias) = scope.value_alias {
                let for_var = cstr!("_for_item{}", scope.depth);

                if let Some(binding) = scope
                    .read_overrides
                    .iter()
                    .rev()
                    .find(|binding| name == binding.local.as_str())
                {
                    return Some(binding.path.clone());
                }
                if value_alias.starts_with(['{', '[', '(']) {
                    for binding in parse_destructure_bindings(value_alias.as_str()) {
                        if name == binding.local.as_str() {
                            return Some(cstr!("{}.value{}", for_var, binding.path));
                        }
                    }
                } else if name == value_alias.as_str() {
                    return Some(cstr!("{}.value", for_var));
                }
            }

            if let Some(ref key_alias) = scope.key_alias
                && name == key_alias.as_str()
            {
                return Some(cstr!("_for_key{}.value", scope.depth));
            }

            if let Some(ref index_alias) = scope.index_alias
                && name == index_alias.as_str()
            {
                return Some(cstr!("_for_index{}.value", scope.depth));
            }
        }

        // Literal constants, including static enums, are hoisted into the
        // module scope. They do not belong to the component proxy.
        if self
            .binding_metadata
            .and_then(|bindings| bindings.bindings.get(name))
            == Some(&BindingType::LiteralConst)
        {
            return Some(name.to_compact_string());
        }
        resolve_props_binding(self.binding_metadata, name)
    }

    /// Free names need no scoped resolution when no binding context is active.
    #[inline]
    pub(in crate::generate) fn resolve_scope_binding(&self, name: &str) -> Option<String> {
        if self.binding_metadata.is_none()
            && self.for_scopes.is_empty()
            && self.slot_scopes.is_empty()
        {
            return None;
        }
        self.resolve_scope_binding_inner(name)
    }

    /// Push a slot scope for scoped slots. Returns the slot props variable name.
    pub(crate) fn push_slot_scope(&mut self, destructure_pattern: &str) -> String {
        // A plain identifier names the whole props object, as upstream emits it:
        // `v-slot="p"` becomes `(p) => ... p.x`, not a read through `_slotProps`.
        let pattern = destructure_pattern.trim();
        if pattern.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
            && pattern
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        {
            self.slot_scopes.push(SlotScope {
                names: std::vec![String::from(pattern)],
                slot_props_var: String::from(pattern),
                for_depth: self.for_scopes.len(),
                whole: true,
                read_overrides: std::vec::Vec::new(),
            });
            return String::from(pattern);
        }
        let slot_props_var = cstr!("_slotProps{}", self.slot_scope_count);
        self.slot_scope_count += 1;
        let bindings = parse_destructure_bindings(destructure_pattern);
        let names = bindings
            .iter()
            .map(|binding| binding.local.clone())
            .collect();
        // A renamed or nested name reads through the key it was destructured
        // from: `{ props: trigger }` reads `.props`, `{ item: { label } }`
        // reads `.item.label`.
        let read_overrides = bindings
            .into_iter()
            .filter(|binding| binding.path != cstr!(".{}", binding.local))
            .map(|binding| DestructureBinding {
                path: cstr!("{slot_props_var}{}", binding.path),
                local: binding.local,
            })
            .collect();
        self.slot_scopes.push(SlotScope {
            names,
            slot_props_var: slot_props_var.clone(),
            for_depth: self.for_scopes.len(),
            whole: false,
            read_overrides,
        });
        if needs_parsed_reads(destructure_pattern) {
            self.resolve_slot_defaults(destructure_pattern, &slot_props_var);
        }
        slot_props_var
    }

    /// Pop the current slot scope
    pub(crate) fn pop_slot_scope(&mut self) {
        self.slot_scopes.pop();
    }

    /// Push a loop scope. A destructured value the textual scanner cannot
    /// follow gets its parsed reads, rooted at the loop item.
    pub(crate) fn push_for_scope(
        &mut self,
        value_alias: Option<&str>,
        key_alias: Option<&str>,
        index_alias: Option<&str>,
    ) {
        let depth = self.for_scopes.len();
        self.for_scopes.push(ForScope {
            value_alias: value_alias.map(String::new),
            key_alias: key_alias.map(String::new),
            index_alias: index_alias.map(String::new),
            depth,
            read_overrides: std::vec::Vec::new(),
        });
        if let Some(pattern) = value_alias
            && pattern.trim_start().starts_with(['{', '['])
            && needs_parsed_reads(pattern)
        {
            self.resolve_for_value_reads(pattern, &cstr!("_for_item{depth}.value"));
        }
    }
}

/// For-loop scope entry
#[derive(Debug, Clone)]
pub(crate) struct ForScope {
    /// Value alias (e.g., "item") -> "_for_item{depth}"
    pub(crate) value_alias: Option<String>,
    /// Key alias (e.g., "index" or "key") -> "_for_key{depth}"
    pub(crate) key_alias: Option<String>,
    /// Index alias -> "_for_index{depth}"
    pub(crate) index_alias: Option<String>,
    /// Depth of for nesting (0-based)
    pub(crate) depth: usize,
    /// Parsed reads for value patterns with defaults, computed keys or rest
    /// elements; the last one recorded for a name wins.
    pub(in crate::generate) read_overrides: std::vec::Vec<DestructureBinding>,
}

/// Slot scope entry for scoped slots
#[derive(Debug, Clone)]
pub(crate) struct SlotScope {
    /// Destructured variable names (e.g., ["item", "index"] from "{ item, index }")
    pub(crate) names: std::vec::Vec<String>,
    /// Slot props variable (e.g., "_slotProps0")
    pub(crate) slot_props_var: String,
    /// Loop scopes already active when this slot scope opened: it is nested
    /// inside exactly those, so its names shadow theirs and no later ones.
    pub(crate) for_depth: usize,
    /// The pattern is a plain identifier naming the whole props object.
    pub(crate) whole: bool,
    /// Authored default-bearing patterns retain their accessor and lazy default.
    pub(super) read_overrides: std::vec::Vec<DestructureBinding>,
}
