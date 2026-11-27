use super::{Path, PathBuf, String, Value};
use vize_l0::FxHashSet;

/// A directory and how many pattern segments were still to match there.
type Walked = FxHashSet<(PathBuf, usize)>;

fn globstars<'p>(segments: impl Iterator<Item = &'p str>) -> usize {
    segments.filter(|segment| *segment == "**").count()
}

pub(super) fn walk_glob(dir: &Path, parts: &[&str], out: &mut Vec<PathBuf>) {
    // With two or more `**`, one directory is reached with the same remaining
    // pattern along many routes; walking each such pair once keeps the walk
    // proportional to the tree instead of to the number of routes.
    let mut walked = (globstars(parts.iter().copied()) > 1).then(Walked::default);
    walk_glob_from(dir, parts, out, &mut walked);
}

fn walk_glob_from(dir: &Path, parts: &[&str], out: &mut Vec<PathBuf>, walked: &mut Option<Walked>) {
    let Some((head, rest)) = parts.split_first() else {
        return;
    };
    if *head == "**" {
        if let Some(walked) = walked
            && !walked.insert((dir.to_path_buf(), parts.len()))
        {
            return;
        }
        walk_glob_from(dir, rest, out, walked);
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if is_walkable_dir(&entry) {
                walk_glob_from(&entry.path(), parts, out, walked);
            }
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if !segment_match(head, name) {
            continue;
        }
        let path = entry.path();
        if rest.is_empty() {
            if entry_names_file(&entry) {
                out.push(path);
            }
            continue;
        }
        if is_walkable_dir(&entry) {
            walk_glob_from(&path, rest, out, walked);
        }
    }
}

/// One directory walk for every TypeScript extension the authoritative scan accepts.
///
/// `**/*.{ts,tsx,mts,cts}` used to walk the tree once per extension, and each
/// `**` read every directory a second time. `.d.ts` stays included because its
/// extension is `ts`.
pub(super) fn collect_typescript_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if is_walkable_dir(&entry) {
            collect_typescript_sources(&entry.path(), out);
            continue;
        }
        if !entry_names_file(&entry) || !is_typescript_file_name(&entry.file_name()) {
            continue;
        }
        out.push(entry.path());
    }
}

fn entry_names_file(entry: &std::fs::DirEntry) -> bool {
    match entry.file_type() {
        Ok(file_type) if file_type.is_file() => true,
        Ok(file_type) if file_type.is_symlink() => entry.path().is_file(),
        Ok(_) => false,
        Err(_) => entry.path().is_file(),
    }
}

fn is_typescript_file_name(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    let Some((_, extension)) = name.rsplit_once('.') else {
        return false;
    };
    matches!(extension, "ts" | "tsx" | "mts" | "cts")
}

fn is_walkable_dir(entry: &std::fs::DirEntry) -> bool {
    let Ok(file_type) = entry.file_type() else {
        return false;
    };
    if !file_type.is_dir() {
        return false;
    }
    let file_name = entry.file_name();
    let Some(name) = file_name.to_str() else {
        return false;
    };
    !is_skipped_dir_name(name)
}

fn is_skipped_dir_name(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | "bower_components"
            | "jspm_packages"
            | ".vize"
            | ".git"
            | "target"
            | "dist"
    )
}

pub(super) fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

fn segment_match(pattern: &str, name: &str) -> bool {
    segment_match_bytes(pattern.as_bytes(), name.as_bytes())
}

fn segment_match_bytes(pattern: &[u8], name: &[u8]) -> bool {
    let mut pattern_index = 0usize;
    let mut name_index = 0usize;
    let mut star_pattern = None;
    let mut star_name = 0usize;
    while name_index < name.len() {
        let Some(name_byte) = name.get(name_index).copied() else {
            break;
        };
        if pattern
            .get(pattern_index)
            .is_some_and(|pattern_byte| *pattern_byte == b'?' || *pattern_byte == name_byte)
        {
            pattern_index += 1;
            name_index += 1;
            continue;
        }
        if pattern
            .get(pattern_index)
            .is_some_and(|pattern_byte| *pattern_byte == b'*')
        {
            star_pattern = Some(pattern_index);
            star_name = name_index;
            pattern_index += 1;
            continue;
        }
        if let Some(star) = star_pattern {
            pattern_index = star + 1;
            star_name += 1;
            name_index = star_name;
            continue;
        }
        return false;
    }
    while pattern.get(pattern_index).is_some_and(|byte| *byte == b'*') {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

/// Remaining pattern and name lengths that are known not to match.
type Unmatched = FxHashSet<(usize, usize)>;

pub(super) fn glob_match(pattern: &str, relative: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    // With two or more `**`, the same pair of remainders is retried along
    // many splits of the name; remembering the failed pairs tries each once.
    let mut unmatched = (globstars(pattern.split('/')) > 1).then(Unmatched::default);
    glob_match_parts(pattern, relative, &mut unmatched)
}

fn glob_match_parts(pattern: &str, name: &str, unmatched: &mut Option<Unmatched>) -> bool {
    let Some((head, pattern_rest)) = next_segment(pattern) else {
        return next_segment(name).is_none();
    };
    if head == "**" {
        let remainders = (pattern.len(), name.len());
        if unmatched
            .as_ref()
            .is_some_and(|unmatched| unmatched.contains(&remainders))
        {
            return false;
        }
        if glob_match_parts(pattern_rest, name, unmatched) {
            return true;
        }
        let matched = next_segment(name)
            .is_some_and(|(_, name_rest)| glob_match_parts(pattern, name_rest, unmatched));
        if !matched && let Some(unmatched) = unmatched {
            unmatched.insert(remainders);
        }
        return matched;
    }
    let Some((name_head, name_rest)) = next_segment(name) else {
        return false;
    };
    segment_match(head, name_head) && glob_match_parts(pattern_rest, name_rest, unmatched)
}

/// Next non-empty `/` segment. `a//b` matches `a/b`, and `**` may match empty.
fn next_segment(path: &str) -> Option<(&str, &str)> {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return None;
    }
    Some(path.split_once('/').unwrap_or((path, "")))
}

pub(super) fn slash_path(path: &Path) -> String {
    let mut rendered = String::new("");
    for component in path.components() {
        match component {
            std::path::Component::RootDir => rendered.push('/'),
            std::path::Component::Normal(part) => {
                if !rendered.is_empty() && !rendered.ends_with('/') {
                    rendered.push('/');
                }
                rendered.push_str(&part.to_string_lossy());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => rendered.push_str("/.."),
            std::path::Component::Prefix(prefix) => {
                push_program_prefix(&mut rendered, prefix.kind(), prefix.as_os_str());
            }
        }
    }
    rendered
}

/// Render a Windows path prefix the way TypeScript spells program roots.
/// Verbatim prefixes (`\\?\C:`, `\\?\UNC\server\share`) name the same file as
/// their plain form, but a `//?/C:/...` file name would be a different UNC
/// root to Corsa, so they are written as `C:` and `//server/share`.
fn push_program_prefix(rendered: &mut String, kind: std::path::Prefix<'_>, raw: &std::ffi::OsStr) {
    match kind {
        std::path::Prefix::VerbatimDisk(drive) => {
            rendered.push(char::from(drive));
            rendered.push(':');
        }
        std::path::Prefix::VerbatimUNC(server, share) | std::path::Prefix::UNC(server, share) => {
            rendered.push_str("//");
            rendered.push_str(&server.to_string_lossy());
            rendered.push('/');
            rendered.push_str(&share.to_string_lossy());
        }
        _ => rendered.push_str(&raw.to_string_lossy()),
    }
}

pub(super) fn parse_jsonc(content: &str) -> Option<Value> {
    let stripped = strip_json_comments(content);
    let normalized = strip_trailing_commas(stripped.as_str());
    serde_json::from_str(normalized.as_str()).ok()
}

fn strip_json_comments(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
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
            line_comment = true;
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            block_comment = true;
            continue;
        }
        output.push(ch);
    }
    output
}

fn strip_trailing_commas(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(ch) = chars.next() {
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
        if ch == ',' {
            let mut lookahead = chars.clone();
            let next = loop {
                match lookahead.next() {
                    Some(next) if next.is_whitespace() => continue,
                    other => break other,
                }
            };
            if matches!(next, Some('}' | ']')) {
                continue;
            }
        }
        output.push(ch);
    }
    output
}

#[cfg(test)]
mod tests;
