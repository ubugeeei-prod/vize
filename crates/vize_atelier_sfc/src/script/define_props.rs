//! defineProps macro handling.
//!
//! Handles the `defineProps` Compiler Macro.
//!
//! Note: The regex-based extraction functions are kept for tests but replaced by
//! OXC-based parsing in production.

use super::MacroCall;
#[allow(dead_code)]
use super::utils::{extract_type_args, find_call_paren, find_matching_paren};
use vize_carton::String;

pub use vize_croquis::macros::{DEFINE_PROPS, WITH_DEFAULTS};

/// Extract defineProps call from source
#[allow(dead_code)]
pub fn extract_define_props(content: &str) -> Option<MacroCall> {
    extract_macro_call(content, DEFINE_PROPS)
}

/// Extract withDefaults call from source
#[allow(dead_code)]
pub fn extract_with_defaults(content: &str) -> Option<MacroCall> {
    extract_macro_call(content, WITH_DEFAULTS)
}

/// Extract a macro call from source
#[allow(dead_code)]
fn extract_macro_call(content: &str, macro_name: &str) -> Option<MacroCall> {
    if let Some(start) = content.find(macro_name) {
        let after = &content[start..];
        if let Some(paren_start) = find_call_paren(after)
            && let Some(paren_end) = find_matching_paren(&after[paren_start..])
        {
            let args = String::from(&after[paren_start + 1..paren_start + paren_end]);
            let type_args = extract_type_args(&after[..paren_start]);
            return Some(MacroCall {
                start,
                end: start + paren_start + paren_end + 1,
                args,
                type_args,
                binding_name: None,
            });
        }
    }
    None
}

/// Props runtime type data
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PropTypeData {
    /// Prop key
    pub key: String,
    /// Runtime type strings
    pub type_: Vec<String>,
    /// Whether required
    pub required: bool,
    /// Whether to skip type check
    pub skip_check: bool,
}

#[cfg(test)]
mod tests {
    use super::{extract_define_props, extract_with_defaults};

    #[test]
    fn test_extract_define_props_simple() {
        let content = "const props = defineProps(['msg'])";
        let result = extract_define_props(content);
        assert!(result.is_some());
        let call = result.unwrap();
        assert_eq!(call.args, "['msg']");
    }

    #[test]
    fn test_extract_define_props_typed() {
        let content = "const props = defineProps<{ msg: string }>()";
        let result = extract_define_props(content);
        assert!(result.is_some());
        let call = result.unwrap();
        assert!(call.type_args.is_some());
        assert_eq!(call.type_args.unwrap(), "{ msg: string }");
    }

    #[test]
    fn test_extract_with_defaults() {
        let content = "const props = withDefaults(defineProps<{ msg?: string }>(), { msg: 'hi' })";
        let result = extract_with_defaults(content);
        assert!(result.is_some());
    }
}
