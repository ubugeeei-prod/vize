use super::walk::{glob_match, is_declaration_file, parse_jsonc, slash_path, walk_glob};
use super::{
    Path, PathBuf, String, ToCompactString, TsconfigOwnershipCache, TsconfigSourceKind, Value,
};
use rayon::prelude::*;

pub(super) fn ambient_program_files(project_root: &Path, tsconfig: &Path) -> Vec<String> {
    let selection = configured_program_files(tsconfig);
    let mut files = selection.declarations;
    let mut sources = selection.source_candidates;
    if selection.needs_authoritative_source_scan {
        sources.extend(owned_typescript_sources(project_root, tsconfig));
    }
    sources.sort();
    sources.dedup();
    let augmentations = sources
        .into_par_iter()
        .filter_map(|path| {
            if !is_typescript_source(&path) {
                return None;
            }
            let content = std::fs::read_to_string(&path).ok()?;
            is_module_augmentation(&content).then(|| {
                let canonical = std::fs::canonicalize(&path).unwrap_or(path);
                slash_path(&canonical)
            })
        })
        .collect::<Vec<_>>();
    files.extend(augmentations);
    files.sort();
    files.dedup();
    files
}

fn owned_typescript_sources(project_root: &Path, tsconfig: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for pattern in ["*.ts", "*.tsx", "*.mts", "*.cts"] {
        walk_glob(project_root, &["**", pattern], &mut candidates);
    }
    candidates.sort();
    candidates.dedup();
    let mut ownership = TsconfigOwnershipCache::default();
    candidates
        .into_iter()
        .filter(|path| ownership.project_owns_source(tsconfig, path, TsconfigSourceKind::Typed))
        .collect()
}

fn is_typescript_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "ts" | "tsx" | "mts" | "cts"))
}

fn is_module_augmentation(content: &str) -> bool {
    if !content.contains("declare module") && !content.contains("declare global") {
        return false;
    }
    has_top_level_import_or_export(content) && has_top_level_declare_augmentation(content)
}

fn has_top_level_import_or_export(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    let mut brace_depth = 0usize;
    let mut at_statement_start = true;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'{' => {
                brace_depth += 1;
                at_statement_start = false;
                index += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                at_statement_start = brace_depth == 0;
                index += 1;
            }
            b';' if brace_depth == 0 => {
                at_statement_start = true;
                index += 1;
            }
            byte if byte.is_ascii_whitespace() => index += 1,
            _ if brace_depth == 0 && at_statement_start => {
                if starts_with_statement_keyword(bytes, index, b"import")
                    || starts_with_statement_keyword(bytes, index, b"export")
                {
                    return true;
                }
                at_statement_start = false;
                index = skip_token(bytes, index);
            }
            _ => index += 1,
        }
    }
    false
}

fn has_top_level_declare_augmentation(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    let mut brace_depth = 0usize;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'{' => {
                brace_depth += 1;
                index += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                index += 1;
            }
            _ if brace_depth == 0 && starts_with_keyword(bytes, index, b"declare") => {
                let scope = skip_trivia(bytes, index + b"declare".len());
                if starts_with_keyword(bytes, scope, b"global") {
                    return true;
                }
                if starts_with_keyword(bytes, scope, b"module") {
                    let name = skip_trivia(bytes, scope + b"module".len());
                    if bytes
                        .get(name)
                        .is_some_and(|byte| matches!(byte, b'\'' | b'"'))
                    {
                        return true;
                    }
                }
                index = skip_token(bytes, index);
            }
            _ => index = skip_token(bytes, index),
        }
    }
    false
}

fn starts_with_statement_keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool {
    if !starts_with_keyword(bytes, index, keyword) {
        return false;
    }
    let after = index + keyword.len();
    match keyword {
        b"import" => bytes.get(after).is_some_and(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'{' | b'*' | b'\'' | b'"')
        }),
        b"export" => bytes
            .get(after)
            .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'{' | b'*' | b'=')),
        _ => false,
    }
}

fn starts_with_keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool {
    bytes
        .get(index..)
        .is_some_and(|rest| rest.starts_with(keyword))
        && bytes
            .get(index + keyword.len())
            .is_none_or(|byte| !is_identifier_continue(*byte))
}

fn skip_trivia(bytes: &[u8], mut index: usize) -> usize {
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            index = skip_line_comment(bytes, index + 2);
        } else if bytes.get(index..index + 2) == Some(b"/*") {
            index = skip_block_comment(bytes, index + 2);
        } else {
            return index;
        }
    }
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes.get(index) != Some(&b'\n') {
        index += 1;
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> usize {
    while index + 1 < bytes.len()
        && !(bytes.get(index) == Some(&b'*') && bytes.get(index + 1) == Some(&b'/'))
    {
        index += 1;
    }
    (index + 2).min(bytes.len())
}

fn skip_quoted(bytes: &[u8], mut index: usize) -> usize {
    let Some(&quote) = bytes.get(index) else {
        return bytes.len();
    };
    index += 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_token(bytes: &[u8], mut index: usize) -> usize {
    if !bytes
        .get(index)
        .is_some_and(|byte| is_identifier_start(*byte))
    {
        return index + 1;
    }
    index += 1;
    while bytes
        .get(index)
        .is_some_and(|byte| is_identifier_continue(*byte))
    {
        index += 1;
    }
    index
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

struct ProgramFileSelection {
    declarations: Vec<String>,
    source_candidates: Vec<PathBuf>,
    needs_authoritative_source_scan: bool,
}

fn configured_program_files(tsconfig: &Path) -> ProgramFileSelection {
    let empty = || ProgramFileSelection {
        declarations: Vec::new(),
        source_candidates: Vec::new(),
        needs_authoritative_source_scan: false,
    };
    let Some(base) = tsconfig.parent() else {
        return empty();
    };
    let Ok(content) = std::fs::read_to_string(tsconfig) else {
        return empty();
    };
    let Some(value) = parse_jsonc(&content) else {
        return empty();
    };
    let excludes = exclude_patterns(&value);
    let mut matched = Vec::new();
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        for entry in files.iter().filter_map(Value::as_str) {
            let path = resolve_config_path(base, entry);
            if path.is_file() {
                matched.push(path);
            }
        }
    }
    let mut include_needs_scan = false;
    if let Some(include) = value.get("include").and_then(Value::as_array) {
        for entry in include.iter().filter_map(Value::as_str) {
            include_needs_scan |= !entry.contains('*')
                && !entry.contains('?')
                && resolve_config_path(base, entry).is_dir();
            collect_include_entry(base, entry, &mut matched);
        }
    }
    if value.get("files").is_none() && value.get("include").is_none() {
        walk_glob(base, &["**", "*.d.ts"], &mut matched);
    }

    let mut declarations = Vec::new();
    let mut source_candidates = Vec::new();
    for path in matched {
        let relative = path.strip_prefix(base).unwrap_or(path.as_path());
        let relative = slash_path(relative);
        if is_excluded(relative.as_str(), &excludes) {
            continue;
        }
        if is_declaration_file(&path) {
            let absolute = std::fs::canonicalize(&path).unwrap_or(path);
            declarations.push(slash_path(&absolute));
        } else if is_typescript_source(&path) {
            source_candidates.push(path);
        }
    }
    declarations.sort();
    declarations.dedup();
    source_candidates.sort();
    source_candidates.dedup();
    ProgramFileSelection {
        declarations,
        source_candidates,
        needs_authoritative_source_scan: value.get("extends").is_some()
            || (value.get("files").is_none() && value.get("include").is_none())
            || include_needs_scan,
    }
}

#[cfg(test)]
mod tests {
    use super::ambient_program_files;
    use std::{fs, path::Path};

    fn write(root: &Path, path: &str, content: &str) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn project_program_adds_only_selected_external_module_augmentations() {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "tsconfig.json",
            r#"{"include":["src/**/*.ts"],"exclude":["src/excluded.ts"]}"#,
        );
        write(
            root.path(),
            "src/env.d.ts",
            "declare const build: string;\n",
        );
        write(
            root.path(),
            "src/augment.ts",
            "import './registry';\ndeclare module './registry' { interface Registry { extra: true } }\n",
        );
        write(
            root.path(),
            "src/global.ts",
            "export {};\ndeclare global { interface Window { widget: true } }\n",
        );
        write(
            root.path(),
            "src/comment.ts",
            "export const text = 'declare module not-a-declaration';\n",
        );
        write(
            root.path(),
            "src/script.ts",
            "declare module 'ambient-but-not-an-augmentation' {}\n",
        );
        write(
            root.path(),
            "src/excluded.ts",
            "export {}; declare global { interface Window { excluded: true } }\n",
        );

        let files = ambient_program_files(root.path(), &root.path().join("tsconfig.json"));
        let relative: Vec<_> = files
            .iter()
            .map(|path| {
                Path::new(path)
                    .strip_prefix(root.path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(
            relative,
            vec!["src/augment.ts", "src/env.d.ts", "src/global.ts"]
        );
    }

    #[test]
    fn inherited_program_membership_uses_the_tsconfig_ownership_authority() {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "base.json",
            r#"{"include":["src/**/*.ts"],"exclude":["src/excluded.ts"]}"#,
        );
        write(root.path(), "tsconfig.json", r#"{"extends":"./base.json"}"#);
        write(
            root.path(),
            "src/augment.ts",
            "export {}; declare global { interface Window { inherited: true } }\n",
        );
        write(
            root.path(),
            "src/excluded.ts",
            "export {}; declare global { interface Window { excluded: true } }\n",
        );

        let files = ambient_program_files(root.path(), &root.path().join("tsconfig.json"));
        let relative: Vec<_> = files
            .iter()
            .map(|path| {
                Path::new(path)
                    .strip_prefix(root.path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(relative, vec!["src/augment.ts"]);
    }
}

fn exclude_patterns(value: &Value) -> Vec<String> {
    match value.get("exclude") {
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(Value::as_str)
            .map(ToCompactString::to_compact_string)
            .collect(),
        Some(_) => Vec::new(),
        None => ["node_modules", "bower_components", "jspm_packages"]
            .into_iter()
            .map(ToCompactString::to_compact_string)
            .collect(),
    }
}

fn is_excluded(relative: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let mut prefix = relative;
    loop {
        if patterns
            .iter()
            .any(|pattern| glob_match(pattern.as_str(), prefix))
        {
            return true;
        }
        let Some((parent, _)) = prefix.rsplit_once('/') else {
            return false;
        };
        prefix = parent;
    }
}

fn collect_include_entry(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    if pattern.contains('*') || pattern.contains('?') {
        collect_glob(base, pattern, out);
        return;
    }
    let path = resolve_config_path(base, pattern);
    if path.is_file() {
        out.push(path);
    } else if path.is_dir() {
        walk_glob(&path, &["**", "*.d.ts"], out);
    }
}

fn resolve_config_path(base: &Path, pattern: &str) -> PathBuf {
    let pattern = pattern.trim_start_matches("./");
    let path = Path::new(pattern);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(pattern)
    }
}

fn collect_glob(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    let pattern = pattern.trim_start_matches("./");
    let absolute = Path::new(pattern).is_absolute();
    let mut root = if absolute {
        PathBuf::from("/")
    } else {
        base.to_path_buf()
    };
    let raw_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let mut index = 0usize;
    if absolute {
        while raw_parts
            .get(index)
            .is_some_and(|part| !part.contains('*') && !part.contains('?'))
        {
            let Some(part) = raw_parts.get(index) else {
                break;
            };
            root.push(part);
            index += 1;
        }
    }
    let Some(parts) = raw_parts.get(index..) else {
        return;
    };
    if parts.is_empty() {
        if root.is_file() {
            out.push(root);
        }
        return;
    }
    walk_glob(&root, parts, out);
}
