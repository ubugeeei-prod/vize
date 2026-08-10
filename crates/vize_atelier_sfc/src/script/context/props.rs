//! Props extraction from defineProps macro calls.
//!
//! Handles extracting prop names and types from both runtime
//! and type-based defineProps declarations.

use vize_carton::{CompactString, FxHashSet, String, ToCompactString};

use crate::script::resolve_type_to_object_body;
use crate::types::BindingType;
use vize_croquis::macros::PropDefinition;

use super::super::MacroCall;
use super::ScriptCompileContext;

/// Check if a string is a valid JavaScript identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn normalize_type_prop_name(name_part: &str) -> &str {
    let mut name = name_part.trim();

    while let Some(rest) = name.strip_prefix("readonly") {
        if !rest.chars().next().is_some_and(|c| c.is_ascii_whitespace()) {
            break;
        }
        name = rest.trim_start();
    }

    name.trim_end_matches('?').trim()
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

        for_each_top_level_type_member(&resolved_content, |segment| {
            self.extract_single_prop_from_type(segment);
        });
    }

    /// Resolve type-based defineProps arguments to top-level prop names.
    ///
    /// This follows the same type-reference expansion and member splitting used
    /// by `extract_props_from_type_args`, but returns a set for callers that need
    /// to distinguish top-level props from nested object/union members.
    pub fn resolve_type_prop_names(&self, type_args: &str) -> FxHashSet<CompactString> {
        self.resolve_type_props(type_args)
            .into_iter()
            .map(|prop| prop.name)
            .collect()
    }

    /// Resolve type-based defineProps arguments to completion-ready metadata.
    ///
    /// Unlike Croquis' local type resolver, this sees normal-script and
    /// externally imported declarations collected by the compile context.
    pub(crate) fn resolve_type_props(&self, type_args: &str) -> Vec<PropDefinition> {
        let content = type_args.trim();
        let Some(resolved_content) =
            resolve_type_to_object_body(content, &self.interfaces, &self.type_aliases)
        else {
            return Vec::new();
        };

        let mut props = Vec::new();
        let mut names = FxHashSet::default();
        for_each_top_level_type_member(&resolved_content, |segment| {
            if let Some(prop) = type_prop_definition(segment)
                && names.insert(prop.name.clone())
            {
                props.push(prop);
            }
        });
        props
    }

    /// Extract a single prop name from a type definition segment
    fn extract_single_prop_from_type(&mut self, segment: &str) {
        if let Some(name) = type_prop_name(segment) {
            self.bindings.bindings.insert(name, BindingType::Props);
        }
    }
}

fn for_each_top_level_type_member(content: &str, mut visit: impl FnMut(&str)) {
    let mut depth = 0;
    let mut current = String::default();
    let mut prev = '\0';

    for c in content.chars() {
        match c {
            '{' | '<' | '(' | '[' => {
                depth += 1;
                current.push(c);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                current.push(c);
            }
            '>' if prev != '=' => {
                depth -= 1;
                current.push(c);
            }
            '\n' if depth == 0 && type_annotation_starts_on_next_line(&current) => {
                current.push(c);
            }
            ',' | ';' | '\n' if depth == 0 => {
                visit(&current);
                current.clear();
            }
            _ => current.push(c),
        }
        prev = c;
    }
    visit(&current);
}

fn type_annotation_starts_on_next_line(member: &str) -> bool {
    member
        .split_once(':')
        .is_some_and(|(_, prop_type)| prop_type.trim().is_empty())
}

fn type_prop_name(segment: &str) -> Option<CompactString> {
    type_prop_definition(segment).map(|prop| prop.name)
}

fn type_prop_definition(segment: &str) -> Option<PropDefinition> {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return None;
    }

    let colon_pos = trimmed.find(':')?;
    let name_part = &trimmed[..colon_pos];
    let raw_name = normalize_type_prop_name(name_part);
    let optional = name_part.trim().trim_end().ends_with('?');
    let name = unquote_prop_name(raw_name)?;
    let prop_type = trimmed[colon_pos + 1..].trim();
    if prop_type.is_empty() {
        return None;
    }

    Some(PropDefinition {
        name: name.to_compact_string(),
        prop_type: Some(prop_type.to_compact_string()),
        required: !optional,
        default_value: None,
    })
}

fn unquote_prop_name(name: &str) -> Option<&str> {
    if name.len() >= 2 {
        let first = name.as_bytes()[0];
        let last = name.as_bytes()[name.len() - 1];
        if matches!((first, last), (b'\'', b'\'') | (b'"', b'"')) {
            let unquoted = &name[1..name.len() - 1];
            return (!unquoted.is_empty()).then_some(unquoted);
        }
    }

    (!name.is_empty() && is_valid_identifier(name)).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::ScriptCompileContext;

    #[test]
    fn resolve_type_prop_names_keeps_union_members_nested() {
        let mut ctx = ScriptCompileContext::new(
            r#"
interface Props {
  isOpened: boolean
  interaction?:
    | { text: string; to: string; event?: never }
    | { text: string; event: () => void; to?: never }
}

const props = defineProps<Props>()
"#,
        );
        ctx.analyze();

        let names = ctx.resolve_type_prop_names("Props");

        assert!(names.contains("isOpened"));
        assert!(names.contains("interaction"));
        assert!(!names.contains("text"));
        assert!(!names.contains("to"));
        assert!(!names.contains("event"));
    }

    #[test]
    fn resolve_type_props_preserves_types_requiredness_and_quoted_names() {
        let mut ctx = ScriptCompileContext::new(
            r#"
interface BaseProps {
  readonly enabled?: boolean
  'aria-label': string
}
interface Props extends BaseProps {
  mode: 'single' | 'multiple'
}
const props = defineProps<Props>()
"#,
        );
        ctx.analyze();

        let props = ctx.resolve_type_props("Props");
        let find = |name: &str| {
            props
                .iter()
                .find(|prop| prop.name == name)
                .unwrap_or_else(|| panic!("missing {name:?} in {props:#?}"))
        };

        assert_eq!(find("enabled").prop_type.as_deref(), Some("boolean"));
        assert!(!find("enabled").required);
        assert_eq!(find("aria-label").prop_type.as_deref(), Some("string"));
        assert!(find("aria-label").required);
        assert_eq!(
            find("mode").prop_type.as_deref(),
            Some("'single' | 'multiple'")
        );
        assert!(find("mode").required);
    }
}
