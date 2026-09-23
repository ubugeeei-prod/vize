//! Generated scope-prefix cleanup for SSR template locals.

use vize_s0::{FxHashSet, String, ToCompactString};

const SCOPED_PARAM_PREFIXES: [&str; 6] = [
    "_ctx.",
    "__props.",
    "$props.",
    "$setup.",
    "$data.",
    "$options.",
];

pub(crate) fn strip_scope_prefixes_for_scoped_params(
    scoped_params: &[FxHashSet<String>],
    content: &str,
) -> String {
    if scoped_params.is_empty()
        || !SCOPED_PARAM_PREFIXES
            .iter()
            .any(|prefix| content.contains(prefix))
    {
        return content.to_compact_string();
    }

    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let mut stripped = false;
        for prefix in SCOPED_PARAM_PREFIXES {
            let prefix_bytes = prefix.as_bytes();
            if index + prefix_bytes.len() > bytes.len()
                || &bytes[index..index + prefix_bytes.len()] != prefix_bytes
            {
                continue;
            }

            let start = index + prefix_bytes.len();
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'$')
            {
                end += 1;
            }

            let ident = &content[start..end];
            if !ident.is_empty() && is_scoped_param(scoped_params, ident) {
                result.push_str(ident);
                index = end;
                stripped = true;
                break;
            }
        }

        if stripped {
            continue;
        }

        // Copy a scalar, not a byte. A multibyte character such as `、`
        // would otherwise become one Latin-1 scalar per UTF-8 byte.
        let character = content[index..]
            .chars()
            .next()
            .expect("valid UTF-8 boundary");
        result.push(character);
        index += character.len_utf8();
    }

    result
}

fn is_scoped_param(scoped_params: &[FxHashSet<String>], name: &str) -> bool {
    scoped_params
        .iter()
        .rev()
        .any(|params| params.contains(name))
}
