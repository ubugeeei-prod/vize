//! defineExpose macro handling.
//!
//! Handles the `defineExpose` Compiler Macro.
//!
//! Note: The regex-based extraction functions are kept for tests but replaced by
//! OXC-based parsing in production.

#[allow(dead_code)]
use super::utils::{extract_type_args, find_call_paren, find_matching_paren};
use super::MacroCall;
use vize_carton::String;

pub use vize_croquis::macros::DEFINE_EXPOSE;

/// Extract defineExpose call from source
#[allow(dead_code)]
pub fn extract_define_expose(content: &str) -> Option<MacroCall> {
    if let Some(start) = content.find(DEFINE_EXPOSE) {
        let after = &content[start..];
        if let Some(paren_start) = find_call_paren(after) {
            if let Some(paren_end) = find_matching_paren(&after[paren_start..]) {
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
    }
    None
}

#[cfg(test)]
mod tests {
    use super::extract_define_expose;

    #[test]
    fn test_extract_define_expose() {
        let content = "defineExpose({ foo, bar })";
        let result = extract_define_expose(content);
        assert!(result.is_some());
        let call = result.unwrap();
        assert_eq!(call.args, "{ foo, bar }");
    }
}
