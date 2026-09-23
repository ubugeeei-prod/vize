//! Script compilation utilities.
//!
//! Common utilities used across script compilation modules.

use vize_carton::{String, ToCompactString};
use vize_croquis::macros::runtime_erased_macro_names;

/// Macro definitions found in script setup
#[derive(Debug, Default)]
pub struct ScriptSetupMacros {
    /// defineProps call info
    pub define_props: Option<MacroCall>,
    /// defineEmits call info
    pub define_emits: Option<MacroCall>,
    /// defineExpose call info
    pub define_expose: Option<MacroCall>,
    /// defineOptions call info
    pub define_options: Option<MacroCall>,
    /// defineSlots call info
    pub define_slots: Option<MacroCall>,
    /// defineModel calls
    pub define_models: Vec<MacroCall>,
    /// withDefaults call info
    pub with_defaults: Option<MacroCall>,
    /// Props destructure bindings (Vue 3.3+)
    pub props_destructure: Option<super::PropsDestructuredBindings>,
}

#[derive(Debug, Clone)]
pub struct MacroCall {
    /// Start offset
    pub start: usize,
    /// End offset
    pub end: usize,
    /// Arguments as string
    pub args: String,
    /// Type arguments as string
    pub type_args: Option<String>,
    /// Variable name this macro is assigned to (e.g., "emit" for "const emit = defineEmits(...)")
    pub binding_name: Option<String>,
}

impl MacroCall {
    pub(crate) fn new(
        start: usize,
        end: usize,
        args: String,
        type_args: Option<String>,
        binding_name: Option<String>,
    ) -> Self {
        Self {
            start,
            end,
            args,
            type_args,
            binding_name,
        }
    }
}

pub(crate) fn model_modifiers_binding_name(source: &str, call: &MacroCall) -> Option<String> {
    let before_call = source.get(..call.start)?;
    let (statement, _) = before_call.rsplit_once('=')?;
    let mut lhs = statement
        .rsplit_once(['\n', ';'])
        .map_or(statement, |(_, lhs)| lhs)
        .trim();

    for keyword in ["const ", "let ", "var "] {
        if let Some(rest) = lhs.strip_prefix(keyword) {
            lhs = rest.trim_start();
            break;
        }
    }

    if let Some(index) = last_top_level_comma(lhs) {
        lhs = lhs.get(index + 1..).unwrap_or_default().trim_start();
    }

    let inner = lhs.strip_prefix('[')?.trim_end().strip_suffix(']')?.trim();
    let second = nth_top_level_item(inner, 1)?.trim();
    simple_binding_name(second).map(String::from)
}

fn nth_top_level_item(input: &str, target_index: usize) -> Option<&str> {
    let mut index = 0;
    let mut start = 0;
    for (comma, _) in top_level_commas(input) {
        if index == target_index {
            return input.get(start..comma);
        }
        index += 1;
        start = comma + 1;
    }
    if index == target_index {
        input.get(start..)
    } else {
        None
    }
}

fn last_top_level_comma(input: &str) -> Option<usize> {
    top_level_commas(input).map(|(index, _)| index).last()
}

fn top_level_commas(input: &str) -> impl Iterator<Item = (usize, char)> + '_ {
    let mut depth = 0usize;
    let mut in_string = None;
    let mut escaped = false;

    input.char_indices().filter(move |(_, ch)| {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                return false;
            }
            if *ch == '\\' {
                escaped = true;
                return false;
            }
            if *ch == quote {
                in_string = None;
            }
            return false;
        }

        match *ch {
            '\'' | '"' | '`' => in_string = Some(*ch),
            '[' | '(' | '{' => depth += 1,
            ']' | ')' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return true,
            _ => {}
        }
        false
    })
}

fn simple_binding_name(input: &str) -> Option<&str> {
    let input = input.trim();
    if input.starts_with(['[', '{']) {
        return None;
    }

    let name = input
        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
        .next()
        .unwrap_or_default();
    if is_valid_identifier(name) {
        Some(name)
    } else {
        None
    }
}

/// Check if a line contains a compiler macro call
pub fn is_compiler_macro_line(line: &str) -> bool {
    runtime_erased_macro_names().any(|macro_name| line.contains(macro_name))
}

/// Check if string is valid JS identifier
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }

    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Escape property name for object key
pub fn get_escaped_prop_name(key: &str) -> String {
    if is_valid_identifier(key) {
        key.to_compact_string()
    } else {
        let mut out = String::default();
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:?}", key);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{get_escaped_prop_name, is_valid_identifier};

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("$baz"));
        assert!(!is_valid_identifier("123"));
        assert!(!is_valid_identifier("my-prop"));
    }

    #[test]
    fn test_get_escaped_prop_name() {
        assert_eq!(get_escaped_prop_name("foo"), "foo");
        assert_eq!(get_escaped_prop_name("my-prop"), "\"my-prop\"");
    }
}
