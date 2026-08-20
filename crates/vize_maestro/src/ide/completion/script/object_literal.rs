//! Exact member completion for local static object literals.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, AssignmentExpression, BindingPattern, CallExpression, Declaration, Expression,
    ObjectExpression, ObjectPropertyKind, PropertyKey, SimpleAssignmentTarget, Statement,
    UnaryExpression, UpdateExpression, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_call_expression, walk_unary_expression,
        walk_update_expression, walk_variable_declarator,
    },
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use oxc_syntax::operator::UnaryOperator;
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

    let members = static_object_members(&script, receiver, local_offset as u32)?;
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
            .is_none_or(|(_, binding, kind)| {
                binding.declaration_offset <= offset
                    && matches!(kind, ScopeKind::ScriptSetup | ScopeKind::NonScriptSetup)
            })
}

fn static_object_members(
    script: &str,
    receiver: &str,
    cursor_offset: u32,
) -> Option<Vec<StaticMember>> {
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
            found = Some((collect_static_members(object)?, declarator.span.end));
        }
    }
    let (members, declaration_end) = found?;
    // Croquis currently records script bindings in the summary without
    // duplicating every top-level binding in the scope chain. The declarator
    // span is therefore the authoritative TDZ guard for that top-level case.
    if declaration_end > cursor_offset {
        return None;
    }
    let mut inexactness = PriorReceiverInexactness {
        receiver,
        declaration_end,
        cursor_offset,
        found: false,
    };
    inexactness.visit_program(&parsed.program);
    (!inexactness.found).then_some(members)
}

struct PriorReceiverInexactness<'s> {
    receiver: &'s str,
    declaration_end: u32,
    cursor_offset: u32,
    found: bool,
}

impl PriorReceiverInexactness<'_> {
    fn is_prior(&self, span: Span) -> bool {
        span.start >= self.declaration_end && span.end <= self.cursor_offset
    }

    fn targets_receiver(&self, target: &SimpleAssignmentTarget<'_>) -> bool {
        target_root_reference(target).is_some_and(|name| name == self.receiver)
    }

    fn is_receiver_value(&self, expression: &Expression<'_>) -> bool {
        matches!(
            expression.get_inner_expression(),
            Expression::Identifier(identifier) if identifier.name == self.receiver
        )
    }

    fn argument_escapes_receiver(&self, argument: &Argument<'_>) -> bool {
        match argument {
            Argument::SpreadElement(spread) => self.is_receiver_value(&spread.argument),
            argument => argument
                .as_expression()
                .is_some_and(|expression| self.is_receiver_value(expression)),
        }
    }
}

impl<'a> Visit<'a> for PriorReceiverInexactness<'_> {
    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression<'a>) {
        if self.is_prior(expression.span)
            && (expression
                .left
                .as_simple_assignment_target()
                .is_some_and(|target| self.targets_receiver(target))
                || self.is_receiver_value(&expression.right))
        {
            self.found = true;
            return;
        }
        walk_assignment_expression(self, expression);
    }

    fn visit_update_expression(&mut self, expression: &UpdateExpression<'a>) {
        if self.is_prior(expression.span) && self.targets_receiver(&expression.argument) {
            self.found = true;
            return;
        }
        walk_update_expression(self, expression);
    }

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Delete
            && self.is_prior(expression.span)
            && root_reference(&expression.argument).is_some_and(|name| name == self.receiver)
        {
            self.found = true;
            return;
        }
        walk_unary_expression(self, expression);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if self.is_prior(declarator.span)
            && declarator
                .init
                .as_ref()
                .is_some_and(|init| self.is_receiver_value(init))
        {
            self.found = true;
            return;
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        if self.is_prior(expression.span)
            && (root_reference(&expression.callee).is_some_and(|name| name == self.receiver)
                || expression
                    .arguments
                    .iter()
                    .any(|argument| self.argument_escapes_receiver(argument)))
        {
            self.found = true;
            return;
        }
        walk_call_expression(self, expression);
    }
}

fn target_root_reference<'a>(target: &'a SimpleAssignmentTarget<'a>) -> Option<&'a str> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some(identifier.name.as_str())
        }
        other if other.is_member_expression() => {
            root_reference(other.to_member_expression().object())
        }
        other => other.get_expression().and_then(root_reference),
    }
}

fn root_reference<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        expression if expression.is_member_expression() => {
            root_reference(expression.to_member_expression().object())
        }
        _ => None,
    }
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
