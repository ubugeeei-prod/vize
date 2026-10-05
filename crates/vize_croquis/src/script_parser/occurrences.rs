//! Authored facts emitted inside the existing script statement/expression walk.

use crate::binding_occurrences::{BindingOccurrences, OccurrenceBlock};
use crate::scope::{ScopeChain, ScopeId, ScopeKind};
use oxc_span::Span;
use vize_carton::{CompactString, FxHashMap, FxHashSet, SmallVec};

#[derive(Debug)]
pub(crate) struct ScriptOccurrenceCapture {
    pub(crate) references: Vec<ScriptReference>,
    declarations: FxHashMap<(ScopeId, CompactString), Span>,
    pub(crate) parameters: SmallVec<[(CompactString, Span); 4]>,
    valid: bool,
    pub(crate) lens_spans: FxHashSet<(u32, u32)>,
}

impl Default for ScriptOccurrenceCapture {
    fn default() -> Self {
        Self {
            references: Vec::new(),
            declarations: FxHashMap::default(),
            parameters: SmallVec::new(),
            valid: true,
            lens_spans: FxHashSet::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ScriptReference {
    scope: ScopeId,
    name: CompactString,
    span: Span,
}

impl ScriptOccurrenceCapture {
    pub(crate) fn reference(&mut self, scope: ScopeId, name: &str, span: Span) {
        self.references.push(ScriptReference {
            scope,
            name: CompactString::new(name),
            span,
        });
    }

    pub(crate) fn refuse(&mut self) {
        self.valid = false;
    }

    pub(crate) fn declaration(&mut self, scope: ScopeId, name: &str, span: Span) {
        self.declarations
            .insert((scope, CompactString::new(name)), span);
    }

    pub(crate) fn finish(
        self,
        scopes: &ScopeChain,
        globals: &FxHashMap<CompactString, (u32, u32)>,
        source: &str,
        packet: &mut BindingOccurrences,
    ) -> Option<()> {
        if !self.valid {
            return None;
        }
        let global_scope = scopes
            .find_scope_by_kind(ScopeKind::ScriptSetup)
            .or_else(|| scopes.find_scope_by_kind(ScopeKind::NonScriptSetup))?;
        let mut declarations = self.declarations;
        for (name, &(start, end)) in globals {
            declarations.insert((global_scope, name.clone()), Span::new(start, end));
        }
        for ((scope, name), span) in &declarations {
            if source.get(span.start as usize..span.end as usize) != Some(name.as_str()) {
                return None;
            }
            let binding = packet.note_binding(*scope, name, span.start, span.end);
            if self.lens_spans.contains(&(span.start, span.end)) {
                packet.note_lens_declaration(binding);
            }
        }
        for reference in self.references {
            if source.get(reference.span.start as usize..reference.span.end as usize)
                != Some(reference.name.as_str())
            {
                return None;
            }
            let mut queue = SmallVec::<[ScopeId; 8]>::from_slice(&[reference.scope]);
            let mut visited = SmallVec::<[ScopeId; 8]>::new();
            let mut owned = false;
            while let Some(scope_id) = queue.pop() {
                if visited.contains(&scope_id) {
                    continue;
                }
                visited.push(scope_id);
                let scope = scopes.get_scope(scope_id)?;
                let key = (scope_id, reference.name.clone());
                if let Some(span) = declarations.get(&key) {
                    let binding =
                        packet.note_binding(scope_id, &reference.name, span.start, span.end);
                    packet.note_reference(
                        binding,
                        reference.span.start,
                        reference.span.end,
                        OccurrenceBlock::Script,
                    );
                    owned = true;
                    break;
                }
                // A lexical owner without an exact authored declaration is not
                // a same-spelled setup declaration. Never escape to that global.
                if scope.get_binding(&reference.name).is_some() {
                    if matches!(
                        scope.kind,
                        ScopeKind::Closure
                            | ScopeKind::Block
                            | ScopeKind::Callback
                            | ScopeKind::EventHandler
                    ) {
                        return None;
                    }
                    // Ambient owners may be shadowed by a genuine normal-script
                    // module declaration after the descriptor's split join.
                    break;
                }
                queue.extend(scope.parents.iter().copied());
            }
            if !owned && scopes.get_scope(global_scope)?.kind == ScopeKind::ScriptSetup {
                packet.defer_setup_read(reference.name, reference.span.start, reference.span.end);
            }
        }
        Some(())
    }
}

impl super::ScriptParseResult {
    pub(crate) fn refuse_occurrences(&mut self) {
        if let Some(capture) = self.occurrence_capture.as_mut() {
            capture.refuse();
        }
    }

    pub(crate) fn refuse_direct_eval(&mut self, callee: &oxc_ast::ast::Expression<'_>) {
        if self.occurrence_capture.is_some()
            && matches!(callee, oxc_ast::ast::Expression::Identifier(id) if id.name == "eval")
        {
            self.refuse_occurrences();
        }
    }

    pub(crate) fn note_identifier_occurrence(&mut self, name: &str, span: Span) {
        if let Some(capture) = self.occurrence_capture.as_mut() {
            capture.reference(self.scopes.current_id(), name, span);
        }
    }

    pub(crate) fn note_lens_pattern(&mut self, pattern: &oxc_ast::ast::BindingPattern<'_>) {
        if let Some(capture) = self.occurrence_capture.as_mut() {
            let has_default = super::walk::collect_param_sites(pattern, &mut |_, span| {
                capture.lens_spans.insert((span.start, span.end));
            });
            if has_default {
                capture.refuse();
            }
        }
    }

    pub(crate) fn install_parameter_occurrences(&mut self) {
        if let Some(capture) = self.occurrence_capture.as_mut() {
            let scope = self.scopes.current_id();
            for (name, span) in std::mem::take(&mut capture.parameters) {
                capture.declaration(scope, &name, span);
            }
        }
    }
}

/// Missing scope-only forms contribute facts inside that same AST walk, without
/// invoking the unrelated diagnostics/reactivity callbacks for new forms.
pub(crate) fn note_expression_only(
    result: &mut super::ScriptParseResult,
    expression: &oxc_ast::ast::Expression<'_>,
) {
    use oxc_ast::ast::Expression;
    if result.occurrence_capture.is_none() {
        return;
    }
    match expression {
        Expression::Identifier(id) => result.note_identifier_occurrence(id.name.as_str(), id.span),
        Expression::StaticMemberExpression(member) => note_expression_only(result, &member.object),
        Expression::ComputedMemberExpression(member) => {
            note_expression_only(result, &member.object);
            note_expression_only(result, &member.expression);
        }
        Expression::TemplateLiteral(template) => {
            for expression in &template.expressions {
                note_expression_only(result, expression);
            }
        }
        Expression::BinaryExpression(binary) => {
            note_expression_only(result, &binary.left);
            note_expression_only(result, &binary.right);
        }
        Expression::LogicalExpression(logical) => {
            note_expression_only(result, &logical.left);
            note_expression_only(result, &logical.right);
        }
        Expression::ConditionalExpression(conditional) => {
            note_expression_only(result, &conditional.test);
            note_expression_only(result, &conditional.consequent);
            note_expression_only(result, &conditional.alternate);
        }
        Expression::UnaryExpression(unary) => note_expression_only(result, &unary.argument),
        Expression::ParenthesizedExpression(parenthesized) => {
            note_expression_only(result, &parenthesized.expression)
        }
        Expression::TSAsExpression(assertion) => {
            note_expression_only(result, &assertion.expression)
        }
        Expression::TSSatisfiesExpression(assertion) => {
            note_expression_only(result, &assertion.expression)
        }
        Expression::TSNonNullExpression(assertion) => {
            note_expression_only(result, &assertion.expression)
        }
        Expression::CallExpression(call) => {
            note_expression_only(result, &call.callee);
            for argument in &call.arguments {
                if let Some(expression) = argument.as_expression() {
                    note_expression_only(result, expression);
                } else if let oxc_ast::ast::Argument::SpreadElement(spread) = argument {
                    note_expression_only(result, &spread.argument);
                }
            }
        }
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::ThisExpression(_) => {}
        _ => {
            if let Some(capture) = result.occurrence_capture.as_mut() {
                capture.valid = false;
            }
        }
    }
}
