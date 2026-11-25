//! Rewrite one import declaration so each specifier has the kind its uses need.
//!
//! The fix is a single replacement of the reported declaration. A declaration
//! that stays one declaration is edited token by token (`type` keywords are
//! inserted or removed and nothing else moves), so comments and layout
//! survive. A declaration that has to split is rebuilt from its specifiers:
//! the type declarations come first and the remaining value declaration keeps
//! the original position, so the module's evaluation order does not change.
//! A split that would drop a comment or an import attribute is not offered.

use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Program};
use oxc_span::GetSpan;
use vize_l0::String;

use super::FixStyle;

/// Where one specifier is written after the fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Placement {
    /// A value specifier of the remaining declaration.
    Value,
    /// A named specifier of the remaining declaration with an inline `type`.
    InlineType,
    /// A specifier of an `import type` declaration.
    TypeDeclaration,
}

/// The placement of a specifier that is only used as a type.
pub(super) fn type_placement(
    specifier: &ImportDeclarationSpecifier<'_>,
    style: FixStyle,
) -> Placement {
    match (specifier, style) {
        (ImportDeclarationSpecifier::ImportSpecifier(_), FixStyle::InlineTypeImports) => {
            Placement::InlineType
        }
        _ => Placement::TypeDeclaration,
    }
}

/// The replacement text of `declaration`, or `None` when no safe rewrite exists.
pub(super) fn rewrite(
    program: &Program<'_>,
    source: &str,
    declaration: &ImportDeclaration<'_>,
    placements: &[Placement],
) -> Option<String> {
    let specifiers = declaration.specifiers.as_deref()?;
    if specifiers.len() != placements.len() {
        return None;
    }
    let in_type_declaration = placements
        .iter()
        .filter(|placement| **placement == Placement::TypeDeclaration)
        .count();
    if in_type_declaration == 0 {
        return edit_in_place(source, declaration, placements, false);
    }
    // Import attributes cannot follow a type-only import.
    if declaration.with_clause.is_some() {
        return None;
    }
    if in_type_declaration == placements.len() && is_single_form(source, declaration) {
        return edit_in_place(source, declaration, placements, true);
    }
    let has_comment = program.comments.iter().any(|comment| {
        comment.span.start >= declaration.span.start && comment.span.end <= declaration.span.end
    });
    if has_comment {
        return None;
    }
    rebuild(source, declaration, placements)
}

/// Default-only, namespace-only, or named-only with no empty `{}` to drop.
fn is_single_form(source: &str, declaration: &ImportDeclaration<'_>) -> bool {
    let Some(specifiers) = declaration.specifiers.as_deref() else {
        return false;
    };
    let named = specifiers
        .iter()
        .filter(|specifier| matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(_)))
        .count();
    if named == specifiers.len() {
        return true;
    }
    specifiers.len() == 1 && braces(source, declaration).is_none()
}

/// Keep the declaration and only insert or remove `type` keywords.
fn edit_in_place(
    source: &str,
    declaration: &ImportDeclaration<'_>,
    placements: &[Placement],
    type_declaration: bool,
) -> Option<String> {
    let specifiers = declaration.specifiers.as_deref()?;
    let start = declaration.span.start as usize;
    let end = declaration.span.end as usize;
    let keyword_end = start.checked_add("import".len())?;
    if source.get(start..keyword_end)? != "import" {
        return None;
    }
    // (start, end, replacement), in source order.
    let mut edits: Vec<(usize, usize, &str)> = Vec::new();
    match (declaration.import_kind.is_type(), type_declaration) {
        (false, true) => edits.push((keyword_end, keyword_end, " type")),
        (true, false) => {
            let keyword = skip_trivia(source, keyword_end);
            edits.push(type_keyword_removal(source, keyword)?);
        }
        _ => {}
    }
    for (specifier, placement) in specifiers.iter().zip(placements) {
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            if !type_declaration && *placement != Placement::Value {
                return None;
            }
            continue;
        };
        let at = named.span.start as usize;
        let inline_now = named.import_kind.is_type();
        let inline_wanted = !type_declaration && *placement == Placement::InlineType;
        match (inline_now, inline_wanted) {
            (false, true) => edits.push((at, at, "type ")),
            (true, false) => edits.push(type_keyword_removal(source, at)?),
            _ => {}
        }
    }
    let mut text = String::default();
    let mut cursor = start;
    for (from, to, replacement) in edits {
        text.push_str(source.get(cursor..from)?);
        text.push_str(replacement);
        cursor = to;
    }
    text.push_str(source.get(cursor..end)?);
    Some(text)
}

/// The inline `type` modifier of the named specifier at `specifier`, with the
/// whitespace that follows it.
pub(super) fn inline_type_keyword(
    source: &str,
    specifier: oxc_span::Span,
) -> Option<oxc_span::Span> {
    let (start, end, _) = type_keyword_removal(source, specifier.start as usize)?;
    Some(oxc_span::Span::new(start as u32, end as u32))
}

/// The `type` keyword at `at` together with the whitespace that follows it.
fn type_keyword_removal(source: &str, at: usize) -> Option<(usize, usize, &'static str)> {
    let after = at.checked_add("type".len())?;
    if source.get(at..after)? != "type" {
        return None;
    }
    let rest = source.get(after..)?;
    let whitespace = rest.len() - rest.trim_start().len();
    Some((at, after + whitespace, ""))
}

/// The first position at or after `at` that is neither whitespace nor comment.
fn skip_trivia(source: &str, mut at: usize) -> usize {
    loop {
        let Some(rest) = source.get(at..) else {
            return at;
        };
        let trimmed = rest.trim_start();
        at += rest.len() - trimmed.len();
        if let Some(comment) = trimmed.strip_prefix("//") {
            at += 2 + comment.find('\n').unwrap_or(comment.len());
        } else if let Some(comment) = trimmed.strip_prefix("/*") {
            at += 2 + comment.find("*/").map_or(comment.len(), |end| end + 2);
        } else {
            return at;
        }
    }
}

/// The byte range between the named specifier braces, exclusive of them.
fn braces(source: &str, declaration: &ImportDeclaration<'_>) -> Option<(usize, usize)> {
    let specifiers = declaration.specifiers.as_deref()?;
    let start = declaration.span.start as usize;
    let clause_end = declaration.source.span.start as usize;
    let mut named = specifiers.iter().filter_map(|specifier| match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(named) => Some(named.span),
        _ => None,
    });
    let (open, close) = match named.next() {
        Some(first) => {
            let last = named.next_back().unwrap_or(first);
            let open = source.get(start..first.start as usize)?.rfind('{')? + start;
            let close = source.get(last.end as usize..clause_end)?.find('}')? + last.end as usize;
            (open, close)
        }
        None => {
            // `import A, {} from 'm'`: only a default specifier can precede.
            let after = specifiers
                .first()
                .map_or(start, |specifier| specifier_end(specifier));
            let clause = source.get(after..clause_end)?;
            let open = clause.find('{')? + after;
            let close = clause.rfind('}')? + after;
            (open, close)
        }
    };
    (open < close).then_some((open + 1, close))
}

fn specifier_end(specifier: &ImportDeclarationSpecifier<'_>) -> usize {
    (match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(named) => named.span.end,
        ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => default.span.end,
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => namespace.span.end,
    }) as usize
}

/// How the named specifier list of the original declaration was laid out.
struct ListLayout<'s> {
    /// Indentation of each specifier when the list spans lines.
    item_indent: Option<&'s str>,
    trailing_comma: bool,
    padded: bool,
}

impl<'s> ListLayout<'s> {
    fn of(source: &'s str, declaration: &ImportDeclaration<'_>) -> Self {
        let inner = braces(source, declaration)
            .and_then(|(open, close)| source.get(open..close))
            .unwrap_or_default();
        let item_indent = inner.contains('\n').then(|| {
            let after_newline = inner.split('\n').nth(1).unwrap_or_default();
            let indent = after_newline.len() - after_newline.trim_start().len();
            after_newline.get(..indent).unwrap_or_default()
        });
        Self {
            item_indent,
            trailing_comma: inner.trim_end().ends_with(','),
            padded: inner.is_empty() || inner.starts_with(char::is_whitespace),
        }
    }

    fn write(&self, items: &[String], indent: &str, newline: &str, text: &mut String) {
        text.push('{');
        if let Some(item_indent) = self.item_indent {
            for item in items {
                text.push_str(newline);
                text.push_str(item_indent);
                text.push_str(item);
                text.push(',');
            }
            if !self.trailing_comma {
                text.pop();
            }
            text.push_str(newline);
            text.push_str(indent);
        } else {
            let pad = if self.padded { " " } else { "" };
            text.push_str(pad);
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    text.push_str(", ");
                }
                text.push_str(item);
            }
            if self.trailing_comma {
                text.push(',');
            }
            text.push_str(pad);
        }
        text.push('}');
    }
}

/// Split the declaration into type declarations and the remaining one.
fn rebuild(
    source: &str,
    declaration: &ImportDeclaration<'_>,
    placements: &[Placement],
) -> Option<String> {
    let specifiers = declaration.specifiers.as_deref()?;
    let start = declaration.span.start as usize;
    let end = declaration.span.end as usize;
    let line_start = source.get(..start)?.rfind('\n').map_or(0, |at| at + 1);
    let line_prefix = source.get(line_start..start)?;
    let indent = if line_prefix.trim().is_empty() {
        line_prefix
    } else {
        ""
    };
    // The line ending that follows the declaration, else the block's first.
    let newline = if source
        .get(end..)?
        .find('\n')
        .map(|at| at + end)
        .or_else(|| source.find('\n'))
        .and_then(|at| at.checked_sub(1))
        .is_some_and(|before| source.as_bytes().get(before) == Some(&b'\r'))
    {
        "\r\n"
    } else {
        "\n"
    };
    let from =
        source.get(declaration.source.span.start as usize..declaration.source.span.end as usize)?;
    let tail = source.get(declaration.source.span.end as usize..end)?;
    let semicolon = if tail.trim_end().ends_with(';') {
        ";"
    } else {
        ""
    };
    let layout = ListLayout::of(source, declaration);

    let mut type_default = None;
    let mut type_namespace = None;
    let mut type_named: Vec<String> = Vec::new();
    let mut default = None;
    let mut namespace = None;
    let mut named: Vec<String> = Vec::new();
    for (specifier, placement) in specifiers.iter().zip(placements) {
        match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                let text =
                    source.get(specifier.span.start as usize..specifier.span.end as usize)?;
                match placement {
                    Placement::TypeDeclaration => type_default = Some(text),
                    Placement::Value => default = Some(text),
                    Placement::InlineType => return None,
                }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                let text =
                    source.get(specifier.span.start as usize..specifier.span.end as usize)?;
                match placement {
                    Placement::TypeDeclaration => type_namespace = Some(text),
                    Placement::Value => namespace = Some(text),
                    Placement::InlineType => return None,
                }
            }
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                // The binding without its inline `type` modifier.
                let bare = source.get(
                    specifier.imported.span().start as usize..specifier.local.span.end as usize,
                )?;
                match placement {
                    Placement::TypeDeclaration => type_named.push(String::from(bare)),
                    Placement::Value => named.push(String::from(bare)),
                    Placement::InlineType => named.push(vize_l0::cstr!("type {bare}")),
                }
            }
        }
    }

    let mut declarations: Vec<String> = Vec::new();
    for clause in [type_default, type_namespace].into_iter().flatten() {
        declarations.push(vize_l0::cstr!(
            "import type {clause} from {from}{semicolon}"
        ));
    }
    if !type_named.is_empty() {
        let mut text = String::from("import type ");
        layout.write(&type_named, indent, newline, &mut text);
        text.push_str(&vize_l0::cstr!(" from {from}{semicolon}"));
        declarations.push(text);
    }
    if default.is_some() || namespace.is_some() || !named.is_empty() {
        let mut text = String::from("import ");
        if let Some(default) = default {
            text.push_str(default);
            if namespace.is_some() || !named.is_empty() {
                text.push_str(", ");
            }
        }
        if let Some(namespace) = namespace {
            text.push_str(namespace);
        } else if !named.is_empty() {
            layout.write(&named, indent, newline, &mut text);
        }
        text.push_str(&vize_l0::cstr!(" from {from}"));
        // Whatever followed the module specifier, including its semicolon.
        text.push_str(tail);
        declarations.push(text);
    }
    let mut text = String::default();
    for (index, declaration) in declarations.iter().enumerate() {
        if index > 0 {
            text.push_str(newline);
            text.push_str(indent);
        }
        text.push_str(declaration);
    }
    (!text.is_empty()).then_some(text)
}
