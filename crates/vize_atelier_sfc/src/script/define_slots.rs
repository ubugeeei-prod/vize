//! defineSlots macro handling.
//!
//! Handles the `defineSlots` Compiler Macro.
//!
//! Note: The regex-based extraction functions are kept for tests but replaced by
//! OXC-based parsing in production.

use super::MacroCall;
#[allow(dead_code)]
use super::utils::{extract_type_args, find_call_paren, find_matching_paren};
use vize_carton::String;

pub use vize_croquis::macros::DEFINE_SLOTS;

/// Extract defineSlots call from source
#[allow(dead_code)]
pub fn extract_define_slots(content: &str) -> Option<MacroCall> {
    if let Some(start) = content.find(DEFINE_SLOTS) {
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

#[cfg(test)]
mod tests {
    use super::extract_define_slots;

    #[test]
    fn test_extract_define_slots() {
        let content = "const slots = defineSlots<{ default(): any }>()";
        let result = extract_define_slots(content);
        assert!(result.is_some());
        let call = result.unwrap();
        assert!(call.type_args.is_some());
    }

    #[test]
    fn test_extract_define_slots_empty() {
        let content = "const slots = defineSlots()";
        let result = extract_define_slots(content);
        assert!(result.is_some());
        let call = result.unwrap();
        assert!(call.type_args.is_none());
    }
}
