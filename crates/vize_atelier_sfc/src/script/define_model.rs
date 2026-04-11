//! defineModel macro handling.
//!
//! Handles the `defineModel` Compiler Macro.
//!
//! Note: The regex-based extraction functions are kept for tests but replaced by
//! OXC-based parsing in production.

#[allow(dead_code)]
use super::utils::{extract_type_args, find_call_paren, find_matching_paren};
use super::MacroCall;
use vize_carton::String;

pub const DEFINE_MODEL: &str = "defineModel";

/// Extract all defineModel calls from source (can appear multiple times)
#[allow(dead_code)]
pub fn extract_define_model(content: &str) -> Vec<MacroCall> {
    let mut calls = Vec::new();
    let mut search_from = 0;

    while let Some(pos) = content[search_from..].find(DEFINE_MODEL) {
        let start = search_from + pos;
        let after = &content[start..];

        if let Some(paren_start) = find_call_paren(after) {
            if let Some(paren_end) = find_matching_paren(&after[paren_start..]) {
                let args = String::from(&after[paren_start + 1..paren_start + paren_end]);
                let type_args = extract_type_args(&after[..paren_start]);
                calls.push(MacroCall {
                    start,
                    end: start + paren_start + paren_end + 1,
                    args,
                    type_args,
                    binding_name: None,
                });
                search_from = start + paren_start + paren_end + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    calls
}

/// Model declaration info
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModelDecl {
    /// Model name
    pub name: String,
    /// Type arguments
    pub type_args: Option<String>,
    /// Options
    pub options: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::extract_define_model;

    #[test]
    fn test_extract_define_model_single() {
        let content = "const modelValue = defineModel<string>()";
        let result = extract_define_model(content);
        assert_eq!(result.len(), 1);
        assert!(result[0].type_args.is_some());
    }

    #[test]
    fn test_extract_define_model_multiple() {
        let content = r#"
const firstName = defineModel('firstName')
const lastName = defineModel('lastName')
"#;
        let result = extract_define_model(content);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_extract_define_model_with_options() {
        let content = "const count = defineModel('count', { type: Number, default: 0 })";
        let result = extract_define_model(content);
        assert_eq!(result.len(), 1);
        insta::assert_debug_snapshot!(result);
    }
}
