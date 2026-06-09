//! Script compilation utilities.
//!
//! Common utilities used across script compilation modules.
//!
//! Note: Some functions in this module are kept for tests but replaced by OXC-based
//! parsing in production. They are marked with `#[allow(dead_code)]`.

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

/// Information about a macro call
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

pub(crate) fn model_modifiers_binding_name(source: &str, call: &MacroCall) -> Option<String> {
    let before_call = source.get(..call.start)?;
    let eq_index = before_call.rfind('=')?;
    let statement = &before_call[..eq_index];
    let statement_start = statement
        .rfind(|c| ['\n', ';'].contains(&c))
        .map_or(0, |index| index + 1);
    let mut lhs = statement[statement_start..].trim();

    for keyword in ["const ", "let ", "var "] {
        if let Some(rest) = lhs.strip_prefix(keyword) {
            lhs = rest.trim_start();
            break;
        }
    }

    if let Some(index) = last_top_level_comma(lhs) {
        lhs = lhs[index + 1..].trim_start();
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
            return Some(&input[start..comma]);
        }
        index += 1;
        start = comma + 1;
    }
    if index == target_index {
        Some(&input[start..])
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

    let name_end = input
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
        .unwrap_or(input.len());
    let name = &input[..name_end];
    if is_valid_identifier(name) {
        Some(name)
    } else {
        None
    }
}

/// Find matching closing parenthesis
#[allow(dead_code)]
pub fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '"';

    for (i, c) in s.char_indices() {
        if in_string {
            if c == string_char && !s[..i].ends_with('\\') {
                in_string = false;
            }
        } else {
            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                }
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
    }

    None
}

/// Find the opening paren after macro name, skipping type args
#[allow(dead_code)]
pub fn find_call_paren(s: &str) -> Option<usize> {
    let mut angle_depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if in_string {
            if c == string_char && (i == 0 || chars[i - 1] != '\\') {
                in_string = false;
            }
        } else {
            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                }
                '<' => angle_depth += 1,
                '>' => {
                    // Check for => arrow function
                    if i > 0 && chars[i - 1] == '=' {
                        continue;
                    }
                    if angle_depth > 0 {
                        angle_depth -= 1;
                    }
                }
                '(' if angle_depth == 0 => return Some(i),
                _ => {}
            }
        }
    }

    None
}

/// Extract type arguments from before a function call
#[allow(dead_code)]
pub fn extract_type_args(before_call: &str) -> Option<String> {
    let trimmed = before_call.trim_end();
    if !trimmed.ends_with('>') {
        return None;
    }

    // Find matching < while handling => (arrow function)
    let chars: Vec<char> = trimmed.chars().collect();
    let mut depth = 0;

    for i in (0..chars.len()).rev() {
        let c = chars[i];
        match c {
            '>' => {
                // Check if this is part of =>
                if i > 0 && chars[i - 1] == '=' {
                    // Skip arrow function =>
                    continue;
                }
                depth += 1;
            }
            '<' => {
                depth -= 1;
                if depth == 0 {
                    return Some(String::from(&trimmed[i + 1..trimmed.len() - 1]));
                }
            }
            _ => {}
        }
    }
    None
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
    use super::{
        extract_type_args, find_matching_paren, get_escaped_prop_name, is_valid_identifier,
    };
    use vize_carton::ToCompactString;

    #[test]
    fn test_find_matching_paren() {
        assert_eq!(find_matching_paren("()"), Some(1));
        assert_eq!(find_matching_paren("(a, b)"), Some(5));
        assert_eq!(find_matching_paren("((nested))"), Some(9));
        assert_eq!(find_matching_paren("(\"string)\")"), Some(10));
    }

    #[test]
    fn test_extract_type_args() {
        assert_eq!(
            extract_type_args("defineProps<{ msg: string }>"),
            Some("{ msg: string }".to_compact_string())
        );
        assert_eq!(extract_type_args("defineProps"), None);
        // Arrow function inside type args
        assert_eq!(
            extract_type_args("defineEmits<(e: 'click') => void>"),
            Some("(e: 'click') => void".to_compact_string())
        );
    }

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
