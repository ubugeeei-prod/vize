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

        let ch = content[i..].chars().next().expect("valid UTF-8 boundary");
        result.push(ch);
        i += ch.len_utf8();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::strip_scope_prefixes_for_slot_params;
    use crate::codegen::CodegenContext;
    use crate::options::CodegenOptions;
    use vize_s0::String;

    // #5613: bytes outside a stripped prefix must be copied as UTF-8, not cast per byte.
    #[test]
    fn preserves_non_ascii_bytes_unrelated_to_the_stripped_scope_prefix() {
        let mut ctx = CodegenContext::new(CodegenOptions::default());
        ctx.add_slot_params(&[String::new("i")]);

        let content = "`\u{2795} ${$setup.n}`";
        let result = strip_scope_prefixes_for_slot_params(&ctx, content);

        assert_eq!(result, content);
    }

    #[test]
    fn still_strips_the_scope_prefix_for_an_actual_slot_param() {
        let mut ctx = CodegenContext::new(CodegenOptions::default());
        ctx.add_slot_params(&[String::new("i")]);

        let result = strip_scope_prefixes_for_slot_params(&ctx, "$setup.i");

        assert_eq!(result, "i");
    }
}
