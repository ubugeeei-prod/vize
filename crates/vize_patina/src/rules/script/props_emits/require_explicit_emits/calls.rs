//! Emit calls a component dispatches to itself, matched as upstream's
//! emit-call visitors do: a read of the `<script setup>` `defineEmits`
//! binding, a Vue object's `this.$emit`, and the `emit` of its `setup` context.

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, CallExpression, Expression, FormalParameters, IdentifierReference,
        MemberExpression, ObjectExpression, ObjectPropertyKind, Program, VariableDeclarationKind,
    },
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_expression},
};
use oxc_semantic::Semantic;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

use vize_l0::CompactString;

use super::declared::{Emits, options_emits};
use super::report;
use crate::rules::script::ScriptLintResult;
use crate::rules::script::vue_object::{VueObjectKind, VueObjects};

/// `<script setup>`'s declaration, live from its macro onwards.
pub(super) struct SetupOwner<'s> {
    pub(super) start: u32,
    pub(super) emits: &'s Emits,
    pub(super) emit: Option<SymbolId>,
}

/// A Vue object being visited.
pub(super) struct ObjectOwner {
    kind: VueObjectKind,
    emits: Emits,
    /// `setup(props, { emit })`.
    emit: Option<SymbolId>,
    /// `setup(props, ctx)`, dispatching through `ctx.emit`.
    context: Option<SymbolId>,
}

pub(super) struct EmitCallVisitor<'s, 'a> {
    pub(super) semantic: &'s Semantic<'a>,
    /// `update:<model>` events with the start of the declaring macro.
    pub(super) models: &'s [(CompactString, u32)],
    pub(super) objects: &'s VueObjects,
    pub(super) setup: Option<SetupOwner<'s>>,
    pub(super) stack: Vec<ObjectOwner>,
    /// The object the template sees without `<script setup>`: the exported one,
    /// or else the last marked or defined one to finish.
    pub(super) component: Option<(VueObjectKind, Emits)>,
    pub(super) offset: u32,
    pub(super) result: &'s mut ScriptLintResult,
    /// Collect every dispatched literal event instead of checking it.
    pub(super) dispatched: Option<Vec<(CompactString, Span)>>,
}

/// Every literal event `program` dispatches to its own component.
///
/// `setup_emit` is the symbol of `const emit = defineEmits()`.
pub(crate) fn dispatched_events<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'a>,
    objects: &VueObjects,
    setup_emit: Option<SymbolId>,
) -> Vec<(CompactString, Span)> {
    let unknown = Emits::Unknown;
    let mut result = ScriptLintResult::default();
    let mut visitor = EmitCallVisitor {
        semantic,
        models: &[],
        objects,
        setup: setup_emit.map(|emit| SetupOwner {
            start: 0,
            emits: &unknown,
            emit: Some(emit),
        }),
        stack: Vec::new(),
        component: None,
        offset: 0,
        result: &mut result,
        dispatched: Some(Vec::new()),
    };
    visitor.visit_program(program);
    visitor.dispatched.unwrap_or_default()
}

impl<'a> Visit<'a> for EmitCallVisitor<'_, 'a> {
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        let Some(kind) = self.objects.kind(it) else {
            walk_object_expression(self, it);
            return;
        };
        let (emit, context) = setup_context(it);
        self.stack.push(ObjectOwner {
            kind,
            emits: options_emits(it),
            emit,
            context,
        });
        walk_object_expression(self, it);
        let Some(owner) = self.stack.pop() else {
            return;
        };
        let exported = matches!(self.component, Some((VueObjectKind::Export, _)));
        if !exported && owner.kind != VueObjectKind::Instance {
            self.component = Some((owner.kind, owner.emits));
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some((name, span)) = literal_event(it) {
            let callee = Callee::of(&it.callee);
            let setup = self
                .setup
                .as_ref()
                .filter(|setup| it.span.start > setup.start)
                .filter(|setup| self.dispatches(callee, setup.emit, None))
                .map(|setup| setup.emits);
            if self.dispatched.is_some() {
                let owner = self
                    .stack
                    .last()
                    .is_some_and(|owner| self.dispatches(callee, owner.emit, owner.context));
                if (setup.is_some() || owner)
                    && let Some(dispatched) = self.dispatched.as_mut()
                {
                    dispatched.push((name, span));
                }
                walk_call_expression(self, it);
                return;
            }
            // Upstream visits in order: a later model does not cover this call.
            let model = self
                .models
                .iter()
                .any(|(model, start)| *model == name && *start < it.span.start);
            if !model && setup.is_some_and(|emits| !emits.declares(&name)) {
                self.report(&name, span);
            }
            let owner = self
                .stack
                .last()
                .filter(|owner| self.dispatches(callee, owner.emit, owner.context));
            if owner.is_some_and(|owner| !owner.emits.declares(&name)) {
                self.report(&name, span);
            }
        }
        walk_call_expression(self, it);
    }
}

impl<'a> EmitCallVisitor<'_, 'a> {
    /// Whether `callee` dispatches the owner's events: its `emit` binding,
    /// `ctx.emit`, or `this.$emit`.
    fn dispatches(
        &self,
        callee: Option<Callee<'_, 'a>>,
        emit: Option<SymbolId>,
        context: Option<SymbolId>,
    ) -> bool {
        let member = match callee {
            Some(Callee::Identifier(identifier)) => {
                return emit.is_some() && self.symbol(identifier) == emit;
            }
            Some(Callee::Member(member)) => member,
            None => return false,
        };
        let object = member.object().without_parentheses();
        match member.static_property_name() {
            Some("emit") => {
                context.is_some()
                    && matches!(object, Expression::Identifier(id) if self.symbol(id) == context)
            }
            Some("$emit") => self.is_this(object),
            _ => false,
        }
    }

    /// Upstream's `isThis`: `this`, or a `const` initialized to it.
    fn is_this(&self, expression: &Expression<'a>) -> bool {
        let identifier = match expression {
            Expression::ThisExpression(_) => return true,
            Expression::Identifier(identifier) => identifier,
            _ => return false,
        };
        let Some(symbol) = self.symbol(identifier) else {
            return false;
        };
        if !self
            .semantic
            .scoping()
            .symbol_redeclarations(symbol)
            .is_empty()
        {
            return false;
        }
        matches!(
            self.semantic.symbol_declaration(symbol).kind(),
            AstKind::VariableDeclarator(declarator)
                if declarator.kind == VariableDeclarationKind::Const
                    && matches!(declarator.id, BindingPattern::BindingIdentifier(_))
                    && matches!(
                        declarator.init.as_ref().map(Expression::without_parentheses),
                        Some(Expression::ThisExpression(_))
                    )
        )
    }

    fn symbol(&self, identifier: &IdentifierReference<'a>) -> Option<SymbolId> {
        let scoping = self.semantic.scoping();
        scoping
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }

    fn report(&mut self, name: &str, span: Span) {
        report(
            name,
            self.offset + span.start,
            self.offset + span.end,
            self.result,
        );
    }
}

/// The `emit` binding and context parameter of the object's `setup` function.
pub(crate) fn setup_context(object: &ObjectExpression<'_>) -> (Option<SymbolId>, Option<SymbolId>) {
    let mut found = (None, None);
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.key.static_name().as_deref() != Some("setup") {
            continue;
        }
        let params = match property.value.without_parentheses() {
            Expression::FunctionExpression(function) => &function.params,
            Expression::ArrowFunctionExpression(function) => &function.params,
            _ => continue,
        };
        found = context_parameter(params);
    }
    found
}

fn context_parameter(params: &FormalParameters<'_>) -> (Option<SymbolId>, Option<SymbolId>) {
    let Some(context) = params.items.get(1) else {
        return (None, None);
    };
    if context.initializer.is_some() {
        return (None, None);
    }
    match &context.pattern {
        BindingPattern::BindingIdentifier(identifier) => (None, identifier.symbol_id.get()),
        BindingPattern::ObjectPattern(pattern) => {
            let emit = pattern
                .properties
                .iter()
                .find(|property| property.key.static_name().as_deref() == Some("emit"));
            match emit.map(|property| &property.value) {
                Some(BindingPattern::BindingIdentifier(identifier)) => {
                    (identifier.symbol_id.get(), None)
                }
                _ => (None, None),
            }
        }
        _ => (None, None),
    }
}

/// The literal first argument: a string, or a template without substitutions.
fn literal_event(call: &CallExpression<'_>) -> Option<(CompactString, Span)> {
    match call
        .arguments
        .first()?
        .as_expression()?
        .without_parentheses()
    {
        Expression::StringLiteral(literal) => {
            Some((CompactString::new(literal.value.as_str()), literal.span))
        }
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => Some((
            CompactString::new(literal.single_quasi()?.as_str()),
            literal.span,
        )),
        _ => None,
    }
}

/// A call's callee through upstream's `skipChainExpression`; parentheses do
/// not exist in ESTree.
#[derive(Clone, Copy)]
enum Callee<'b, 'a> {
    Identifier(&'b IdentifierReference<'a>),
    Member(&'b MemberExpression<'a>),
}

impl<'b, 'a> Callee<'b, 'a> {
    fn of(expression: &'b Expression<'a>) -> Option<Self> {
        match expression.without_parentheses() {
            Expression::Identifier(identifier) => Some(Self::Identifier(identifier)),
            Expression::ChainExpression(chain) => {
                chain.expression.as_member_expression().map(Self::Member)
            }
            expression => expression.as_member_expression().map(Self::Member),
        }
    }
}
