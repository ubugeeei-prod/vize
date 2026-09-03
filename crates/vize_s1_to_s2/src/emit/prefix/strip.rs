//! The codegen-time slot-param strips: `scope_prefix::strip_scope_prefixes_for_slot_params`
//! (scan-based, every scope prefix, identifier must be a slot param) and
//! `slots::generate::strip_ctx_prefix_for_slot_params` (`str::replace`
//! per param, `_ctx.` only, no identifier boundary). Both are ported as
//! spelled — they differ, and the shipped bytes depend on which site runs
//! which.

use vize_s0::String;

use super::scope::PrefixScope;

const SLOT_PARAM_SCOPE_PREFIXES: [&str; 6] = [
    "_ctx.",
    "__props.",
    "$props.",
    "$setup.",
    "$data.",
    "$options.",
];

pub(super) fn contains_slot_param_scope_prefix(content: &str) -> bool {
    SLOT_PARAM_SCOPE_PREFIXES
        .iter()
        .any(|prefix| content.contains(prefix))
}

pub(super) fn strip_scope_prefixes_for_slot_params(
    scope: &PrefixScope<'_>,
    content: &str,
) -> String {
    if !contains_slot_param_scope_prefix(content) {
        return String::from(content);
    }
    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let mut stripped = false;
        for prefix in SLOT_PARAM_SCOPE_PREFIXES {
            let prefix_bytes = prefix.as_bytes();
            if i + prefix_bytes.len() > bytes.len()
                || &bytes[i..i + prefix_bytes.len()] != prefix_bytes
            {
                continue;
            }
            let start = i + prefix_bytes.len();
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'$')
            {
                end += 1;
            }
            let ident = &content[start..end];
            if !ident.is_empty() && scope.is_slot_param(ident) {
                result.push_str(ident);
                i = end;
                stripped = true;
                break;
            }
        }
        if stripped {
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

pub(super) fn strip_ctx_prefix_for_slot_params(scope: &PrefixScope<'_>, content: &str) -> String {
    let mut result = String::from(content);
    for param in scope.slot_params() {
        let mut prefixed = String::with_capacity(5 + param.len());
        prefixed.push_str("_ctx.");
        prefixed.push_str(param.as_str());
        let replaced = result.replace(prefixed.as_str(), param.as_str());
        result = String::from(replaced.as_str());
    }
    result
}
