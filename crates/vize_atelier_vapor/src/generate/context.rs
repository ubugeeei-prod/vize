//! Code generation context that tracks state during Vapor code emission.

use super::expression;
use vize_atelier_core::options::BindingMetadata;
use vize_carton::{camelize, capitalize, cstr, FxHashMap, FxHashSet, String, ToCompactString};
use vize_croquis::builtins::is_global_allowed;

/// For-loop scope entry
#[derive(Debug, Clone)]
pub(crate) struct ForScope {
    /// Value alias (e.g., "item") -> "_for_item{depth}"
    pub(crate) value_alias: Option<String>,
    /// Key alias (e.g., "index" or "key") -> "_for_key{depth}"
    pub(crate) key_alias: Option<String>,
    /// Index alias -> unused in current vapor
    #[allow(dead_code)]
    pub(crate) index_alias: Option<String>,
    /// Depth of for nesting (0-based)
    pub(crate) depth: usize,
}

/// Slot scope entry for scoped slots
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SlotScope {
    /// Destructured variable names (e.g., ["item", "index"] from "{ item, index }")
    pub(crate) names: std::vec::Vec<String>,
    /// Slot props variable (e.g., "_slotProps0")
    pub(crate) slot_props_var: String,
}

/// Generate context
pub(crate) struct GenerateContext<'a> {
    pub(crate) code: String,
    indent_level: u32,
    #[allow(dead_code)]
    pub(crate) element_template_map: &'a FxHashMap<usize, usize>,
    temp_count: usize,
    /// Used helpers for import generation
    pub(crate) used_helpers: FxHashSet<&'static str>,
    /// Events that need delegation (event names)
    pub(crate) delegate_events: FxHashSet<String>,
    /// Text node references (element_id -> text_node_var)
    pub(crate) text_nodes: FxHashMap<usize, String>,
    /// Whether currently inside a non-root block (v-if, v-for)
    pub(crate) is_fragment: bool,
    /// For-loop scope stack
    pub(crate) for_scopes: std::vec::Vec<ForScope>,
    /// Slot scope stack for scoped slots
    #[allow(dead_code)]
    pub(crate) slot_scopes: std::vec::Vec<SlotScope>,
    /// Counter for slot scope variable names
    #[allow(dead_code)]
    pub(crate) slot_scope_count: usize,
    /// Components that have already been resolved (to avoid duplicate resolveComponent calls)
    pub(crate) resolved_components: FxHashSet<String>,
    /// Component resolutions created inside callback scopes and removed on exit.
    resolved_component_scopes: std::vec::Vec<std::vec::Vec<String>>,
    /// Element IDs that are standalone text nodes (no _txt needed)
    pub(crate) standalone_text_elements: &'a FxHashSet<usize>,
    /// Binding metadata from script setup imports.
    pub(crate) binding_metadata: Option<&'a BindingMetadata>,
}

impl<'a> GenerateContext<'a> {
    pub(crate) fn new(
        element_template_map: &'a FxHashMap<usize, usize>,
        standalone_text_elements: &'a FxHashSet<usize>,
        binding_metadata: Option<&'a BindingMetadata>,
    ) -> Self {
        Self {
            code: String::with_capacity(4096),
            indent_level: 0,
            element_template_map,
            temp_count: 0,
            used_helpers: FxHashSet::default(),
            delegate_events: FxHashSet::default(),
            text_nodes: FxHashMap::default(),
            is_fragment: false,
            for_scopes: std::vec::Vec::new(),
            slot_scopes: std::vec::Vec::new(),
            slot_scope_count: 0,
            resolved_components: FxHashSet::default(),
            resolved_component_scopes: std::vec::Vec::new(),
            standalone_text_elements,
            binding_metadata,
        }
    }

    /// Resolve an expression, replacing for-loop aliases with _for_item/key references
    pub(crate) fn resolve_expression(&self, expr: &str) -> String {
        expression::resolve_expression(self, expr)
    }

    /// Resolve complex expressions (object/array literals) by prefixing identifiers inside
    pub(super) fn resolve_complex_expression_fallback(&self, expr: &str) -> String {
        let mut result = String::default();
        let mut chars = expr.chars().peekable();
        let mut in_string = false;
        let mut string_char = ' ';
        // Track whether we're in key position (after { or ,) vs value position (after :)
        let mut in_object = false;
        let mut is_key_position = false;

        while let Some(&ch) = chars.peek() {
            if in_string {
                result.push(ch);
                chars.next();
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = ch;
                    result.push(ch);
                    chars.next();
                }
                '{' => {
                    in_object = true;
                    is_key_position = true;
                    result.push(ch);
                    chars.next();
                }
                '}' => {
                    in_object = false;
                    result.push(ch);
                    chars.next();
                }
                ':' => {
                    is_key_position = false;
                    result.push(ch);
                    chars.next();
                }
                ',' => {
                    if in_object {
                        is_key_position = true;
                    }
                    result.push(ch);
                    chars.next();
                }
                '[' => {
                    // Computed property key: [expr] - contents should be prefixed
                    if in_object && is_key_position {
                        // Save state, temporarily treat as value position
                        is_key_position = false;
                    }
                    result.push(ch);
                    chars.next();
                }
                ']' => {
                    // After computed key, we're back to key position until ':'
                    if in_object {
                        is_key_position = true;
                    }
                    result.push(ch);
                    chars.next();
                }
                ' ' | '\n' | '\t' => {
                    result.push(ch);
                    chars.next();
                }
                _ => {
                    // Collect identifier/value
                    let mut ident = String::default();
                    while let Some(&c) = chars.peek() {
                        if c == ','
                            || c == '}'
                            || c == ']'
                            || c == ':'
                            || c == ' '
                            || c == '\n'
                            || c == '\t'
                        {
                            break;
                        }
                        ident.push(c);
                        chars.next();
                    }
                    let trimmed_ident = ident.trim();
                    if trimmed_ident.is_empty()
                        || trimmed_ident == "true"
                        || trimmed_ident == "false"
                        || trimmed_ident == "null"
                        || trimmed_ident == "undefined"
                        || trimmed_ident.parse::<f64>().is_ok()
                        || (in_object && is_key_position)
                    {
                        // Don't prefix: literals, empty, or object keys
                        result.push_str(&ident);
                    } else {
                        result.push_str(&self.resolve_expression(trimmed_ident));
                    }
                }
            }
        }
        result
    }

    pub(super) fn resolve_scope_binding(&self, name: &str) -> Option<String> {
        for scope in self.for_scopes.iter().rev() {
            if let Some(ref value_alias) = scope.value_alias {
                let for_var = cstr!("_for_item{}", scope.depth);

                if value_alias.starts_with('{') || value_alias.starts_with('(') {
                    let names = parse_destructure_names(value_alias.as_str());
                    for binding_name in &names {
                        if name == *binding_name {
                            return Some(cstr!("{}.value.{}", for_var, binding_name));
                        }
                    }
                } else if name == value_alias.as_str() {
                    return Some(cstr!("{}.value", for_var));
                }
            }

            if let Some(ref key_alias) = scope.key_alias {
                if name == key_alias.as_str() {
                    return Some(cstr!("_for_key{}.value", scope.depth));
                }
            }
        }

        for scope in self.slot_scopes.iter().rev() {
            for slot_name in &scope.names {
                if name == slot_name.as_str() {
                    return Some(cstr!("{}.{}", scope.slot_props_var, slot_name));
                }
            }
        }

        None
    }

    pub(super) fn resolve_simple_reference(&self, expr: &str) -> String {
        if let Some((head, tail)) = expr.split_once('.') {
            if let Some(replacement) = self.resolve_scope_binding(head) {
                let mut resolved = replacement;
                resolved.push('.');
                resolved.push_str(tail);
                return resolved;
            }

            if is_global_allowed(head)
                || matches!(head, "_ctx" | "$props" | "$slots" | "$attrs" | "$emit")
            {
                return expr.to_compact_string();
            }

            let mut resolved = String::with_capacity(expr.len() + 5);
            resolved.push_str("_ctx.");
            resolved.push_str(expr);
            return resolved;
        }

        if let Some(replacement) = self.resolve_scope_binding(expr) {
            return replacement;
        }

        if is_global_allowed(expr)
            || matches!(expr, "_ctx" | "$props" | "$slots" | "$attrs" | "$emit")
        {
            return expr.to_compact_string();
        }

        cstr!("_ctx.{}", expr)
    }

    pub(crate) fn add_delegate_event(&mut self, event_name: &str) {
        self.delegate_events.insert(event_name.to_compact_string());
    }

    pub(crate) fn next_text_node(&mut self, element_id: usize) -> String {
        // Use element ID for text node variable name (x2 matches n2)
        let mut var_name = String::with_capacity(8);
        var_name.push('x');
        var_name.push_str(&element_id.to_compact_string());
        self.text_nodes.insert(element_id, var_name.clone());
        var_name
    }

    pub(crate) fn use_helper(&mut self, name: &'static str) {
        self.used_helpers.insert(name);
    }

    pub(crate) fn push_component_scope(&mut self) {
        self.resolved_component_scopes.push(std::vec::Vec::new());
    }

    pub(crate) fn pop_component_scope(&mut self) {
        let Some(added_components) = self.resolved_component_scopes.pop() else {
            return;
        };

        for component in added_components {
            self.resolved_components.remove(&component);
        }
    }

    pub(crate) fn is_component_resolved(&self, component: &str) -> bool {
        self.resolved_components.contains(component)
    }

    pub(crate) fn resolve_component_binding_expr(&self, component: &str) -> Option<String> {
        let bindings = self.binding_metadata?;

        let resolve_base = |name: &str| {
            if bindings.bindings.contains_key(name) {
                return Some(name.to_compact_string());
            }

            let camel = camelize(name);
            if bindings.bindings.contains_key(camel.as_str()) {
                return Some(camel);
            }

            let pascal = capitalize(&camel);
            if bindings.bindings.contains_key(pascal.as_str()) {
                return Some(pascal);
            }

            None
        };

        if let Some((base, suffix)) = component.split_once('.') {
            let resolved_base = resolve_base(base)?;
            return Some(cstr!("_ctx.{}.{}", resolved_base, suffix));
        }

        resolve_base(component).map(|binding| cstr!("_ctx.{}", binding))
    }

    pub(crate) fn mark_component_resolved(&mut self, component: &str) {
        let component = component.to_compact_string();
        if self.resolved_components.insert(component.clone()) {
            if let Some(scope) = self.resolved_component_scopes.last_mut() {
                scope.push(component);
            }
        }
    }

    pub(crate) fn push(&mut self, s: &str) {
        self.code.push_str(s);
    }

    pub(crate) fn push_line(&mut self, s: &str) {
        self.push_indent();
        self.code.push_str(s);
        self.code.push('\n');
    }

    pub(crate) fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.code.push_str("  ");
        }
    }

    pub(crate) fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub(crate) fn deindent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Push string to buffer (alias for `push`, compatible with `appends!`/`append!` macros)
    #[allow(dead_code)]
    pub(crate) fn push_str(&mut self, s: &str) {
        self.code.push_str(s);
    }

    /// Push formatted line (format_args! + newline with indentation)
    pub(crate) fn push_line_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        self.push_indent();
        use std::fmt::Write as _;
        self.write_fmt(args).unwrap();
        self.code.push('\n');
    }

    /// Push a slot scope for scoped slots. Returns the slot props variable name.
    #[allow(dead_code)]
    pub(crate) fn push_slot_scope(&mut self, destructure_pattern: &str) -> String {
        let slot_props_var = cstr!("_slotProps{}", self.slot_scope_count);
        self.slot_scope_count += 1;

        let names = parse_destructure_names(destructure_pattern)
            .into_iter()
            .map(|s| s.to_compact_string())
            .collect();

        self.slot_scopes.push(SlotScope {
            names,
            slot_props_var: slot_props_var.clone(),
        });
        slot_props_var
    }

    /// Pop the current slot scope
    #[allow(dead_code)]
    pub(crate) fn pop_slot_scope(&mut self) {
        self.slot_scopes.pop();
    }

    pub(crate) fn next_temp(&mut self) -> String {
        let name = cstr!("_t{}", self.temp_count);
        self.temp_count += 1;
        name
    }
}

/// Parse destructured variable names from patterns like "{ id, name }" or "{ a: b }"
fn parse_destructure_names(pattern: &str) -> std::vec::Vec<&str> {
    let inner = pattern
        .trim_start_matches(['{', '(', ' '])
        .trim_end_matches(['}', ')', ' ']);
    inner
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            // Handle "a: b" -> use "b" (the bound name)
            if let Some(pos) = part.find(':') {
                Some(part[pos + 1..].trim())
            } else if part.is_empty() {
                None
            } else {
                Some(part)
            }
        })
        .collect()
}

impl std::fmt::Write for GenerateContext<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.code.push_str(s);
        Ok(())
    }
}
