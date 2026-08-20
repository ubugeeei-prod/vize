//! Mutation targets for `defineProps` bindings in `<script setup>`.
//!
//! Symbol IDs are used instead of names so a function parameter or block-local
//! variable that shadows `props` cannot inherit the component prop's readonly
//! status. Only unresolved calls to Vue's compiler macros are accepted; a
//! user-defined function named `defineProps` is an ordinary local initializer.

use crate::rules::script::script_source_type;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentExpression, CallExpression, Expression, IdentifierReference, Program,
    SimpleAssignmentTarget, Statement, UpdateExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_update_expression},
};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolId;
use vize_carton::{FxHashSet, String};

pub(super) struct ScriptPropMutation {
    pub(super) target: String,
    pub(super) span: Span,
}

pub(super) fn find_prop_mutations(source: &str) -> Vec<ScriptPropMutation> {
    if memchr::memmem::find(source.as_bytes(), b"defineProps").is_none() {
        return Vec::new();
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, script_source_type()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }

    let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
    let scoping = semantic.scoping();
    let prop_symbols = collect_prop_symbols(&parsed.program, scoping);
    if prop_symbols.is_empty() {
        return Vec::new();
    }

    let mut collector = MutationCollector {
        source,
        scoping,
        prop_symbols: &prop_symbols,
        mutations: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    collector
        .mutations
        .sort_unstable_by_key(|mutation| (mutation.span.start, mutation.span.end));
    collector.mutations
}

fn collect_prop_symbols(program: &Program<'_>, scoping: &Scoping) -> FxHashSet<SymbolId> {
    let mut symbols = FxHashSet::default();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(initializer) = declarator.init.as_ref() else {
                continue;
            };
            if !is_props_initializer(initializer, scoping) {
                continue;
            }
            for identifier in declarator.id.get_binding_identifiers() {
                if let Some(symbol_id) = identifier.symbol_id.get() {
                    symbols.insert(symbol_id);
                }
            }
        }
    }
    symbols
}

fn is_props_initializer(expression: &Expression<'_>, scoping: &Scoping) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    if is_unresolved_macro_call(call, "defineProps", scoping) {
        return true;
    }
    if !is_unresolved_macro_call(call, "withDefaults", scoping) {
        return false;
    }
    call.arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .is_some_and(|argument| is_define_props_expression(argument, scoping))
}

fn is_define_props_expression(expression: &Expression<'_>, scoping: &Scoping) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::CallExpression(call) if is_unresolved_macro_call(call, "defineProps", scoping)
    )
}

fn is_unresolved_macro_call(call: &CallExpression<'_>, name: &str, scoping: &Scoping) -> bool {
    let Expression::Identifier(callee) = &call.callee else {
        return false;
    };
    if callee.name.as_str() != name {
        return false;
    }
    callee
        .reference_id
        .get()
        .is_some_and(|reference_id| scoping.get_reference(reference_id).symbol_id().is_none())
}

struct MutationCollector<'s> {
    source: &'s str,
    scoping: &'s Scoping,
    prop_symbols: &'s FxHashSet<SymbolId>,
    mutations: Vec<ScriptPropMutation>,
}

impl MutationCollector<'_> {
    fn record(&mut self, target: &SimpleAssignmentTarget<'_>) {
        let Some(reference) = target_reference(target) else {
            return;
        };
        let Some(reference_id) = reference.reference_id.get() else {
            return;
        };
        let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id() else {
            return;
        };
        if !self.prop_symbols.contains(&symbol_id) {
            return;
        }

        let span = target.span();
        let Some(target) = self.source.get(span.start as usize..span.end as usize) else {
            return;
        };
        self.mutations.push(ScriptPropMutation {
            target: String::from(target),
            span,
        });
    }
}

impl<'a> Visit<'a> for MutationCollector<'_> {
    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression<'a>) {
        if let Some(target) = expression.left.as_simple_assignment_target() {
            self.record(target);
        }
        walk_assignment_expression(self, expression);
    }

    fn visit_update_expression(&mut self, expression: &UpdateExpression<'a>) {
        self.record(&expression.argument);
        walk_update_expression(self, expression);
    }
}

fn target_reference<'a>(
    target: &'a SimpleAssignmentTarget<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => Some(identifier),
        other if other.is_member_expression() => {
            root_reference(other.to_member_expression().object())
        }
        other => other.get_expression().and_then(root_reference),
    }
}

fn root_reference<'a>(expression: &'a Expression<'a>) -> Option<&'a IdentifierReference<'a>> {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier),
        expression if expression.is_member_expression() => {
            root_reference(expression.to_member_expression().object())
        }
        _ => None,
    }
}
