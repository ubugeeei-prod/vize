//! A literal `vue` specifier is not package authority when project config rewrites it.

use std::fs;
use std::path::Path;

/// Type-side `compilerOptions.paths` or a Vite `resolve.alias` sends `vue`
/// somewhere other than the Vue package.
pub(crate) fn vue_specifier_is_redirected(root: &Path) -> bool {
    tsconfig_redirects_vue(root) || vite_redirects_vue(root)
}

fn tsconfig_redirects_vue(root: &Path) -> bool {
    let Ok(text) = fs::read_to_string(root.join("tsconfig.json")) else {
        return false;
    };
    let Some(paths_at) = text.find("\"paths\"") else {
        return false;
    };
    let after = text[paths_at + "\"paths\"".len()..].trim_start();
    let Some(object) = after.strip_prefix(':').map(str::trim_start) else {
        return false;
    };
    object.starts_with('{') && object_redirects_vue(object)
}

fn text_names_vue_package(target: &str) -> bool {
    let target = target.trim().trim_end_matches(['*', '/', '\\']);
    target == "vue"
        || target == "node_modules/vue"
        || target == "node_modules\\vue"
        || target.ends_with("/node_modules/vue")
        || target.ends_with("\\node_modules\\vue")
}

fn vite_redirects_vue(root: &Path) -> bool {
    const NAMES: [&str; 6] = [
        "vite.config.js",
        "vite.config.mjs",
        "vite.config.cjs",
        "vite.config.ts",
        "vite.config.mts",
        "vite.config.cts",
    ];
    NAMES.iter().any(|name| {
        fs::read_to_string(root.join(name)).is_ok_and(|text| alias_config_redirects_vue(&text))
    })
}

pub(crate) fn alias_config_redirects_vue(text: &str) -> bool {
    let mut rest = text;
    while let Some(found) = rest.find("alias") {
        let start = found;
        let end = start + "alias".len();
        if !ident_boundary(rest, start, end) {
            rest = &rest[end..];
            continue;
        }
        let after = rest[end..].trim_start();
        if let Some(body) = after.strip_prefix(':').map(str::trim_start) {
            if (body.starts_with('{') && object_redirects_vue(body))
                || (body.starts_with('[') && array_redirects_vue(body))
            {
                return true;
            }
        }
        rest = &rest[end..];
    }
    false
}

fn ident_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = start == 0
        || text[..start]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '$');
    let after_ok = text[end..]
        .chars()
        .next()
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '$');
    before_ok && after_ok
}

fn object_redirects_vue(object: &str) -> bool {
    let Some(body) = balanced_body(object, '{', '}') else {
        return false;
    };
    let mut depth: i32 = 0;
    let mut index = 0;
    while index < body.len() {
        let ch = body[index..].chars().next().unwrap_or('\0');
        let width = ch.len_utf8();
        if ch == '"' || ch == '\'' || ch == '`' {
            let Some(literal) = skip_string(&body[index..]) else {
                return false;
            };
            if depth == 0 {
                let raw = &body[index + 1..index + literal - 1];
                if raw == "vue" && value_after_key_is_redirect(&body[index + literal..]) {
                    return true;
                }
            }
            index += literal;
            continue;
        }
        if ch == '{' || ch == '[' || ch == '(' {
            depth += 1;
        } else if ch == '}' || ch == ']' || ch == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && is_ident_char(ch) {
            let ident = ident_at(&body[index..]);
            if ident == "vue" && value_after_key_is_redirect(&body[index + ident.len()..]) {
                return true;
            }
            index += ident.len();
            continue;
        }
        index += width;
    }
    false
}

fn value_after_key_is_redirect(after_key: &str) -> bool {
    let value = after_key.trim_start();
    value.starts_with(':') && !value_text_is_vue_package(value[1..].trim_start())
}

fn array_redirects_vue(array: &str) -> bool {
    let Some(body) = balanced_body(array, '[', ']') else {
        return false;
    };
    for key in ["find", "\"find\"", "'find'"] {
        let mut rest = body;
        while let Some(found) = rest.find(key) {
            let end = found + key.len();
            if !ident_boundary(rest, found, end) && !key.starts_with(['"', '\'']) {
                rest = &rest[end..];
                continue;
            }
            let after = rest[end..].trim_start();
            let Some(value) = after.strip_prefix(':').map(str::trim_start) else {
                rest = &rest[end..];
                continue;
            };
            if value.starts_with("\"vue\"")
                || value.starts_with("'vue'")
                || value.starts_with("`vue`")
            {
                return replacement_is_redirect(rest);
            }
            rest = &rest[end..];
        }
    }
    false
}

fn replacement_is_redirect(entry: &str) -> bool {
    let Some(found) = entry.find("replacement") else {
        return true;
    };
    let after = entry[found + "replacement".len()..].trim_start();
    let Some(value) = after.strip_prefix(':').map(str::trim_start) else {
        return true;
    };
    !value_text_is_vue_package(value)
}

fn value_text_is_vue_package(value: &str) -> bool {
    let value = value.trim_start();
    if let Some(body) = value
        .strip_prefix('[')
        .and_then(|_| balanced_body(value, '[', ']'))
    {
        let mut saw_target = false;
        let mut index = 0;
        while index < body.len() {
            let ch = body[index..].chars().next().unwrap_or('\0');
            if ch == '"' || ch == '\'' {
                let Some(literal) = skip_string(&body[index..]) else {
                    return false;
                };
                let raw = &body[index + 1..index + literal - 1];
                if !text_names_vue_package(raw) {
                    return false;
                }
                saw_target = true;
                index += literal;
                continue;
            }
            index += ch.len_utf8();
        }
        return saw_target;
    }
    if let Some(literal) = quoted_literal(value) {
        return text_names_vue_package(literal);
    }
    false
}

fn quoted_literal(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return None;
    }
    let rest = &value[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn ident_at(text: &str) -> &str {
    let end = text
        .char_indices()
        .find(|(_, ch)| !is_ident_char(*ch))
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    &text[..end]
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

fn skip_string(text: &str) -> Option<usize> {
    let quote = text.chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return None;
    }
    let mut escaped = false;
    for (index, ch) in text.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(index + ch.len_utf8());
        }
    }
    None
}

fn balanced_body(text: &str, open: char, close: char) -> Option<&str> {
    if !text.starts_with(open) {
        return None;
    }
    let mut depth = 0;
    let mut in_string = None;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                in_string = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = Some(ch);
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(&text[open.len_utf8()..index]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{alias_config_redirects_vue, vue_specifier_is_redirected};
    use std::fs;

    #[test]
    fn literal_vue_alias_to_a_local_file_is_not_package_authority() {
        let vite = "export default { resolve: { alias: { vue: new URL(\"./src/fake-vue.ts\", import.meta.url).pathname } } };\n";
        assert!(alias_config_redirects_vue(vite));
        assert!(alias_config_redirects_vue(
            "export default { resolve: { alias: [{ find: \"vue\", replacement: \"./src/fake-vue.ts\" }] } };",
        ));
        assert!(!alias_config_redirects_vue(
            "export default { resolve: { alias: { vue: \"vue\" } } };",
        ));
        assert!(!alias_config_redirects_vue(
            "export default { resolve: { alias: { \"@\": \"./src\" } } };",
        ));
    }

    #[test]
    fn tsconfig_and_vite_files_redirect_only_the_vue_specifier() {
        let root = std::env::temp_dir().join(format!("vize-vue-authority-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        assert!(!vue_specifier_is_redirected(&root));
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"vue":["src/fake-vue.ts"]}}}"#,
        )
        .unwrap();
        assert!(vue_specifier_is_redirected(&root));
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"vue":["node_modules/vue"]}}}"#,
        )
        .unwrap();
        assert!(!vue_specifier_is_redirected(&root));
        fs::write(
            root.join("vite.config.mjs"),
            "export default { resolve: { alias: { vue: new URL(\"./src/fake-vue.ts\", import.meta.url).pathname } } };\n",
        )
        .unwrap();
        assert!(vue_specifier_is_redirected(&root));
        let _ = fs::remove_dir_all(&root);
    }
}
