//! script/require-slots-as-functions
//!
//! In Vue 3 every `$slots` property is a function. Reading it without calling
//! it was the Vue 2 vnode-array shape.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   render() {
//!     return [this.$slots.default]
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   render() {
//!     return this.$slots.default()
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    ArrayExpression, AssignmentExpression, AssignmentTarget, BindingPattern, Expression, Function,
    MemberExpression, Program, SpreadElement, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_array_expression, walk_arrow_function_expression, walk_assignment_expression,
        walk_function, walk_member_expression, walk_spread_element, walk_variable_declarator,
    },
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-slots-as-functions",
    description: "Require $slots properties to be called as functions",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "`$slots` properties are functions in Vue 3 and must be called.";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    SlotFn(Span),
    NotSlot,
}

pub struct RequireSlotsAsFunctions;

impl ScriptRule for RequireSlotsAsFunctions {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !source.contains("$slots") {
            return;
        }
        let mut visitor = SlotsVisitor {
            offset,
            result,
            scopes: vec![FxHashMap::default()],
        };
        visitor.visit_program(program);
    }
}

struct SlotsVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    scopes: Vec<FxHashMap<CompactString, Kind>>,
}

impl<'a> Visit<'a> for SlotsVisitor<'_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.push_function_scope(function);
        walk_function(self, function, flags);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.scopes.push(FxHashMap::default());
        for parameter in &arrow.params.items {
            self.bind_pattern(&parameter.pattern, Kind::NotSlot);
        }
        if let Some(rest) = &arrow.params.rest {
            self.bind_pattern(&rest.rest.argument, Kind::NotSlot);
        }
        walk_arrow_function_expression(self, arrow);
        self.scopes.pop();
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        self.bind_pattern(&declarator.id, Kind::NotSlot);
        if let Some(init) = &declarator.init
            && let Some(origin) = self.slot_function_origin(init)
        {
            if let BindingPattern::BindingIdentifier(identifier) = &declarator.id {
                self.bind(identifier.name.as_str(), Kind::SlotFn(origin));
            } else {
                self.report(origin);
            }
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let Some(origin) = self.slot_function_origin(&assignment.right)
            && let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &assignment.left
        {
            self.assign(identifier.name.as_str(), Kind::SlotFn(origin));
        }
        walk_assignment_expression(self, assignment);
    }

    fn visit_member_expression(&mut self, member: &MemberExpression<'a>) {
        if let Some(origin) = self.slot_function_origin(member.object()) {
            self.report(origin);
        }
        if let MemberExpression::ComputedMemberExpression(computed) = member
            && let Some(origin) = self.slot_function_origin(&computed.expression)
        {
            self.report(origin);
        }
        walk_member_expression(self, member);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        for element in &array.elements {
            if let Some(expression) = element.as_expression()
                && let Some(origin) = self.slot_function_origin(expression)
            {
                self.report(origin);
            }
        }
        walk_array_expression(self, array);
    }

    fn visit_spread_element(&mut self, spread: &SpreadElement<'a>) {
        if let Some(origin) = self.slot_function_origin(&spread.argument) {
            self.report(origin);
        }
        walk_spread_element(self, spread);
    }
}

impl SlotsVisitor<'_> {
    fn push_function_scope(&mut self, function: &Function<'_>) {
        self.scopes.push(FxHashMap::default());
        for parameter in &function.params.items {
            self.bind_pattern(&parameter.pattern, Kind::NotSlot);
        }
        if let Some(rest) = &function.params.rest {
            self.bind_pattern(&rest.rest.argument, Kind::NotSlot);
        }
    }

    fn slot_function_origin(&self, expression: &Expression<'_>) -> Option<Span> {
        let expression = expression.get_inner_expression();
        if let Expression::Identifier(identifier) = expression {
            return match self.lookup(identifier.name.as_str()) {
                Some(Kind::SlotFn(origin)) => Some(origin),
                _ => None,
            };
        }
        member_of(expression).and_then(slot_property_span)
    }

    fn lookup(&self, name: &str) -> Option<Kind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn bind(&mut self, name: &str, kind: Kind) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(CompactString::new(name), kind);
        }
    }

    fn assign(&mut self, name: &str, kind: Kind) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            scope.insert(CompactString::new(name), kind);
        } else {
            self.bind(name, kind);
        }
    }

    fn bind_pattern(&mut self, pattern: &BindingPattern<'_>, kind: Kind) {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                self.bind(identifier.name.as_str(), kind);
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.bind_pattern(&property.value, kind);
                }
                if let Some(rest) = &object.rest {
                    self.bind_pattern(&rest.argument, kind);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.bind_pattern(element, kind);
                }
                if let Some(rest) = &array.rest {
                    self.bind_pattern(&rest.argument, kind);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.bind_pattern(&assignment.left, kind);
            }
        }
    }

    fn report(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end));
    }
}

fn member_of<'a>(expression: &'a Expression<'a>) -> Option<&'a MemberExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
        other => other.as_member_expression(),
    }
}

fn slot_property_span(member: &MemberExpression<'_>) -> Option<Span> {
    let slots = member_of(member.object())?;
    if slots.static_property_name() != Some("$slots") || !is_this(slots.object()) {
        return None;
    }
    match member {
        MemberExpression::ComputedMemberExpression(computed) => Some(computed.expression.span()),
        MemberExpression::StaticMemberExpression(static_member) => {
            Some(static_member.property.span)
        }
        MemberExpression::PrivateFieldExpression(_) => None,
    }
}

fn is_this(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::ThisExpression(_)
    )
}

#[cfg(test)]
mod tests {
    use super::RequireSlotsAsFunctions;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(RequireSlotsAsFunctions));
        linter.lint(source, 0)
    }

    #[test]
    fn matches_the_upstream_parent_contexts() {
        assert_eq!(
            lint("export default { render() { return this.$slots.default } }").error_count,
            0
        );
        assert_eq!(
            lint("export default { render() { return [this.$slots.default] } }").error_count,
            1
        );
        assert_eq!(
            lint("export default { render() { return this.$slots.default.length } }").error_count,
            1
        );
        assert_eq!(
            lint("export default { render() { return this.$slots.default() } }").error_count,
            0
        );
        assert_eq!(
            lint("export default { methods: { forward() { return [this.$attrs, this.$slots] } } }")
                .error_count,
            0
        );
    }

    #[test]
    fn tracks_slot_aliases() {
        let source = "export default { methods: { forward() { const children = this.$slots.foo; const slots = this.$slots; return [children, slots.foo, children(), slots.bar()] } } }";
        let result = lint(source);
        assert_eq!(result.error_count, 1);
        let property = source.find("foo").unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, property);
        assert_eq!(result.diagnostics[0].end, property + 3);
    }
}

#[cfg(test)]
mod presence_tests;

#[cfg(test)]
mod condition_tests;

#[cfg(test)]
mod typeof_tests;

#[cfg(test)]
mod conditional_tests;
