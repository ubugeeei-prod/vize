use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};
use vize_carton::{SmallVec, String};

use super::css_scope;
use super::js_string::push_js_string_literal;

static CSS_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^@import\s+(?:"([^"]+)"|'([^']+)');?\s*$"#).expect("valid css import regex")
});
static CUSTOM_MEDIA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"@custom-media\s+(--[\w-]+)\s+(.+?)\s*;").expect("valid custom media regex")
});
static CUSTOM_MEDIA_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^@custom-media\s+[^;]+;\s*$").expect("valid custom media line regex")
});

/// Serializable CSS alias rule used by the Vite plugin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAliasRule {
    /// String alias key or regex source.
    pub find: String,
    /// Replacement target.
    pub replacement: String,
    /// Whether `find` should be interpreted as a regex source.
    pub is_regex: bool,
    /// JavaScript regex flags. Global/sticky are ignored for stable matching.
    pub flags: Option<String>,
}

/// Scope CSS with Vize's Vite pipeline selector semantics.
pub fn scope_css_for_pipeline(css: &str, scope_id: &str) -> String {
    css_scope::scope_css_for_pipeline(css, scope_id)
}

/// Resolve CSS imports, custom media, dev asset URLs, and Vue deep selectors.
pub fn resolve_css_imports(
    css: &str,
    importer: &str,
    alias_rules: &[CssAliasRule],
    is_dev: bool,
    dev_url_base: Option<&str>,
) -> String {
    let mut custom_media: SmallVec<[CustomMedia; 4]> = SmallVec::new();
    let mut result = inline_css_imports(css, importer, alias_rules, &mut custom_media);

    parse_custom_media(result.as_str(), &mut custom_media);
    result = replace_regex_all(&CUSTOM_MEDIA_LINE_RE, result.as_str(), "");

    for entry in &custom_media {
        let mut pattern = String::with_capacity(entry.name.len() + 2);
        pattern.push('(');
        pattern.push_str(entry.name.as_str());
        pattern.push(')');
        result = replace_literal(result.as_str(), pattern.as_str(), entry.query.as_str());
    }

    if is_dev {
        result = resolve_dev_urls(result.as_str(), importer, alias_rules, dev_url_base);
    }

    result = css_scope::unwrap_deep_selectors(result.as_str());
    collapse_excess_blank_lines(result.as_str())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CustomMedia {
    name: String,
    query: String,
}

fn inline_css_imports(
    css: &str,
    importer: &str,
    alias_rules: &[CssAliasRule],
    custom_media: &mut SmallVec<[CustomMedia; 4]>,
) -> String {
    let mut output = String::with_capacity(css.len());
    let mut last = 0usize;

    for captures in CSS_IMPORT_RE.captures_iter(css) {
        let Some(matched) = captures.get(0) else {
            continue;
        };
        let import_path = captures
            .get(1)
            .or_else(|| captures.get(2))
            .map(|capture| capture.as_str());
        let Some(import_path) = import_path else {
            continue;
        };

        let replacement = resolve_css_path(import_path, importer, alias_rules)
            .filter(|path| path.exists())
            .and_then(|path| std::fs::read_to_string(path).ok());

        output.push_str(&css[last..matched.start()]);
        if let Some(content) = replacement {
            parse_custom_media(content.as_str(), custom_media);
            output.push_str(content.as_str());
        } else {
            output.push_str(matched.as_str());
        }
        last = matched.end();
    }

    if last == 0 {
        return String::from(css);
    }

    output.push_str(&css[last..]);
    output
}

fn parse_custom_media(css: &str, entries: &mut SmallVec<[CustomMedia; 4]>) {
    for captures in CUSTOM_MEDIA_RE.captures_iter(css) {
        let Some(name) = captures.get(1).map(|capture| capture.as_str()) else {
            continue;
        };
        let Some(query) = captures.get(2).map(|capture| capture.as_str()) else {
            continue;
        };

        if let Some(entry) = entries.iter_mut().find(|entry| entry.name.as_str() == name) {
            entry.query = String::from(query);
        } else {
            entries.push(CustomMedia {
                name: String::from(name),
                query: String::from(query),
            });
        }
    }
}

fn resolve_dev_urls(
    css: &str,
    importer: &str,
    alias_rules: &[CssAliasRule],
    dev_url_base: Option<&str>,
) -> String {
    let bytes = css.as_bytes();
    let mut output = String::with_capacity(css.len());
    let mut cursor = 0usize;
    let mut last = 0usize;

    while cursor + "url(".len() <= bytes.len() {
        if bytes.get(cursor..cursor + "url(".len()) != Some(&b"url("[..]) {
            cursor += 1;
            continue;
        }

        let Some(candidate) = parse_url_candidate(css, cursor) else {
            cursor += 1;
            continue;
        };

        let trimmed = css[candidate.value_start..candidate.value_end].trim();
        if should_skip_url(trimmed) {
            cursor = candidate.end;
            continue;
        }

        let replacement = resolve_css_path(trimmed, importer, alias_rules)
            .filter(|path| path.exists())
            .map(|path| {
                let normalized = normalize_path_for_url(&path);
                let base = dev_url_base.unwrap_or("/");
                let mut url = String::with_capacity(base.len() + normalized.len() + 4);
                url.push_str(if base.ends_with('/') { base } else { "" });
                if !base.ends_with('/') {
                    url.push_str(base);
                    url.push('/');
                }
                url.push_str("@fs");
                url.push_str(normalized.as_str());
                url
            });

        if let Some(replacement) = replacement {
            output.push_str(&css[last..cursor]);
            output.push_str("url(");
            push_js_string_literal(&mut output, replacement.as_str());
            output.push(')');
            last = candidate.end;
        }

        cursor = candidate.end;
    }

    if last == 0 {
        return String::from(css);
    }

    output.push_str(&css[last..]);
    output
}

struct UrlCandidate {
    value_start: usize,
    value_end: usize,
    end: usize,
}

fn parse_url_candidate(css: &str, cursor: usize) -> Option<UrlCandidate> {
    let bytes = css.as_bytes();
    let mut index = cursor + "url(".len();
    while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        index += 1;
    }

    let quote = match bytes.get(index).copied() {
        Some(b'"' | b'\'') => {
            let quote = bytes[index];
            index += 1;
            Some(quote)
        }
        _ => None,
    };

    let value_start = index;
    let value_end = if let Some(quote) = quote {
        while index < bytes.len() && bytes[index] != quote {
            index += 1;
        }
        (index < bytes.len()).then_some(index)?
    } else {
        while index < bytes.len() && bytes[index] != b')' {
            index += 1;
        }
        (index < bytes.len()).then_some(index)?
    };

    if quote.is_some() {
        index += 1;
    }
    while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        index += 1;
    }
    (bytes.get(index) == Some(&b')')).then_some(UrlCandidate {
        value_start,
        value_end,
        end: index + 1,
    })
}

fn should_skip_url(value: &str) -> bool {
    value.starts_with("data:")
        || value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("/@fs/")
}

fn resolve_css_path(
    import_path: &str,
    importer: &str,
    alias_rules: &[CssAliasRule],
) -> Option<PathBuf> {
    for rule in alias_rules {
        if let Some(resolved) = resolve_alias_path(import_path, rule) {
            return Some(absolutize_path(Path::new(resolved.as_str())));
        }
    }

    if import_path.starts_with('.') {
        return Path::new(importer)
            .parent()
            .map(|dir| absolutize_path(&dir.join(import_path)));
    }

    let path = Path::new(import_path);
    path.is_absolute().then(|| path.to_path_buf())
}

fn resolve_alias_path(import_path: &str, rule: &CssAliasRule) -> Option<String> {
    if rule.is_regex {
        let pattern = build_alias_regex(rule)?;
        if !pattern.is_match(import_path) {
            return None;
        }
        let replaced = pattern.replace(import_path, rule.replacement.as_str());
        return Some(String::from(replaced.as_ref()));
    }

    let suffix = matched_alias_suffix(import_path, rule.find.as_str())?;
    let mut resolved = String::with_capacity(rule.replacement.len() + suffix.len() + 1);
    resolved.push_str(rule.replacement.as_str());
    if !suffix.is_empty() {
        if !resolved.ends_with(std::path::MAIN_SEPARATOR) && !resolved.ends_with('/') {
            resolved.push(std::path::MAIN_SEPARATOR);
        }
        resolved.push_str(suffix.as_str());
    }
    Some(resolved)
}

fn build_alias_regex(rule: &CssAliasRule) -> Option<Regex> {
    let mut builder = RegexBuilder::new(rule.find.as_str());
    if let Some(flags) = rule.flags.as_deref() {
        builder
            .case_insensitive(flags.as_bytes().contains(&b'i'))
            .multi_line(flags.as_bytes().contains(&b'm'))
            .dot_matches_new_line(flags.as_bytes().contains(&b's'));
    }
    builder.build().ok()
}

fn matched_alias_suffix(import_path: &str, find: &str) -> Option<String> {
    if import_path == find {
        return Some(String::default());
    }

    if find.ends_with('/') {
        return import_path.strip_prefix(find).map(String::from);
    }

    import_path
        .strip_prefix(find)
        .and_then(|suffix| suffix.strip_prefix('/'))
        .map(String::from)
}

fn absolutize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| PathBuf::from(path))
}

fn normalize_path_for_url(path: &Path) -> String {
    let value = path.to_string_lossy();
    if !value.as_bytes().contains(&b'\\') {
        return String::from(value.as_ref());
    }
    let mut output = String::with_capacity(value.len());
    for char in value.chars() {
        output.push(if char == '\\' { '/' } else { char });
    }
    output
}

fn replace_regex_all(regex: &Regex, input: &str, replacement: &str) -> String {
    let replaced = regex.replace_all(input, replacement);
    String::from(replaced.as_ref())
}

fn replace_literal(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return String::from(input);
    }

    let mut output = String::with_capacity(input.len());
    let mut last = 0usize;
    let mut cursor = 0usize;
    while let Some(offset) = input[cursor..].find(needle) {
        let start = cursor + offset;
        output.push_str(&input[last..start]);
        output.push_str(replacement);
        cursor = start + needle.len();
        last = cursor;
    }

    if last == 0 {
        return String::from(input);
    }

    output.push_str(&input[last..]);
    output
}

fn collapse_excess_blank_lines(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while cursor < bytes.len() {
        if bytes[cursor] != b'\n' {
            output.push(bytes[cursor] as char);
            cursor += 1;
            continue;
        }

        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] == b'\n' {
            cursor += 1;
        }
        let count = cursor - start;
        output.push('\n');
        if count >= 2 {
            output.push('\n');
        }
        changed |= count >= 3;
    }

    if changed { output } else { String::from(input) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_custom_media_and_deep_selectors() {
        let css = "@custom-media --mobile (max-width: 768px);\n.foo { color: red; }\n@media (--mobile) { .foo :deep(.bar) { color: blue; } }";
        let result = resolve_css_imports(css, "/project/Component.vue", &[], false, None);

        assert!(!result.contains("@custom-media"));
        assert!(result.contains("@media (max-width: 768px)"));
        assert!(result.contains(".foo .bar"));
    }

    #[test]
    fn string_alias_does_not_match_package_prefix() {
        assert_eq!(
            matched_alias_suffix("@/asset.svg", "@").as_deref(),
            Some("asset.svg")
        );
        assert_eq!(matched_alias_suffix("@scope/icon.svg", "@"), None);
    }
}
