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
//!     return this.$slots.default
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
    BindingPattern, Expression, Function, MemberExpression, Program, SpreadElement,
    StaticMemberExpression, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_call_expression, walk_function,
        walk_identifier_reference, walk_spread_element, walk_static_member_expression,
        walk_variable_declarator,
    },
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{CompactString, FxHashMap, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-slots-as-functions",
    description: "Require $slots properties to be called as functions",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "`$slots` properties are functions in Vue 3 and must be called.";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    SlotFn,
    SlotsObject,
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
            suppressed: FxHashSet::default(),
        };
        visitor.visit_program(program);
    }
}

struct SlotsVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    scopes: Vec<FxHashMap<CompactString, Kind>>,
    suppressed: FxHashSet<u32>,
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
        if let Some(init) = &declarator.init {
            self.note_init(&declarator.id, init);
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        self.note_callee(&call.callee);
        walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if self.is_slot_member_expr(member) && !self.suppressed.contains(&member.span.start) {
            self.report(member.span);
        }
        walk_static_member_expression(self, member);
    }

    fn visit_identifier_reference(&mut self, identifier: &oxc_ast::ast::IdentifierReference<'a>) {
        if self.lookup(identifier.name.as_str()) == Some(Kind::SlotFn)
            && !self.suppressed.contains(&identifier.span.start)
        {
            self.report(identifier.span);
        }
        walk_identifier_reference(self, identifier);
    }

    fn visit_spread_element(&mut self, spread: &SpreadElement<'a>) {
        if self.is_bare_slots(&spread.argument) {
            self.report(spread.span);
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

    fn note_init(&mut self, pattern: &BindingPattern<'_>, init: &Expression<'_>) {
        if self.is_bare_slots(init) {
            if let BindingPattern::BindingIdentifier(identifier) = pattern {
                self.bind(identifier.name.as_str(), Kind::SlotsObject);
            } else {
                self.report(init.span());
            }
            return;
        }
        if self.is_slot_property(init)
            && let BindingPattern::BindingIdentifier(identifier) = pattern
        {
            self.bind(identifier.name.as_str(), Kind::SlotFn);
            if let Some(member) = member_of(init) {
                self.suppressed.insert(member.span().start);
            }
        }
    }

    fn note_callee(&mut self, callee: &Expression<'_>) {
        let callee = callee.get_inner_expression();
        if let Expression::Identifier(identifier) = callee
            && self.lookup(identifier.name.as_str()) == Some(Kind::SlotFn)
        {
            self.suppressed.insert(identifier.span.start);
            return;
        }
        if let Some(member) = member_of(callee)
            && self.is_slot_member(member)
        {
            self.suppressed.insert(member.span().start);
        }
    }

    fn is_slot_member_expr(&self, member: &StaticMemberExpression<'_>) -> bool {
        let name = member.property.name.as_str();
        name != "$slots" && self.is_slots_object(&member.object)
    }

    fn is_slot_member(&self, member: &MemberExpression<'_>) -> bool {
        let Some(name) = member.static_property_name() else {
            return false;
        };
        name != "$slots" && self.is_slots_object(member.object())
    }

    fn is_slot_property(&self, expression: &Expression<'_>) -> bool {
        member_of(expression).is_some_and(|member| self.is_slot_member(member))
    }

    fn is_slots_object(&self, expression: &Expression<'_>) -> bool {
        self.is_bare_slots(expression)
    }

    fn is_bare_slots(&self, expression: &Expression<'_>) -> bool {
        if let Some(member) = member_of(expression) {
            return member.static_property_name() == Some("$slots") && is_this(member.object());
        }
        matches!(
            expression.get_inner_expression(),
            Expression::Identifier(identifier)
                if self.lookup(identifier.name.as_str()) == Some(Kind::SlotsObject)
        )
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
    fn flags_non_call_slot_uses_and_allows_calls() {
        assert_eq!(
            lint("export default { render() { return this.$slots.default } }").error_count,
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
        assert_eq!(lint(source).error_count, 2);
    }
}
