//! A literal `vue` specifier is not package authority when project config rewrites it.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::String;

/// Type-side `compilerOptions.paths` or a Vite `resolve.alias` sends `vue`
/// somewhere other than the Vue package.
pub(crate) fn vue_specifier_is_redirected(root: &Path) -> bool {
    tsconfig_redirects_vue(root) || vite_redirects_vue(root)
}

fn tsconfig_redirects_vue(root: &Path) -> bool {
    matches!(
        tsconfig_vue_paths(&root.join("tsconfig.json"), 0, &mut Vec::new()),
        VuePaths::Redirected
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VuePaths {
    Absent,
    Package,
    Redirected,
}

fn tsconfig_vue_paths(path: &Path, depth: u8, visited: &mut Vec<PathBuf>) -> VuePaths {
    if depth == 8 {
        return VuePaths::Absent;
    }
    let identity = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if visited.contains(&identity) {
        return VuePaths::Absent;
    }
    let Ok(text) = fs::read_to_string(path) else {
        return VuePaths::Absent;
    };
    let Some(config) = parse_jsonc(&text) else {
        return VuePaths::Absent;
    };
    visited.push(identity);

    if let Some(paths) = config
        .get("compilerOptions")
        .and_then(|options| options.get("paths"))
        .and_then(Value::as_object)
    {
        return match paths.get("vue") {
            Some(targets) if json_targets_name_vue_package(targets) => VuePaths::Package,
            Some(_) => VuePaths::Redirected,
            // `paths` is one compiler option and replaces the inherited value.
            None => VuePaths::Package,
        };
    }

    let Some(parent) = path.parent() else {
        return VuePaths::Absent;
    };
    let extended = match config.get("extends") {
        Some(Value::String(specifier)) => vec![specifier.as_str()],
        Some(Value::Array(specifiers)) => specifiers.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    for specifier in extended.into_iter().rev() {
        let Some(extended_path) = relative_tsconfig_path(parent, specifier) else {
            continue;
        };
        let inherited = tsconfig_vue_paths(&extended_path, depth + 1, visited);
        if inherited != VuePaths::Absent {
            return inherited;
        }
    }
    VuePaths::Absent
}

fn relative_tsconfig_path(parent: &Path, specifier: &str) -> Option<PathBuf> {
    if !specifier.starts_with('.') {
        return None;
    }
    let candidate = parent.join(specifier);
    if candidate.is_file() {
        return Some(candidate);
    }
    let mut json_name = candidate.as_os_str().to_os_string();
    json_name.push(".json");
    let json = PathBuf::from(json_name);
    if json.is_file() {
        Some(json)
    } else {
        let nested = candidate.join("tsconfig.json");
        nested.is_file().then_some(nested)
    }
}

fn json_targets_name_vue_package(targets: &Value) -> bool {
    match targets {
        Value::String(target) => tsconfig_target_names_vue_package(target),
        Value::Array(targets) => {
            !targets.is_empty()
                && targets.iter().all(|target| {
                    target
                        .as_str()
                        .is_some_and(tsconfig_target_names_vue_package)
                })
        }
        _ => false,
    }
}

fn tsconfig_target_names_vue_package(target: &str) -> bool {
    let target = target.trim().trim_end_matches(['*', '/', '\\']);
    target == "node_modules/vue"
        || target == "node_modules\\vue"
        || target.ends_with("/node_modules/vue")
        || target.ends_with("\\node_modules\\vue")
}

fn parse_jsonc(text: &str) -> Option<Value> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let without_comments = strip_json_comments(text)?;
    let normalized = strip_trailing_commas(&without_comments);
    serde_json::from_str(&normalized).ok()
}

fn strip_json_comments(text: &str) -> Option<String> {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            }
            continue;
        }

        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            }
            continue;
        }

        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            output.push(' ');
            line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            output.push(' ');
            block_comment = true;
            continue;
        }

        output.push(ch);
    }

    (!block_comment).then_some(output)
}

fn strip_trailing_commas(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(&ch) = chars.get(index) {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut next = index + 1;
            while chars.get(next).is_some_and(|ch| ch.is_whitespace()) {
                next += 1;
            }
            if chars.get(next).is_some_and(|ch| matches!(ch, '}' | ']')) {
                index += 1;
                continue;
            }
        }

        output.push(ch);
        index += 1;
    }

    output
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
            rest = tail(rest, end);
            continue;
        }
        let after = tail(rest, end).trim_start();
        if let Some(body) = after.strip_prefix(':').map(str::trim_start)
            && ((body.starts_with('{') && object_redirects_vue(body))
                || (body.starts_with('[') && array_redirects_vue(body)))
        {
            return true;
        }
        rest = tail(rest, end);
    }
    false
}

fn ident_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = start == 0
        || head(text, start)
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '$');
    let after_ok = tail(text, end)
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
        let ch = tail(body, index).chars().next().unwrap_or('\0');
        let width = ch.len_utf8();
        if ch == '"' || ch == '\'' || ch == '`' {
            let Some(literal) = skip_string(tail(body, index)) else {
                return false;
            };
            if depth == 0 {
                let raw = middle(body, index + 1, index + literal - 1);
                if raw == "vue" && value_after_key_is_redirect(tail(body, index + literal)) {
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
            let ident = ident_at(tail(body, index));
            if ident == "vue" && value_after_key_is_redirect(tail(body, index + ident.len())) {
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
    value.starts_with(':') && !value_text_is_vue_package(tail(value, 1).trim_start())
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
                rest = tail(rest, end);
                continue;
            }
            let after = tail(rest, end).trim_start();
            let Some(value) = after.strip_prefix(':').map(str::trim_start) else {
                rest = tail(rest, end);
                continue;
            };
            if value.starts_with("\"vue\"")
                || value.starts_with("'vue'")
                || value.starts_with("`vue`")
            {
                return replacement_is_redirect(rest);
            }
            rest = tail(rest, end);
        }
    }
    false
}

fn replacement_is_redirect(entry: &str) -> bool {
    let Some(found) = entry.find("replacement") else {
        return true;
    };
    let after = tail(entry, found + "replacement".len()).trim_start();
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
            let ch = tail(body, index).chars().next().unwrap_or('\0');
            if ch == '"' || ch == '\'' {
                let Some(literal) = skip_string(tail(body, index)) else {
                    return false;
                };
                let raw = middle(body, index + 1, index + literal - 1);
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
    let rest = tail(value, quote.len_utf8());
    let end = rest.find(quote)?;
    Some(head(rest, end))
}

fn ident_at(text: &str) -> &str {
    let end = text
        .char_indices()
        .find(|(_, ch)| !is_ident_char(*ch))
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    head(text, end)
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
                return Some(middle(text, open.len_utf8(), index));
            }
        }
    }
    None
}

#[inline]
fn head(text: &str, end: usize) -> &str {
    text.get(..end).unwrap_or_default()
}

#[inline]
fn tail(text: &str, start: usize) -> &str {
    text.get(start..).unwrap_or_default()
}

#[inline]
fn middle(text: &str, start: usize, end: usize) -> &str {
    text.get(start..end).unwrap_or_default()
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
        assert!(alias_config_redirects_vue(
            "const 日本語 = true; export default { resolve: { alias: { vue: \"./src/偽-vue.ts\" } } };",
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
            root.join("tsconfig.base.json"),
            concat!(
                "\u{feff}",
                r#"{
                    // Comment-like path text must remain data.
                    "compilerOptions": {
                        "paths": {
                            "@/*": ["https://example.test/*"],
                            "vue": ["src/日本語/fake-vue.ts",],
                        },
                    },
                }"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{
                /* TypeScript accepts comments and trailing commas. */
                "extends": "./tsconfig.base",
            }"#,
        )
        .unwrap();
        assert!(vue_specifier_is_redirected(&root));

        fs::write(
            root.join("tsconfig.json"),
            r#"{
                "extends": "./tsconfig.base",
                "compilerOptions": { "paths": { "vue": ["node_modules/vue",], }, },
            }"#,
        )
        .unwrap();
        assert!(!vue_specifier_is_redirected(&root));

        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions": { /* unterminated"#,
        )
        .unwrap();
        assert!(!vue_specifier_is_redirected(&root));

        fs::write(
            root.join("tsconfig.json"),
            r#"{
                "extends": "./tsconfig.base",
                "compilerOptions": { "paths": { "@/*": ["src/*"] } },
            }"#,
        )
        .unwrap();
        assert!(!vue_specifier_is_redirected(&root));

        fs::write(
            root.join("tsconfig.package.json"),
            r#"{"compilerOptions":{"paths":{"vue":["node_modules/vue"]}}}"#,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.redirect.json"),
            r#"{"compilerOptions":{"paths":{"vue":["src/fake-vue.ts"]}}}"#,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{"extends":["./tsconfig.redirect.json","./tsconfig.package.json"]}"#,
        )
        .unwrap();
        assert!(!vue_specifier_is_redirected(&root));
        fs::write(
            root.join("tsconfig.json"),
            r#"{"extends":["./tsconfig.package.json","./tsconfig.redirect.json"]}"#,
        )
        .unwrap();
        assert!(vue_specifier_is_redirected(&root));

        fs::write(
            root.join("tsconfig.cycle-a.json"),
            r#"{"extends":"./tsconfig.cycle-b.json"}"#,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.cycle-b.json"),
            r#"{"extends":"./tsconfig.cycle-a.json"}"#,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{"extends":"./tsconfig.cycle-a.json"}"#,
        )
        .unwrap();
        assert!(!vue_specifier_is_redirected(&root));

        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"vue":["src/fake-vue.ts"]}}}"#,
        )
        .unwrap();
        assert!(vue_specifier_is_redirected(&root));
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":"src","paths":{"vue":["vue"]}}}"#,
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
