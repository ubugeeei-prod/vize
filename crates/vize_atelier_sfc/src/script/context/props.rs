//! Props extraction from defineProps macro calls.
//!
//! Handles extracting prop names and types from both runtime
//! and type-based defineProps declarations.

use vize_carton::ToCompactString;

use crate::script::resolve_type_to_object_body;
use crate::types::BindingType;

use super::super::MacroCall;
use super::ScriptCompileContext;

/// Check if a string is a valid JavaScript identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

impl ScriptCompileContext {
    /// Extract prop names from defineProps/withDefaults and add to bindings
    pub(super) fn extract_props_bindings(&mut self, call: &MacroCall) {
        // Handle type-based defineProps: defineProps<{ msg: string }>()
        if let Some(ref type_args) = call.type_args {
            self.extract_props_from_type_args(type_args);
            return;
        }

        // Parse args to extract prop names
        // Handle array syntax: ['msg', 'count']
        // Handle object syntax: { msg: String, count: Number }
        let args = call.args.trim();

        if args.starts_with('[') && args.ends_with(']') {
            // Array syntax
            let inner = &args[1..args.len() - 1];
            for part in inner.split(',') {
                let part = part.trim();
                // Extract string literal
                if (part.starts_with('\'') && part.ends_with('\''))
                    || (part.starts_with('"') && part.ends_with('"'))
                {
                    let name = &part[1..part.len() - 1];
                    self.bindings
                        .bindings
                        .insert(name.to_compact_string(), BindingType::Props);
                }
            }
        } else if args.starts_with('{') && args.ends_with('}') {
            // Object syntax - extract keys
            let inner = &args[1..args.len() - 1];
            for part in inner.split(',') {
                let part = part.trim();
                // Find key before : or whitespace
                if let Some(colon_pos) = part.find(':') {
                    let key = part[..colon_pos].trim();
                    if !key.is_empty() && is_valid_identifier(key) {
                        self.bindings
                            .bindings
                            .insert(key.to_compact_string(), BindingType::Props);
                    }
                } else if is_valid_identifier(part) {
                    // Shorthand property
                    self.bindings
                        .bindings
                        .insert(part.to_compact_string(), BindingType::Props);
                }
            }
        }
    }

    /// Extract prop names from TypeScript type arguments
    fn extract_props_from_type_args(&mut self, type_args: &str) {
        let content = type_args.trim();

        let Some(resolved_content) =
            resolve_type_to_object_body(content, &self.interfaces, &self.type_aliases)
        else {
            return;
        };

        // Split by commas/semicolons/newlines (but not inside nested braces)
        let mut depth = 0;
        let mut current = vize_carton::String::default();

        for c in resolved_content.chars() {
            match c {
                '{' | '<' | '(' | '[' => {
                    depth += 1;
                    current.push(c);
                }
                '}' | '>' | ')' | ']' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' | ';' | '\n' if depth == 0 => {
                    self.extract_single_prop_from_type(&current);
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        self.extract_single_prop_from_type(&current);
    }

    /// Extract a single prop name from a type definition segment
    fn extract_single_prop_from_type(&mut self, segment: &str) {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            return;
        }

        // Parse "name?: Type" or "name: Type"
        if let Some(colon_pos) = trimmed.find(':') {
            let name_part = &trimmed[..colon_pos];
            let name = name_part.trim().trim_end_matches('?').trim();

            if !name.is_empty() && is_valid_identifier(name) {
                self.bindings
                    .bindings
                    .insert(name.to_compact_string(), BindingType::Props);
            }
        }
    }
}
