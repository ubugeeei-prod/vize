use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};
use vize_carton::{SmallVec, String};

use super::css_scope;
use super::js_string::push_js_string_literal;

// The patterns are literals, so construction cannot fail; `None` would only
// disable the rewrite it drives and leave the CSS unchanged.
static CSS_IMPORT_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?m)^@import\s+(?:"([^"]+)"|'([^']+)');?\s*$"#).ok());
static CUSTOM_MEDIA_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"@custom-media\s+(--[\w-]+)\s+(.+?)\s*;").ok());
static CUSTOM_MEDIA_LINE_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?m)^@custom-media\s+[^;]+;\s*$").ok());

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
    resolve_css_imports_in(css, importer, alias_rules, is_dev, dev_url_base, None)
}

/// [`resolve_css_imports`] for a dev server that serves only `readable_roots`.
///
/// An `@import` is inlined, and a `url()` rewritten to `/@fs/`, only for a file
/// inside one of the roots (Vite's `server.fs.allow`). Anything else is left
/// as authored, for Vite's own resolver and its `server.fs` rules. `None`
/// reads every file the stylesheet names, as a build does.
pub fn resolve_css_imports_in(
    css: &str,
    importer: &str,
    alias_rules: &[CssAliasRule],
    is_dev: bool,
    dev_url_base: Option<&str>,
    readable_roots: Option<&[PathBuf]>,
) -> String {
    let readable = Readable::new(readable_roots);
    let mut custom_media: SmallVec<[CustomMedia; 4]> = SmallVec::new();
    let mut result = inline_css_imports(css, importer, alias_rules, &readable, &mut custom_media);

    parse_custom_media(result.as_str(), &mut custom_media);
    result = replace_regex_all(CUSTOM_MEDIA_LINE_RE.as_ref(), result.as_str(), "");

    for entry in &custom_media {
        let mut pattern = String::with_capacity(entry.name.len() + 2);
        pattern.push('(');
        pattern.push_str(entry.name.as_str());
        pattern.push(')');
        result = replace_literal(result.as_str(), pattern.as_str(), entry.query.as_str());
    }

    if is_dev {
        result = resolve_dev_urls(
            result.as_str(),
            importer,
            alias_rules,
            &readable,
            dev_url_base,
        );
    }

    result = css_scope::unwrap_deep_selectors(result.as_str());
    collapse_excess_blank_lines(result.as_str())
}

/// The files a stylesheet may read: all of them, or those inside some roots.
struct Readable {
    roots: Option<std::vec::Vec<PathBuf>>,
}

impl Readable {
    fn new(roots: Option<&[PathBuf]>) -> Self {
        let roots = roots.map(|roots| {
            roots
                .iter()
                .map(|root| std::fs::canonicalize(root).unwrap_or_else(|_| root.clone()))
                .collect()
        });
        Self { roots }
    }

    /// Whether `path` names an existing file this stylesheet may read.
    fn allows(&self, path: &Path) -> bool {
        let Some(roots) = &self.roots else {
            return path.exists();
        };
        std::fs::canonicalize(path)
            .is_ok_and(|real| roots.iter().any(|root| real.starts_with(root)))
    }
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
    readable: &Readable,
    custom_media: &mut SmallVec<[CustomMedia; 4]>,
) -> String {
    let mut output = String::with_capacity(css.len());
    let mut last = 0usize;

    for captures in CSS_IMPORT_RE.iter().flat_map(|re| re.captures_iter(css)) {
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
            .filter(|path| readable.allows(path))
            .and_then(|path| std::fs::read_to_string(path).ok());

        output.push_str(css.get(last..matched.start()).unwrap_or_default());
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

    output.push_str(css.get(last..).unwrap_or_default());
    output
}

fn parse_custom_media(css: &str, entries: &mut SmallVec<[CustomMedia; 4]>) {
    for captures in CUSTOM_MEDIA_RE.iter().flat_map(|re| re.captures_iter(css)) {
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
    readable: &Readable,
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

        let value = css.get(candidate.value_start..candidate.value_end);
        let trimmed = value.unwrap_or_default().trim();
        if should_skip_url(trimmed) {
            cursor = candidate.end;
            continue;
        }

        let replacement = resolve_css_path(trimmed, importer, alias_rules)
            .filter(|path| readable.allows(path))
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
            output.push_str(css.get(last..cursor).unwrap_or_default());
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

    output.push_str(css.get(last..).unwrap_or_default());
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
        Some(quote @ (b'"' | b'\'')) => {
            index += 1;
            Some(quote)
        }
        _ => None,
    };

    let value_start = index;
    let value_end = if let Some(quote) = quote {
        index += bytes.get(index..)?.iter().position(|&byte| byte == quote)?;
        index
    } else {
        index += bytes.get(index..)?.iter().position(|&byte| byte == b')')?;
        index
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

fn replace_regex_all(regex: Option<&Regex>, input: &str, replacement: &str) -> String {
    let Some(regex) = regex else {
        return String::from(input);
    };
    let replaced = regex.replace_all(input, replacement);
    String::from(replaced.as_ref())
}

fn replace_literal(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return String::from(input);
    }

    let mut output = String::with_capacity(input.len());
    let mut last = 0usize;
    for (start, _) in input.match_indices(needle) {
        output.push_str(input.get(last..start).unwrap_or_default());
        output.push_str(replacement);
        last = start + needle.len();
    }

    if last == 0 {
        return String::from(input);
    }

    output.push_str(input.get(last..).unwrap_or_default());
    output
}

fn collapse_excess_blank_lines(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while cursor < bytes.len() {
        let text_start = cursor;
        while bytes.get(cursor).is_some_and(|&byte| byte != b'\n') {
            cursor += 1;
        }
        output.push_str(input.get(text_start..cursor).unwrap_or_default());

        if cursor == bytes.len() {
            break;
        }

        let start = cursor;
        while bytes.get(cursor) == Some(&b'\n') {
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
mod tests;
