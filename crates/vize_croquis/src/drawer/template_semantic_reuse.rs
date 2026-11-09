//! Optional owned hints from actual original parser owners.
//!
//! This bridge preserves the historical Drawer analyses. It is separate from
//! lexical-reference packets and never supplies occurrence/editor witnesses.
//! Petite hints may seed the text-keyed identifier cache only when occurrence
//! capture is absent. Callers keep the original root and source alive until
//! drawing; source identity is checked again at entry.

use oxc_ast::ast::{Expression, FunctionBody, ObjectPropertyKind, PropertyKey, Statement};
use oxc_span::Span;
use vize_armature::parser::{TemplateSyntax, TemplateSyntaxFacts};
use vize_carton::CompactString;
use vize_l0::Span as NativeSpan;
use vize_l2::{
    dialect::petite::{PetiteDirectiveKind, PetiteDocument},
    expr::ExprRef,
};
use vize_relief::{DirectiveNode, ExpressionNode, JsExpression, RootNode};

use super::{
    Drawer,
    helpers::{is_valid_identifier_fast, legacy_retained_identifier_names},
};

/// Demand-only reuse authenticated by original source identity and bytes.
///
/// Its private fields cannot be populated from caller-authored semantic facts.
/// It borrows no AST and must be used while its original source/root are alive.
#[derive(Debug)]
pub struct TemplateSemanticReuse {
    source_pointer: usize,
    source: CompactString,
    entries: Vec<Entry>,
    petite_entries: Vec<PetiteEntry>,
    petite_expression_index: Vec<PetiteExpressionIndex>,
    authenticated: bool,
}

#[derive(Debug)]
struct Entry {
    span: Span,
    cooked: CompactString,
    hint: Hint,
}

#[derive(Debug)]
enum Hint {
    InlineHandler,
    ForCollection {
        cooked: CompactString,
        names: Vec<CompactString>,
    },
}

#[derive(Debug)]
struct PetiteEntry {
    directive_span: NativeSpan,
    kind: PetiteKind,
    expression: Option<PetiteExpression>,
    scope_bindings: Option<Vec<(CompactString, u32)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PetiteKind {
    Scope,
    Effect,
}

#[derive(Debug)]
struct PetiteExpression {
    span: NativeSpan,
    cooked: CompactString,
    identifiers: Option<Vec<CompactString>>,
}

#[derive(Debug)]
struct PetiteExpressionIndex {
    span: NativeSpan,
    entry: usize,
}

/// Build optional semantic hints only from one original parser-owned bundle.
///
/// Incomplete/foreign syntax or no proven hints returns `None`. Unsupported
/// roles contribute no hint, preserving the historical fallback. Construction
/// and all source/name allocations occur only when explicitly demanded.
pub fn collect_template_semantic_reuse(
    root: &RootNode<'_>,
    syntax: &TemplateSyntaxFacts<'_>,
) -> Option<TemplateSemanticReuse> {
    let records = syntax.records_for(root.source)?;
    let mut entries = Vec::new();
    for record in records {
        // Require an unambiguous whole-goal observation from this exact source.
        let selected = syntax.syntax_for(root.source, record.span(), record.cooked());
        if !selected.is_some_and(|selected| core::ptr::eq(selected, record)) {
            continue;
        }
        let hint = match record.syntax() {
            // HandlerExpression is deliberately not reused here: the ordinary
            // classifier remains responsible for complete expression shapes.
            TemplateSyntax::Handler(owner) => {
                if !owner.body().is_some_and(proves_statement_only_handler) {
                    continue;
                }
                Hint::InlineHandler
            }
            TemplateSyntax::For(owner) if owner.hole().is_none() => {
                let Some(Ok(collection)) = owner.collection() else {
                    continue;
                };
                let Some(ast) = collection.expression() else {
                    continue;
                };
                let cooked = collection.source().text();
                let Some(names) =
                    legacy_retained_identifier_names(&JsExpression { ast, raw: cooked })
                else {
                    continue;
                };
                Hint::ForCollection {
                    cooked: CompactString::new(cooked),
                    names,
                }
            }
            _ => continue,
        };
        entries.push(Entry {
            span: record.span(),
            cooked: CompactString::new(record.cooked()),
            hint,
        });
    }
    if entries.is_empty() {
        return None;
    }
    entries.sort_unstable_by_key(|entry| (entry.span.start, entry.span.end));
    Some(TemplateSemanticReuse {
        source_pointer: root.source.as_ptr() as usize,
        source: CompactString::new(root.source),
        entries,
        petite_entries: Vec::new(),
        petite_expression_index: Vec::new(),
        authenticated: false,
    })
}

/// Build owned petite-vue hints from one completed Levels L2 document.
///
/// The Relief root remains the traversal owner so virtual output and recovery
/// stay byte-for-byte compatible. Only meanings authenticated against the same
/// source are admitted: `v-scope` keys and directive expression identifiers
/// come from the retained L2 AST, while a refused document or an unsupported
/// identifier walk leaves the historical Drawer fallback intact.
pub fn collect_petite_semantic_reuse(
    root: &RootNode<'_>,
    document: &PetiteDocument<'_, '_>,
) -> Option<TemplateSemanticReuse> {
    let source = document.owner().source();
    if source.as_ptr() != root.source.as_ptr() || source != root.source {
        return None;
    }

    let mut petite_entries = Vec::new();
    for directive in document.directives() {
        let kind = match directive.kind() {
            PetiteDirectiveKind::Scope => PetiteKind::Scope,
            PetiteDirectiveKind::Effect => PetiteKind::Effect,
        };
        let retained = match directive.expression() {
            Some(ExprRef::Js(js)) if js.matches_authored_source(root.source) => Some(js),
            _ => None,
        };
        let expression = directive.value().map(|value| PetiteExpression {
            span: value.authored_span(),
            cooked: CompactString::new(value.decoded()),
            identifiers: retained.and_then(|js| {
                legacy_retained_identifier_names(&JsExpression {
                    ast: js.ast,
                    raw: js.source,
                })
            }),
        });
        let scope_bindings = if kind == PetiteKind::Scope {
            retained.and_then(petite_scope_bindings)
        } else {
            None
        };
        petite_entries.push(PetiteEntry {
            directive_span: directive.span(),
            kind,
            expression,
            scope_bindings,
        });
    }
    if petite_entries.is_empty() {
        return None;
    }
    petite_entries
        .sort_unstable_by_key(|entry| (entry.directive_span.start, entry.directive_span.end));
    let mut petite_expression_index: Vec<_> = petite_entries
        .iter()
        .enumerate()
        .filter_map(|(entry, directive)| {
            directive
                .expression
                .as_ref()
                .map(|expression| PetiteExpressionIndex {
                    span: expression.span,
                    entry,
                })
        })
        .collect();
    petite_expression_index.sort_unstable_by_key(|entry| (entry.span.start, entry.span.end));
    Some(TemplateSemanticReuse {
        source_pointer: root.source.as_ptr() as usize,
        source: CompactString::new(root.source),
        entries: Vec::new(),
        petite_entries,
        petite_expression_index,
        authenticated: false,
    })
}

/// Recover top-level object keys and their legacy declaration offsets from one
/// retained `v-scope` JavaScript expression.
///
/// Non-object expressions remain unavailable so the historical extractor can
/// preserve its recovery behavior for inputs parsed as a different AST shape.
fn petite_scope_bindings(js: &vize_l2::expr::JsExpr<'_>) -> Option<Vec<(CompactString, u32)>> {
    let mut expression = js.ast;
    while let Expression::ParenthesizedExpression(parenthesized) = expression {
        expression = &parenthesized.expression;
    }
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };

    Some(
        object
            .properties
            .iter()
            .filter_map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.computed {
                    return None;
                }
                let (name, span) = match &property.key {
                    PropertyKey::StaticIdentifier(identifier) => {
                        (identifier.name.as_str(), identifier.span)
                    }
                    PropertyKey::StringLiteral(literal)
                        if is_valid_identifier_fast(literal.value.as_bytes()) =>
                    {
                        (literal.value.as_str(), literal.span)
                    }
                    _ => return None,
                };
                let decoded = js.ast_span_to_source(span)?;
                // Preserve the shipped Drawer's decoded-byte offset convention.
                // Exact authored entity projection remains available on `JsExpr`,
                // but changing declaration offsets here would change LSP behavior.
                let offset = js.span.start.checked_add(decoded.start)?;
                Some((CompactString::new(name), offset))
            })
            .collect(),
    )
}

fn proves_statement_only_handler(body: &FunctionBody<'_>) -> bool {
    if body.directives.len().saturating_add(body.statements.len()) > 1 {
        return true;
    }
    if !body.directives.is_empty() {
        return false;
    }
    // A trailing line comment can make the original Expr goal refuse even
    // though the historical newline-wrapped classifier admits an expression.
    // Functions/classes/blocks are ambiguous, never positive Inline evidence.
    matches!(body.statements.as_slice(), [statement] if !matches!(statement,
        Statement::ExpressionStatement(_)
            | Statement::FunctionDeclaration(_)
            | Statement::ClassDeclaration(_)
            | Statement::BlockStatement(_)
    ))
}

impl TemplateSemanticReuse {
    pub(super) fn authenticate(&mut self, root: &RootNode<'_>) -> bool {
        self.authenticated = self.source_pointer == root.source.as_ptr() as usize
            && self.source.len() == root.source.len()
            && self.source.as_str() == root.source;
        self.authenticated
    }

    fn entry(&self, expression: &ExpressionNode<'_>) -> Option<&Entry> {
        if !self.authenticated {
            return None;
        }
        let ExpressionNode::Simple(node) = expression else {
            return None;
        };
        let span = Span::new(node.loc.span.start, node.loc.span.end);
        let start = self
            .entries
            .partition_point(|entry| entry.span.start < span.start);
        self.entries
            .get(start..)?
            .iter()
            .take_while(|entry| entry.span.start == span.start)
            .find(|entry| entry.span == span && entry.cooked == node.content)
    }

    fn petite_entry(&self, directive: &DirectiveNode<'_>) -> Option<&PetiteEntry> {
        if !self.authenticated {
            return None;
        }
        let span = directive.loc.span;
        let start = self
            .petite_entries
            .partition_point(|entry| entry.directive_span.start < span.start);
        self.petite_entries
            .get(start..)?
            .iter()
            .take_while(|entry| entry.directive_span.start == span.start)
            .find(|entry| {
                entry.directive_span == span
                    && matches!(
                        (entry.kind, directive.name),
                        (PetiteKind::Scope, "scope") | (PetiteKind::Effect, "effect")
                    )
                    && self.petite_expression_matches(
                        entry.expression.as_ref(),
                        directive.exp.as_ref(),
                    )
            })
    }

    fn petite_expression_matches(
        &self,
        native: Option<&PetiteExpression>,
        legacy: Option<&ExpressionNode<'_>>,
    ) -> bool {
        match (native, legacy) {
            (None, None) => true,
            (Some(native), Some(legacy)) if native.span == legacy.loc().span => {
                let cooked = match legacy {
                    ExpressionNode::Simple(simple) => simple.content,
                    ExpressionNode::Compound(compound) => compound.loc.span.slice(&self.source),
                };
                native.cooked == cooked
            }
            _ => false,
        }
    }

    pub(super) fn petite_scope_bindings(
        &self,
        directive: &DirectiveNode<'_>,
    ) -> Option<&[(CompactString, u32)]> {
        let entry = self.petite_entry(directive)?;
        (entry.kind == PetiteKind::Scope)
            .then_some(entry.scope_bindings.as_deref())
            .flatten()
    }

    pub(super) fn petite_expression(
        &self,
        directive: &DirectiveNode<'_>,
    ) -> Option<(CompactString, NativeSpan)> {
        let expression = self.petite_entry(directive)?.expression.as_ref()?;
        Some((expression.cooked.clone(), expression.span))
    }

    pub(super) fn petite_expression_names(
        &self,
        expression: &ExpressionNode<'_>,
    ) -> Option<&[CompactString]> {
        if !self.authenticated {
            return None;
        }
        let span = expression.loc().span;
        let cooked = match expression {
            ExpressionNode::Simple(simple) => simple.content,
            ExpressionNode::Compound(compound) => compound.loc.span.slice(&self.source),
        };
        let start = self
            .petite_expression_index
            .partition_point(|entry| entry.span.start < span.start);
        self.petite_expression_index
            .get(start..)?
            .iter()
            .take_while(|entry| entry.span.start == span.start)
            .find_map(|entry| {
                let native = self.petite_entries.get(entry.entry)?.expression.as_ref()?;
                (native.span == span && native.cooked == cooked)
                    .then_some(native.identifiers.as_deref())
                    .flatten()
            })
    }

    pub(super) fn is_inline_handler(&self, expression: &ExpressionNode<'_>) -> bool {
        self.entry(expression)
            .is_some_and(|entry| matches!(entry.hint, Hint::InlineHandler))
    }

    pub(super) fn for_collection_names(
        &self,
        expression: &ExpressionNode<'_>,
        collection: &str,
    ) -> Option<Vec<CompactString>> {
        let Hint::ForCollection { cooked, names } = &self.entry(expression)?.hint else {
            return None;
        };
        (cooked.as_str() == collection).then(|| names.clone())
    }
}

impl Drawer {
    /// Reuse bounded, proven semantic hints from this template's original
    /// parser bundle. Ordinary drawers retain `None` and allocate no packet.
    pub fn with_template_semantic_reuse(mut self, reuse: TemplateSemanticReuse) -> Self {
        self.template_semantic_reuse = Some(Box::new(reuse));
        self
    }
}

#[cfg(test)]
mod petite_tests;
