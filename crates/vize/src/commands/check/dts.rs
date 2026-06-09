//! Helpers for lightweight `.d.ts` parsing used by `vize check`.

#![allow(clippy::disallowed_macros)]

use std::{fs, path::Path};

use vize_carton::{String, ToCompactString, profile, profiler::global_profiler};

pub(super) fn parse_interface_members(
    path: &Path,
    interface_name: &str,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = match profile!("cli.check.dts.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            content
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(error);
        }
    };
    Ok(parse_interface_members_content(&content, interface_name))
}

pub(super) fn parse_interface_members_with_rewritten_imports(
    path: &Path,
    interface_name: &str,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = match profile!("cli.check.dts.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            content
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(error);
        }
    };
    let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parse_interface_members_content(&content, interface_name)
        .into_iter()
        .map(|(name, type_annotation)| {
            (
                name,
                normalize_rewritten_type(type_annotation.as_str(), source_dir),
            )
        })
        .collect())
}

pub(super) fn parse_global_component_members_with_rewritten_imports(
    path: &Path,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = match profile!("cli.check.dts.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            content
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(error);
        }
    };
    let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parse_global_component_members_content(&content)
        .into_iter()
        .map(|(name, type_annotation)| {
            (
                name,
                normalize_rewritten_type(type_annotation.as_str(), source_dir),
            )
        })
        .collect())
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
                let delta = brace_delta(trimmed);
                if delta <= 0 && trimmed.contains('{') {
                    continue;
                }
                in_interface = true;
                brace_depth = delta;
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

fn parse_global_component_members_content(content: &str) -> Vec<(String, String)> {
    let mut members = parse_interface_members_content(content, "interface GlobalComponents");

    for extended in extended_interface_names(content, "GlobalComponents") {
        let interface_name = format!("interface {extended}");
        for member in parse_interface_members_content(content, &interface_name) {
            if !members
                .iter()
                .any(|(name, _)| name.as_str() == member.0.as_str())
            {
                members.push(member);
            }
        }
    }

    members
}

fn extended_interface_names(content: &str, interface_name: &str) -> Vec<String> {
    let needle = format!("interface {interface_name}");
    let mut names = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let Some(index) = trimmed.find(&needle) else {
            continue;
        };
        if !interface_needle_has_boundary(trimmed, index, needle.len()) {
            continue;
        }

        let after_name = &trimmed[index + needle.len()..];
        let Some((_, after_extends)) = after_name.split_once("extends") else {
            continue;
        };
        let extends_clause = after_extends.split('{').next().unwrap_or(after_extends);
        for raw_name in extends_clause.split(',') {
            let name = raw_name
                .trim()
                .split(|ch: char| ch.is_whitespace() || ch == '<')
                .next()
                .unwrap_or_default()
                .trim();
            if is_interface_reference_name(name) && !names.iter().any(|existing| existing == name) {
                names.push(name.to_compact_string());
            }
        }
    }

    names
}

fn interface_needle_has_boundary(line: &str, index: usize, needle_len: usize) -> bool {
    let before = line[..index]
        .chars()
        .next_back()
        .is_none_or(|ch| !is_identifier_char(ch));
    let after = line[index + needle_len..]
        .chars()
        .next()
        .is_none_or(|ch| !is_identifier_char(ch));
    before && after
}

fn is_interface_reference_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic()) && chars.all(is_identifier_char)
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

pub(super) fn parse_declared_global_values(
    path: &Path,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = match profile!("cli.check.dts.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            content
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(error);
        }
    };
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
            && let Some((name, type_ann)) = parse_named_type(rest)
        {
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

    if is_type_complete(current_type.as_str())
        && let Some(name) = current_name.take()
    {
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

    if is_type_complete(current_type.as_str())
        && let Some(name) = current_name.take()
    {
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
    let mut name = name_part.trim().trim_end_matches('?').trim();
    if let Some(rest) = name.strip_prefix("readonly ") {
        name = rest.trim().trim_end_matches('?').trim();
    }
    let name = name.trim_matches('"').trim_matches('\'').trim();
    if name.is_empty() {
        return None;
    }
    if name.starts_with('[') {
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

    use super::{
        parse_declared_global_values_content, parse_global_component_members_content,
        parse_interface_members_content,
    };

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
    fn parses_readonly_and_quoted_members_without_index_signatures() {
        let content = r#"
declare module 'vue' {
  interface ComponentCustomProperties {
    readonly $config?: typeof import('./config').config
    "quoted-key": string
    [key: string]: unknown
  }
}
"#;

        let members =
            parse_interface_members_content(content, "interface ComponentCustomProperties");

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].0.as_str(), "$config");
        assert_eq!(members[0].1.as_str(), "typeof import('./config').config");
        assert_eq!(members[1].0.as_str(), "quoted-key");
        assert_eq!(members[1].1.as_str(), "string");
    }

    #[test]
    fn parses_global_components_from_extended_interface() {
        let content = r#"
interface _GlobalComponents {
  GlobalButton: GlobalComponentConstructor<GlobalButtonProps>
  GlobalInput:
    GlobalComponentConstructor<GlobalInputProps>
}

declare module "vue" {
  interface GlobalComponents extends _GlobalComponents {}
}
"#;

        let members = parse_global_component_members_content(content);

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].0.as_str(), "GlobalButton");
        assert_eq!(
            members[0].1.as_str(),
            "GlobalComponentConstructor<GlobalButtonProps>"
        );
        assert_eq!(members[1].0.as_str(), "GlobalInput");
        assert_eq!(
            members[1].1.as_str(),
            "GlobalComponentConstructor<GlobalInputProps>"
        );
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
