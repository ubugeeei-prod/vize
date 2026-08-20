//! Exact member completion for local static object literals.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    Statement, VariableDeclarationKind,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};
use vize_carton::{FxHashSet, String};
use vize_croquis::{Drawer, DrawerOptions, ScopeKind};

use super::context::{member_access_receiver, script_content_and_offset_for_context};
use crate::ide::IdeContext;

struct StaticMember {
    name: String,
    kind: CompletionItemKind,
}

pub(super) fn complete(ctx: &IdeContext, is_setup: bool) -> Option<Vec<CompletionItem>> {
    let receiver = member_access_receiver(&ctx.content, ctx.offset)?;
    if !is_plain_identifier(receiver) {
        return None;
    }

    let (script, script_start) = script_content_and_offset_for_context(ctx, is_setup)?;
    let local_offset = ctx.offset.checked_sub(script_start)?;
    if local_offset > script.len()
        || !receiver_resolves_to_top_level(&script, local_offset as u32, receiver, is_setup)
    {
        return None;
    }

    let members = static_object_members(&script, receiver)?;
    if members.is_empty() {
        return None;
    }
    Some(
        members
            .into_iter()
            .map(|member| completion_item(receiver, member))
            .collect(),
    )
}

fn receiver_resolves_to_top_level(
    script: &str,
    offset: u32,
    receiver: &str,
    is_setup: bool,
) -> bool {
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    if is_setup {
        drawer.analyze_script_setup(script);
    } else {
        drawer.analyze_script_plain(script);
    }
    let summary = drawer.finish();
    summary.bindings.contains(receiver)
        && summary
            .scopes
            .bindings_visible_at(offset)
            .into_iter()
            .find(|(name, _, _)| *name == receiver)
            .is_none_or(|(_, _, kind)| {
                matches!(kind, ScopeKind::ScriptSetup | ScopeKind::NonScriptSetup)
            })
}

fn static_object_members(script: &str, receiver: &str) -> Option<Vec<StaticMember>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }

    let mut found = None;
    for statement in &parsed.program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration,
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::VariableDeclaration(declaration)) => declaration,
                _ => continue,
            },
            _ => continue,
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            if identifier.name != receiver {
                continue;
            }
            if declaration.kind != VariableDeclarationKind::Const {
                // A mutable binding may be reassigned before the cursor. Its
                // initializer alone is not an authoritative completion set.
                return None;
            }
            if declarator.type_annotation.is_some() {
                // An annotation can contribute optional or otherwise absent
                // members that are not present in the initializer.
                return None;
            }
            if found.is_some() {
                // Invalid duplicate declarations are not a basis for an exact
                // fast path; let the language service resolve them.
                return None;
            }
            let object = declarator.init.as_ref().and_then(unwrap_object)?;
            found = Some(collect_static_members(object)?);
        }
    }
    found
}

fn unwrap_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::TSSatisfiesExpression(expression) => unwrap_object(&expression.expression),
        Expression::TSNonNullExpression(expression) => unwrap_object(&expression.expression),
        Expression::ParenthesizedExpression(expression) => unwrap_object(&expression.expression),
        _ => None,
    }
}

fn collect_static_members(object: &ObjectExpression<'_>) -> Option<Vec<StaticMember>> {
    let mut members = Vec::new();
    let mut seen = FxHashSet::default();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // A spread may contribute unknown keys, so the local list would
            // not be authoritative enough to bypass Corsa.
            return None;
        };
        let name = static_property_name(&property.key)?;
        if seen.insert(name) {
            members.push(StaticMember {
                name: name.into(),
                kind: if property.method {
                    CompletionItemKind::METHOD
                } else {
                    CompletionItemKind::PROPERTY
                },
            });
        }
    }
    Some(members)
}

fn static_property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn is_plain_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}

#[allow(clippy::disallowed_macros)]
fn completion_item(receiver: &str, member: StaticMember) -> CompletionItem {
    CompletionItem {
        label: member.name.to_string(),
        kind: Some(member.kind),
        detail: Some("local object member".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("Statically declared member of the local object `{receiver}`."),
        })),
        sort_text: Some(format!("0{}", member.name)),
        ..Default::default()
    }
}
