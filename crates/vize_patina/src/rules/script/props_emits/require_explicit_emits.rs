//! script/require-explicit-emits
//!
//! Require every emitted event to be declared in `defineEmits` / the Options API
//! `emits` option. An undeclared emit is invisible to tooling and (since
//! Vue 3.3) falls through as a native DOM listener on the root element, which is
//! rarely intended. Declaring emits documents the component's event surface.
//!
//! Port of
//! [`vue/require-explicit-emits`](https://eslint.vuejs.org/rules/require-explicit-emits.html),
//! with its scope:
//!
//! * A Vue object (see [`crate::rules::script::vue_object`]) declares what its
//!   `emits` option lists, and nothing when the option is absent. Its
//!   `this.$emit('x')` calls (also through `const vm = this`) and the `emit` of
//!   its `setup(props, { emit })` / `setup(props, ctx)` are checked against it.
//! * `<script setup>` declares what its `defineEmits` macro lists. Script calls
//!   of the macro's binding after the macro are checked; without a macro the
//!   script is not.
//! * The template's `$emit('x')` (and a call of the `defineEmits` binding) is
//!   checked against `<script setup>` when the SFC has one, and otherwise
//!   against the exported (or last defined) component object.
//! * A declaration that cannot be fully enumerated — a spread, a computed key, a
//!   type imported from elsewhere, an untyped `defineEmits()` — declares every
//!   event. Only literal event names are checked.
//!
//! ```ts
//! const emit = defineEmits(['change'])
//! emit('change') // ok
//! emit('input')  // reported: 'input' is not declared
//! ```

mod declared;
mod types;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, Expression, FormalParameters,
        IdentifierReference, MemberExpression, ObjectExpression, ObjectPropertyKind, Program,
        VariableDeclarationKind,
    },
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_expression},
};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

use vize_l0::{CompactString, FxHashSet};

use self::declared::{Emits, define_emits, options_emits};
use self::types::Resolver;
use super::super::template_scan::{DOLLAR_EMIT, for_each_template_call};
use super::super::vue_object::{VueObjectKind, VueObjects};
use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-explicit-emits",
    description: "Require emitted events to be declared in defineEmits or the emits option",
    default_severity: Severity::Warning,
};

/// Require every emitted event to be declared.
pub struct RequireExplicitEmits;

impl ScriptRule for RequireExplicitEmits {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn uses_template_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        // `emit` covers `$emit` and `ctx.emit`; `Emits` a renamed macro binding.
        if !source.contains("emit")
            && !source.contains("Emits")
            && !sfc
                .template_source
                .is_some_and(|template| template.contains("emit"))
        {
            return;
        }
        let built = SemanticBuilder::new().with_build_nodes(true).build(program);
        let semantic = &built.semantic;
        let resolver = Resolver {
            semantic,
            program,
            sibling: sfc.sibling_program,
        };
        let setup = sfc
            .is_script_setup
            .then(|| define_emits(&resolver))
            .flatten();
        let objects = VueObjects::collect(
            program,
            source,
            semantic,
            sfc.is_sfc && !sfc.is_script_setup,
        );
        let mut visitor = EmitCallVisitor {
            semantic,
            objects: &objects,
            setup: setup.as_ref().map(|setup| SetupOwner {
                start: setup.start,
                emits: &setup.emits,
                emit: setup.binding.and_then(|(_, symbol)| symbol),
            }),
            stack: Vec::new(),
            component: None,
            offset: offset as u32,
            result,
        };
        visitor.visit_program(program);
        let component = visitor.component.take();

        // The template sees `<script setup>` when there is one, and otherwise
        // the component of the only script block.
        if sfc.is_script_setup {
            let empty = Emits::Known(FxHashSet::default());
            let emits = setup.as_ref().map_or(&empty, |setup| &setup.emits);
            let binding = setup
                .as_ref()
                .and_then(|setup| setup.binding)
                .map(|(name, _)| name);
            check_template(emits, binding, sfc, result);
        } else if sfc.sole_script_block
            && let Some((_, emits)) = component.as_ref()
        {
            check_template(emits, None, sfc, result);
        }
    }
}

/// The template half: `$emit('x')` — or a call of the `defineEmits` binding —
/// naming an event the declaration does not contain.
fn check_template(
    declared: &Emits,
    binding: Option<&str>,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let Some((root, template_offset)) = sfc.template_ast() else {
        return;
    };
    for_each_template_call(root, |call| {
        if call.callee != DOLLAR_EMIT && Some(call.callee) != binding {
            return;
        }
        let Some(event) = call.first_string_argument else {
            return;
        };
        if !declared.declares(event.value) {
            report(
                event.value,
                template_offset + event.start,
                template_offset + event.end,
                result,
            );
        }
    });
}

/// `<script setup>`'s declaration, live from its macro onwards.
struct SetupOwner<'s> {
    start: u32,
    emits: &'s Emits,
    emit: Option<SymbolId>,
}

/// A Vue object being visited.
struct ObjectOwner {
    kind: VueObjectKind,
    emits: Emits,
    /// `setup(props, { emit })`.
    emit: Option<SymbolId>,
    /// `setup(props, ctx)`, dispatching through `ctx.emit`.
    context: Option<SymbolId>,
}

struct EmitCallVisitor<'s, 'a> {
    semantic: &'s Semantic<'a>,
    objects: &'s VueObjects,
    setup: Option<SetupOwner<'s>>,
    stack: Vec<ObjectOwner>,
    /// The object the template sees without `<script setup>`: the exported one,
    /// or else the last marked or defined one to finish.
    component: Option<(VueObjectKind, Emits)>,
    offset: u32,
    result: &'s mut ScriptLintResult,
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
            if setup.is_some_and(|emits| !emits.declares(&name)) {
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
fn setup_context(object: &ObjectExpression<'_>) -> (Option<SymbolId>, Option<SymbolId>) {
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
    match call.arguments.first()? {
        Argument::StringLiteral(literal) => {
            Some((CompactString::new(literal.value.as_str()), literal.span))
        }
        Argument::TemplateLiteral(literal) if literal.expressions.is_empty() => Some((
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

fn report(name: &str, start: u32, end: u32, result: &mut ScriptLintResult) {
    let mut message = CompactString::with_capacity(name.len() + 48);
    message.push_str("The emitted event '");
    message.push_str(name);
    message.push_str("' is not declared in defineEmits or the emits option.");

    let diagnostic = LintDiagnostic::warn(META.name, message, start, end)
        .with_label("undeclared emitted event", start, end)
        .with_help(
            "Add this event to the defineEmits declaration (or the Options API `emits` \
             option) so the component's event surface is explicit.",
        );
    result.add_diagnostic(diagnostic);
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
