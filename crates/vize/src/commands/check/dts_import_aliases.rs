use std::path::Path;

use vize_carton::{FxHashMap, String, ToCompactString, cstr};

use super::dts_rewrite::rewrite_relative_specifier;

#[derive(Default)]
pub(super) struct ImportTypeAliases {
    aliases: FxHashMap<String, ImportTypeAlias>,
}

enum ImportTypeAlias {
    Default { module: String },
    Named { module: String, imported: String },
    Namespace { module: String },
}

pub(super) fn collect_import_type_aliases(content: &str, source_dir: &Path) -> ImportTypeAliases {
    let mut aliases = ImportTypeAliases::default();
    let mut current = String::default();

    for line in content.lines() {
        let trimmed = line.trim();
        if current.is_empty() {
            if !trimmed.starts_with("import ") {
                continue;
            }
            current.push_str(trimmed);
        } else {
            current.push(' ');
            current.push_str(trimmed);
        }

        if import_statement_complete(current.as_str()) {
            collect_import_statement(current.as_str(), source_dir, &mut aliases);
            current.clear();
        }
    }

    if !current.is_empty() {
        collect_import_statement(current.as_str(), source_dir, &mut aliases);
    }

    aliases
}

pub(super) fn rewrite_import_type_aliases(
    type_annotation: &str,
    aliases: &ImportTypeAliases,
) -> String {
    if aliases.aliases.is_empty() {
        return type_annotation.to_compact_string();
    }

    let mut out = String::with_capacity(type_annotation.len());
    let mut i = 0usize;
    while i < type_annotation.len() {
        let ch = type_annotation[i..].chars().next().unwrap();
        if ch == '\'' || ch == '"' || ch == '`' {
            i = copy_quoted(type_annotation, i, &mut out);
            continue;
        }
        if !is_identifier_start(ch) {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let start = i;
        i += ch.len_utf8();
        while i < type_annotation.len() {
            let ch = type_annotation[i..].chars().next().unwrap();
            if !is_identifier_char(ch) {
                break;
            }
            i += ch.len_utf8();
        }

        let ident = &type_annotation[start..i];
        let next = type_annotation[i..].chars().next();
        if let Some(alias) = aliases.aliases.get(ident) {
            match alias {
                ImportTypeAlias::Namespace { module } if next == Some('.') => {
                    out.push_str(&render_namespace_reference(module.as_str()));
                    continue;
                }
                ImportTypeAlias::Named { module, imported }
                    if previous_non_whitespace(type_annotation, start) != Some('.') =>
                {
                    out.push_str(&render_named_reference(module.as_str(), imported.as_str()));
                    continue;
                }
                ImportTypeAlias::Default { module }
                    if previous_non_whitespace(type_annotation, start) != Some('.') =>
                {
                    out.push_str(&render_named_reference(module.as_str(), "default"));
                    continue;
                }
                _ => {}
            }
        }
        out.push_str(ident);
    }

    out
}

fn collect_import_statement(statement: &str, source_dir: &Path, aliases: &mut ImportTypeAliases) {
    let statement = statement.trim().trim_end_matches(';').trim();
    let Some(rest) = statement.strip_prefix("import ") else {
        return;
    };
    let Some((imports, from_part)) = rest.rsplit_once(" from ") else {
        return;
    };
    let Some(specifier) = parse_module_specifier(from_part) else {
        return;
    };
    let module = rewrite_relative_specifier(specifier, source_dir);
    let imports = imports.trim();

    if let Some(type_imports) = imports.strip_prefix("type ") {
        collect_type_only_imports(type_imports.trim(), module.as_str(), aliases);
    } else {
        collect_inline_type_imports(imports, module.as_str(), aliases);
    }
}

fn collect_type_only_imports(imports: &str, module: &str, aliases: &mut ImportTypeAliases) {
    if imports.starts_with('{') {
        collect_named_imports(imports, module, aliases, false);
        return;
    }
    if let Some(namespace) = imports.strip_prefix("* as ") {
        push_namespace_alias(namespace.trim(), module, aliases);
        return;
    }
    if let Some((default_import, rest)) = imports.split_once(',') {
        push_default_alias(default_import.trim(), module, aliases);
        collect_type_only_imports(rest.trim(), module, aliases);
        return;
    }
    push_default_alias(imports.trim(), module, aliases);
}

fn collect_inline_type_imports(imports: &str, module: &str, aliases: &mut ImportTypeAliases) {
    if imports.starts_with('{') {
        collect_named_imports(imports, module, aliases, true);
    }
}

fn collect_named_imports(
    imports: &str,
    module: &str,
    aliases: &mut ImportTypeAliases,
    require_type_prefix: bool,
) {
    let Some(inner) = imports
        .trim()
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
    else {
        return;
    };

    for raw in inner.split(',') {
        let mut item = raw.trim();
        if item.is_empty() {
            continue;
        }
        if require_type_prefix {
            let Some(rest) = item.strip_prefix("type ") else {
                continue;
            };
            item = rest.trim();
        }
        let (imported, local) = parse_import_binding(item);
        if is_identifier(imported) && is_identifier(local) {
            aliases.aliases.insert(
                local.to_compact_string(),
                ImportTypeAlias::Named {
                    module: module.into(),
                    imported: imported.into(),
                },
            );
        }
    }
}

fn parse_import_binding(item: &str) -> (&str, &str) {
    let mut parts = item.split_whitespace();
    let imported = parts.next().unwrap_or_default();
    match (parts.next(), parts.next()) {
        (Some("as"), Some(local)) => (imported, local),
        _ => (imported, imported),
    }
}

fn push_default_alias(local: &str, module: &str, aliases: &mut ImportTypeAliases) {
    if is_identifier(local) {
        aliases.aliases.insert(
            local.to_compact_string(),
            ImportTypeAlias::Default {
                module: module.into(),
            },
        );
    }
}

fn push_namespace_alias(local: &str, module: &str, aliases: &mut ImportTypeAliases) {
    if is_identifier(local) {
        aliases.aliases.insert(
            local.to_compact_string(),
            ImportTypeAlias::Namespace {
                module: module.into(),
            },
        );
    }
}

fn import_statement_complete(statement: &str) -> bool {
    statement.ends_with(';')
        || statement
            .rsplit_once(" from ")
            .and_then(|(_, from_part)| parse_module_specifier(from_part))
            .is_some()
}

fn parse_module_specifier(from_part: &str) -> Option<&str> {
    let from_part = from_part.trim();
    let quote = from_part
        .chars()
        .next()
        .filter(|ch| *ch == '\'' || *ch == '"')?;
    let rest = &from_part[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn copy_quoted(input: &str, start: usize, out: &mut String) -> usize {
    let quote = input[start..].chars().next().unwrap();
    let mut i = start;
    while i < input.len() {
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
        if ch == '\\' && i < input.len() {
            let escaped = input[i..].chars().next().unwrap();
            out.push(escaped);
            i += escaped.len_utf8();
            continue;
        }
        if ch == quote && i > start + quote.len_utf8() {
            break;
        }
    }
    i
}

fn render_named_reference(module: &str, imported: &str) -> String {
    cstr!("import('{module}').{imported}")
}

fn render_namespace_reference(module: &str) -> String {
    cstr!("import('{module}')")
}

fn previous_non_whitespace(input: &str, index: usize) -> Option<char> {
    input[..index].chars().rev().find(|ch| !ch.is_whitespace())
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_char)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

fn is_identifier_char(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{collect_import_type_aliases, rewrite_import_type_aliases};

    #[test]
    fn rewrites_import_type_aliases() {
        let content = r#"
import type { User, Wrapped as AliasedWrapped } from '../../app/types/user'
import type DefaultThing from '../../app/types/default'
import type * as Shared from '../../app/types/shared'
"#;
        let aliases = collect_import_type_aliases(content, Path::new("/workspace/.nuxt/types"));
        let rewritten = rewrite_import_type_aliases(
            "() => User | AliasedWrapped | DefaultThing | Shared.Thing",
            &aliases,
        );

        assert_eq!(
            rewritten.as_str(),
            "() => import('/workspace/app/types/user').User | import('/workspace/app/types/user').Wrapped | import('/workspace/app/types/default').default | import('/workspace/app/types/shared').Thing"
        );
    }

    #[test]
    fn ignores_value_imports_and_quoted_literals() {
        let content = r#"import { type User, value } from './types'"#;
        let aliases = collect_import_type_aliases(content, Path::new("/workspace/.nuxt/types"));
        let rewritten = rewrite_import_type_aliases("Record<'User', User> | value", &aliases);

        assert_eq!(
            rewritten.as_str(),
            "Record<'User', import('/workspace/.nuxt/types/types').User> | value"
        );
    }
}
