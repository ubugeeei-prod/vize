//! Optional owned hints from actual original parser owners.
//!
//! This bridge preserves the historical Drawer analyses. It is separate from
//! lexical-reference packets and never supplies occurrence/editor witnesses or
//! populates the text-keyed identifier cache. Callers keep the original root
//! and source alive until drawing; source identity is checked again at entry.

use oxc_ast::ast::{FunctionBody, Statement};
use oxc_span::Span;
use vize_armature::parser::{TemplateSyntax, TemplateSyntaxFacts};
use vize_carton::CompactString;
use vize_relief::{ExpressionNode, JsExpression, RootNode};

use super::{Drawer, helpers::legacy_retained_identifier_names};

/// Demand-only reuse authenticated by original source identity and bytes.
///
/// Its private fields cannot be populated from caller-authored semantic facts.
/// It borrows no AST and must be used while its original source/root are alive.
#[derive(Debug)]
pub struct TemplateSemanticReuse {
    source_pointer: usize,
    source: CompactString,
    entries: Vec<Entry>,
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
        authenticated: false,
    })
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
