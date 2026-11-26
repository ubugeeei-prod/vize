//! File-local types that copied declaration members name.
//!
//! Nuxt types a lazy component as `LazyComponent<typeof import('…')>` and a
//! server component as `IslandComponent<…>`, where both are type aliases local
//! to the generated `components.d.ts`. A member copied out of that file names a
//! type that no longer exists there, and the stub file's `@ts-nocheck` turns it
//! into `any` without a word. These helpers find the local types a set of
//! members needs, directly or through each other, and qualify the members'
//! references, so the caller can declare those types once in a namespace that
//! no authored code can name.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{FxHashSet, String, ToCompactString};

/// One top-level type alias or interface of a declaration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LocalType {
    pub(super) name: String,
    /// The declaration as written, without a leading `export`.
    pub(super) declaration: String,
}

/// The top-level type aliases and interfaces of `content`, in source order.
/// Declarations inside `declare module` and `declare global` are left out:
/// they belong to another scope and stay where the program loads them.
pub(super) fn collect_local_types(content: &str) -> Vec<LocalType> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, content, SourceType::d_ts()).parse();
    if ret.panicked {
        return Vec::new();
    }
    let mut locals = Vec::new();
    for statement in &ret.program.body {
        let declaration = match statement {
            Statement::TSTypeAliasDeclaration(alias) => Some((alias.id.name.as_str(), alias.span)),
            Statement::TSInterfaceDeclaration(interface) => {
                Some((interface.id.name.as_str(), interface.span))
            }
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSTypeAliasDeclaration(alias)) => {
                    Some((alias.id.name.as_str(), alias.span))
                }
                Some(Declaration::TSInterfaceDeclaration(interface)) => {
                    Some((interface.id.name.as_str(), interface.span()))
                }
                _ => None,
            },
            _ => None,
        };
        let Some((name, span)) = declaration else {
            continue;
        };
        let Some(text) = content.get(span.start as usize..span.end as usize) else {
            continue;
        };
        // The caller declares these inside an ambient namespace, where a
        // `declare` of its own is redundant.
        let text = text.trim();
        let text = text.strip_prefix("declare ").unwrap_or(text);
        locals.push(LocalType {
            name: name.to_compact_string(),
            declaration: text.trim_end_matches(';').to_compact_string(),
        });
    }
    locals
}

/// Prefix every reference in `text` to a type in `names` with `namespace.`,
/// adding each name it qualifies to `used`.
///
/// Strings are copied verbatim, so a module specifier that spells a local
/// name stays intact, and so are a name that follows a `.` (a member of
/// something else) and a property key.
pub(super) fn qualify_local_references(
    text: &str,
    names: &FxHashSet<&str>,
    namespace: &str,
    used: &mut FxHashSet<String>,
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        if matches!(ch, '\'' | '"' | '`') {
            out.push(ch);
            let mut escaped = false;
            for (_, inner) in chars.by_ref() {
                out.push(inner);
                if escaped {
                    escaped = false;
                } else if inner == '\\' {
                    escaped = true;
                } else if inner == ch {
                    break;
                }
            }
            continue;
        }
        if !is_identifier_start(ch) {
            out.push(ch);
            continue;
        }
        let mut end = start + ch.len_utf8();
        while let Some(&(index, next)) = chars.peek() {
            if !is_identifier_char(next) {
                break;
            }
            end = index + next.len_utf8();
            chars.next();
        }
        let ident = text.get(start..end).unwrap_or_default();
        if names.contains(ident)
            && previous_non_whitespace(text, start) != Some('.')
            && !is_property_key(text.get(end..).unwrap_or_default())
        {
            out.push_str(namespace);
            out.push('.');
            used.insert(ident.to_compact_string());
        }
        out.push_str(ident);
    }
    out
}

/// The local types `used` names, together with every local type those name in
/// turn, in source order.
pub(super) fn reachable_local_types(
    locals: &[LocalType],
    mut used: FxHashSet<String>,
) -> Vec<&LocalType> {
    let names = locals
        .iter()
        .map(|local| local.name.as_str())
        .collect::<FxHashSet<_>>();
    let mut pending = used.iter().cloned().collect::<Vec<_>>();
    while let Some(name) = pending.pop() {
        let Some(local) = locals.iter().find(|local| local.name == name) else {
            continue;
        };
        let mut named = FxHashSet::default();
        qualify_local_references(local.declaration.as_str(), &names, "", &mut named);
        for name in named {
            if used.insert(name.clone()) {
                pending.push(name);
            }
        }
    }
    locals
        .iter()
        .filter(|local| used.contains(&local.name))
        .collect()
}

fn previous_non_whitespace(text: &str, index: usize) -> Option<char> {
    text.get(..index)?
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
}

/// Whether the identifier the text before `rest` ends with is a property key:
/// `Name:` or `Name?:`, as opposed to the `?` of a conditional type.
fn is_property_key(rest: &str) -> bool {
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('?').map_or(rest, str::trim_start);
    rest.starts_with(':')
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

fn is_identifier_char(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}
