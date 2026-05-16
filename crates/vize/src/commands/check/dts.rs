//! Helpers for lightweight `.d.ts` parsing used by `vize check`.

use std::{fs, path::Path};

use vize_carton::{String, ToCompactString};

pub(super) fn parse_interface_members(
    path: &Path,
    interface_name: &str,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(parse_interface_members_content(&content, interface_name))
}

pub(super) fn parse_interface_members_content(
    content: &str,
    interface_name: &str,
) -> Vec<(String, String)> {
    let mut members = Vec::new();
    let mut in_interface = false;
    let mut brace_depth = 0i32;
    let mut current_name: Option<String> = None;
    let mut current_type = String::default();

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_interface {
            if trimmed.contains(interface_name) {
                in_interface = true;
                brace_depth += brace_delta(trimmed);
            }
            continue;
        }

        brace_depth += brace_delta(trimmed);
        if brace_depth <= 0 {
            flush_pending_member(&mut members, &mut current_name, &mut current_type);
            in_interface = false;
            continue;
        }

        if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
            continue;
        }

        if append_pending_member(&mut members, &mut current_name, &mut current_type, trimmed) {
            continue;
        }

        if let Some((name, type_ann)) = parse_named_type(trimmed) {
            if type_ann.trim().is_empty() {
                current_name = Some(name);
                current_type.clear();
            } else if is_type_complete(type_ann.as_str()) {
                members.push((name, normalize_type(type_ann.as_str())));
            } else {
                current_name = Some(name);
                current_type = type_ann;
            }
        }
    }

    flush_pending_member(&mut members, &mut current_name, &mut current_type);
    members
}

pub(super) fn parse_declared_global_values(
    path: &Path,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parse_declared_global_values_content(&content, base_dir))
}

pub(super) fn parse_declared_global_values_content(
    content: &str,
    source_dir: &Path,
) -> Vec<(String, String)> {
    let mut values = Vec::new();
    let mut in_global = false;
    let mut brace_depth = 0i32;
    let mut current_name: Option<String> = None;
    let mut current_type = String::default();

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_global {
            if trimmed.starts_with("declare global") {
                in_global = true;
                brace_depth = brace_delta(trimmed);
            }
            continue;
        }

        brace_depth += brace_delta(trimmed);
        if brace_depth <= 0 {
            flush_pending_global(
                &mut values,
                &mut current_name,
                &mut current_type,
                source_dir,
            );
            in_global = false;
            continue;
        }

        if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
            continue;
        }

        if append_pending_global(
            &mut values,
            &mut current_name,
            &mut current_type,
            trimmed,
            source_dir,
        ) {
            continue;
        }

        if let Some(rest) = trimmed
            .strip_prefix("const ")
            .or_else(|| trimmed.strip_prefix("let "))
            .or_else(|| trimmed.strip_prefix("var "))
        {
            if let Some((name, type_ann)) = parse_named_type(rest) {
                if type_ann.trim().is_empty() {
                    current_name = Some(name);
                    current_type.clear();
                } else if is_type_complete(type_ann.as_str()) {
                    values.push((
                        name,
                        normalize_rewritten_type(type_ann.as_str(), source_dir),
                    ));
                } else {
                    current_name = Some(name);
                    current_type = type_ann;
                }
            }
        }
    }

    flush_pending_global(
        &mut values,
        &mut current_name,
        &mut current_type,
        source_dir,
    );
    values
}

fn append_pending_member(
    members: &mut Vec<(String, String)>,
    current_name: &mut Option<String>,
    current_type: &mut String,
    trimmed: &str,
) -> bool {
    if current_name.is_none() {
        return false;
    }

    current_type.push(' ');
    current_type.push_str(trimmed.trim_end_matches(';'));

    if is_type_complete(current_type.as_str()) {
        let name = current_name.take().unwrap();
        members.push((name, normalize_type(current_type.as_str())));
        current_type.clear();
    }

    true
}

fn append_pending_global(
    values: &mut Vec<(String, String)>,
    current_name: &mut Option<String>,
    current_type: &mut String,
    trimmed: &str,
    source_dir: &Path,
) -> bool {
    if current_name.is_none() {
        return false;
    }

    current_type.push(' ');
    current_type.push_str(trimmed.trim_end_matches(';'));

    if is_type_complete(current_type.as_str()) {
        let name = current_name.take().unwrap();
        values.push((
            name,
            normalize_rewritten_type(current_type.as_str(), source_dir),
        ));
        current_type.clear();
    }

    true
}

fn flush_pending_member(
    members: &mut Vec<(String, String)>,
    current_name: &mut Option<String>,
    current_type: &mut String,
) {
    if let Some(name) = current_name.take() {
        members.push((name, normalize_type(current_type.as_str())));
        current_type.clear();
    }
}

fn flush_pending_global(
    values: &mut Vec<(String, String)>,
    current_name: &mut Option<String>,
    current_type: &mut String,
    source_dir: &Path,
) {
    if let Some(name) = current_name.take() {
        values.push((
            name,
            normalize_rewritten_type(current_type.as_str(), source_dir),
        ));
        current_type.clear();
    }
}

fn parse_named_type(line: &str) -> Option<(String, String)> {
    let (name_part, type_part) = line.split_once(':')?;
    let name = name_part.trim().trim_end_matches('?').trim();
    if name.is_empty() {
        return None;
    }

    Some((
        name.to_compact_string(),
        type_part.trim().trim_end_matches(';').to_compact_string(),
    ))
}

fn normalize_type(type_annotation: &str) -> String {
    type_annotation
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_compact_string()
}

fn normalize_rewritten_type(type_annotation: &str, source_dir: &Path) -> String {
    normalize_type(&rewrite_relative_import_types(type_annotation, source_dir))
}

fn brace_delta(line: &str) -> i32 {
    let mut delta = 0i32;
    for ch in line.chars() {
        match ch {
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

fn is_type_complete(s: &str) -> bool {
    let mut paren = 0i32;
    let mut angle = 0i32;
    let mut brace = 0i32;
    for ch in s.chars() {
        match ch {
            '(' => paren += 1,
            ')' => paren -= 1,
            '<' => angle += 1,
            '>' => angle -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            _ => {}
        }
    }
    paren <= 0 && angle <= 0 && brace <= 0
}

fn rewrite_relative_import_types(type_annotation: &str, source_dir: &Path) -> String {
    let bytes = type_annotation.as_bytes();
    let mut out = String::with_capacity(type_annotation.len());
    let mut i = 0usize;

    while i < bytes.len() {
        let import_prefix = if type_annotation[i..].starts_with("import('") {
            Some('\'')
        } else if type_annotation[i..].starts_with("import(\"") {
            Some('"')
        } else {
            None
        };

        let Some(quote) = import_prefix else {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        };

        out.push_str("import(");
        out.push(quote);
        i += 8;

        let start = i;
        while i < bytes.len() && bytes[i] != quote as u8 {
            i += 1;
        }

        let specifier = &type_annotation[start..i];
        out.push_str(&rewrite_relative_specifier(specifier, source_dir));

        if i < bytes.len() {
            out.push(quote);
            i += 1;
        }
    }

    out
}

pub(super) fn rewrite_relative_specifier(specifier: &str, source_dir: &Path) -> String {
    if !specifier.starts_with("./") && !specifier.starts_with("../") {
        return specifier.to_compact_string();
    }

    normalize_path(&source_dir.join(specifier))
}

fn normalize_path(path: &Path) -> String {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized.to_string_lossy().to_compact_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{parse_declared_global_values_content, parse_interface_members_content};

    #[test]
    fn parses_interface_members_with_multiline_types() {
        let content = r#"
declare module 'vue' {
  interface ComponentCustomProperties {
    foo: string
    bar:
      typeof import('./bar').bar
  }
}
"#;

        let members =
            parse_interface_members_content(content, "interface ComponentCustomProperties");
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].0.as_str(), "foo");
        assert_eq!(members[0].1.as_str(), "string");
        assert_eq!(members[1].0.as_str(), "bar");
        assert_eq!(members[1].1.as_str(), "typeof import('./bar').bar");
    }

    #[test]
    fn parses_declared_globals_and_rewrites_relative_imports() {
        let content = r#"
declare global {
  const currentUser:
    typeof import('../../app/composables/users').currentUser
  var $t: (Composer)['t']
}
"#;

        let values =
            parse_declared_global_values_content(content, Path::new("/workspace/.nuxt/types"));
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].0.as_str(), "currentUser");
        assert_eq!(
            values[0].1.as_str(),
            "typeof import('/workspace/app/composables/users').currentUser"
        );
        assert_eq!(values[1].0.as_str(), "$t");
        assert_eq!(values[1].1.as_str(), "(Composer)['t']");
    }
}
