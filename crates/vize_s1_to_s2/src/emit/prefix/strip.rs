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
    let mut rest = content;
    'scan: while let Some(character) = rest.chars().next() {
        for prefix in SLOT_PARAM_SCOPE_PREFIXES {
            let Some(after) = rest.strip_prefix(prefix) else {
                continue;
            };
            let len = after
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
                .count();
            // The identifier is ASCII, so it ends on a char boundary.
            let Some((ident, tail)) = after.split_at_checked(len) else {
                continue;
            };
            if !ident.is_empty() && scope.is_slot_param(ident) {
                result.push_str(ident);
                rest = tail;
                continue 'scan;
            }
        }
        result.push(character);
        rest = rest.get(character.len_utf8()..).unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::{strip_ctx_prefix_for_slot_params, strip_scope_prefixes_for_slot_params};
    use crate::emit::prefix::scope::PrefixScope;

    #[test]
    fn preserves_utf8_when_prefixes_do_not_name_slot_params() {
        let mut scope = PrefixScope::new(None, true, false, false);
        scope.push_for([Some("i"), None, None]);

        let content = "`\u{2795} ${$setup.n}`";

        assert_eq!(
            strip_scope_prefixes_for_slot_params(&scope, content),
            content
        );
    }

    #[test]
    fn preserves_utf8_while_stripping_scope_prefixes() {
        let mut scope = PrefixScope::new(None, true, false, false);
        scope.push_for([Some("i"), None, None]);

        assert_eq!(
            strip_scope_prefixes_for_slot_params(&scope, "`\u{2795} ${$setup.i}`"),
            "`\u{2795} ${i}`"
        );
        assert_eq!(strip_ctx_prefix_for_slot_params(&scope, "_ctx.i"), "i");
    }
}
