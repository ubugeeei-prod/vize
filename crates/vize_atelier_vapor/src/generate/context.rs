//! Code generation context that tracks state during Vapor code emission.

use vize_carton::{cstr, FxHashMap, FxHashSet, String, ToCompactString};

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
    /// Element IDs that are standalone text nodes (no _txt needed)
    pub(crate) standalone_text_elements: &'a FxHashSet<usize>,
}

impl<'a> GenerateContext<'a> {
    pub(crate) fn new(
        element_template_map: &'a FxHashMap<usize, usize>,
        standalone_text_elements: &'a FxHashSet<usize>,
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
            standalone_text_elements,
        }
    }

    /// Resolve an expression, replacing for-loop aliases with _for_item/key references
    pub(crate) fn resolve_expression(&self, expr: &str) -> String {
        let trimmed = expr.trim();

        // Don't prefix numeric literals
        if trimmed.parse::<f64>().is_ok() {
            return trimmed.to_compact_string();
        }

        // Don't prefix string literals
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            return trimmed.to_compact_string();
        }

        // Handle negation
        if let Some(rest) = trimmed.strip_prefix('!') {
            let inner = self.resolve_expression(rest.trim());
            return cstr!("!{}", inner);
        }

        // Don't prefix object/array literals - prefix inner expressions instead
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return self.resolve_complex_expression(trimmed);
        }

        // Check for-loop scopes (innermost first)
        for scope in self.for_scopes.iter().rev() {
            if let Some(ref value_alias) = scope.value_alias {
                let for_var = cstr!("_for_item{}", scope.depth);

                // Check if this is a destructured alias like "{ id, name }"
                if value_alias.starts_with('{') || value_alias.starts_with('(') {
                    // Parse destructured names
                    let names = parse_destructure_names(value_alias.as_str());
                    for name in &names {
                        if trimmed == *name {
                            return cstr!("{}.value.{}", for_var, name);
                        }
                        let member_prefix = [*name, "."].concat();
                        if trimmed.starts_with(&member_prefix) {
                            return cstr!("{}.value.{}", for_var, trimmed);
                        }
                    }
                } else {
                    // Simple alias
                    if trimmed == value_alias.as_str() {
                        return cstr!("{}.value", for_var);
                    }
                    let prefix = [value_alias.as_str(), "."].concat();
                    if trimmed.starts_with(&prefix) {
                        return cstr!("{}.value.{}", for_var, &trimmed[prefix.len()..]);
                    }
                }
            }
            if let Some(ref key_alias) = scope.key_alias {
                let for_key_var = cstr!("_for_key{}", scope.depth);
                if trimmed == key_alias.as_str() {
                    return cstr!("{}.value", for_key_var);
                }
                let prefix = [key_alias.as_str(), "."].concat();
                if trimmed.starts_with(&prefix) {
                    return cstr!("{}.value.{}", for_key_var, &trimmed[prefix.len()..]);
                }
            }
        }
        // Check slot scopes (innermost first)
        for scope in self.slot_scopes.iter().rev() {
            for name in &scope.names {
                if trimmed == name.as_str() {
                    return cstr!("{}.{}", scope.slot_props_var, name);
                }
                let member_prefix = [name.as_str(), "."].concat();
                if trimmed.starts_with(&member_prefix) {
                    return cstr!("{}.{}", scope.slot_props_var, trimmed);
                }
            }
        }

        // Not a for-loop or slot variable, use _ctx prefix
        cstr!("_ctx.{}", trimmed)
    }

    /// Resolve complex expressions (object/array literals) by prefixing identifiers inside
    fn resolve_complex_expression(&self, expr: &str) -> String {
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
