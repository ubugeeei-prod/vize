//! Generated scope-prefix cleanup for template locals.

use super::super::context::CodegenContext;
use vize_s0::String;

const SLOT_PARAM_SCOPE_PREFIXES: [&str; 6] = [
    "_ctx.",
    "__props.",
    "$props.",
    "$setup.",
    "$data.",
    "$options.",
];

pub(crate) fn contains_slot_param_scope_prefix(content: &str) -> bool {
    SLOT_PARAM_SCOPE_PREFIXES
        .iter()
        .any(|prefix| content.contains(prefix))
}

pub(crate) fn strip_scope_prefixes_for_slot_params(ctx: &CodegenContext, content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let mut stripped = false;
        for prefix in SLOT_PARAM_SCOPE_PREFIXES {
            let prefix_bytes = prefix.as_bytes();
            if bytes.get(i..i + prefix_bytes.len()) != Some(prefix_bytes) {
                continue;
            }

            let start = i + prefix_bytes.len();
            let mut end = start;
            while bytes
                .get(end)
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
            {
                end += 1;
            }

            let ident = content.get(start..end).unwrap_or_default();
            if !ident.is_empty() && ctx.is_slot_param(ident) {
                result.push_str(ident);
                i = end;
                stripped = true;
                break;
            }
        }

        if stripped {
            continue;
        }

        let Some(character) = content.get(i..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        result.push(character);
        i += character.len_utf8();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::strip_scope_prefixes_for_slot_params;
    use crate::{codegen::CodegenContext, options::CodegenOptions};
    use vize_s0::String;

    #[test]
    fn preserves_utf8_while_stripping_scope_prefixes() {
        let mut ctx = CodegenContext::new(CodegenOptions::default());
        ctx.add_slot_params(&[String::new("i")]);

        let content = "`\u{2795} ${$setup.n}`";

        assert_eq!(strip_scope_prefixes_for_slot_params(&ctx, content), content);
        assert_eq!(strip_scope_prefixes_for_slot_params(&ctx, "$setup.i"), "i");
    }
}
