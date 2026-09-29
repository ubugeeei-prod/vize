use super::{Path, PathBuf, String, Value};

pub(super) fn walk_glob(dir: &Path, parts: &[&str], out: &mut Vec<PathBuf>) {
    let Some((head, rest)) = parts.split_first() else {
        return;
    };
    if *head == "**" {
        walk_glob(dir, rest, out);
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if is_walkable_dir(&entry) {
                walk_glob(&entry.path(), parts, out);
            }
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !segment_match(head, &name) {
            continue;
        }
        let path = entry.path();
        if rest.is_empty() {
            if path.is_file() {
                out.push(path);
            }
            continue;
        }
        if is_walkable_dir(&entry) {
            walk_glob(&path, rest, out);
        }
    }
}

fn is_walkable_dir(entry: &std::fs::DirEntry) -> bool {
    let Ok(file_type) = entry.file_type() else {
        return false;
    };
    if !file_type.is_dir() {
        return false;
    }
    let Ok(name) = entry.file_name().into_string() else {
        return false;
    };
    !matches!(
        name.as_str(),
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

pub(super) fn glob_match(pattern: &str, relative: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let name_parts: Vec<&str> = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    glob_parts(&pattern_parts, &name_parts)
}

fn glob_parts(pattern: &[&str], name: &[&str]) -> bool {
    let Some((head, pattern_rest)) = pattern.split_first() else {
        return name.is_empty();
    };
    if *head == "**" {
        if glob_parts(pattern_rest, name) {
            return true;
        }
        return match name.split_first() {
            Some((_, name_rest)) => glob_parts(pattern, name_rest),
            None => false,
        };
    }
    let Some((name_head, name_rest)) = name.split_first() else {
        return false;
    };
    segment_match(head, name_head) && glob_parts(pattern_rest, name_rest)
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
                rendered.push_str(&prefix.as_os_str().to_string_lossy());
            }
        }
    }
    rendered
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
