//! The shared JS-module dialect gate (P1-7). A retained TS-dialect AST drives the
//! rewrite only when the legacy `SourceType::default().with_module(true)`
//! parse of the same bytes is provably the same parse; everything the scan
//! rejects keeps the legacy re-parse chain, whose *failure* modes (raw
//! passthrough) are part of the shipped bytes.

use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;

/// Return whether a retained TypeScript expression can bypass the legacy JavaScript re-parse.
pub fn js_module_compatible(ast: &oxc_ast_types::Expression<'_>, source: &str) -> bool {
    // Exact lexer-owned identifiers have no nested syntax to classify. Keeping
    // this admission in the shared gate avoids a full visitor for the dominant
    // template-expression shape while retaining the strict-module refusal.
    if let oxc_ast_types::Expression::Identifier(identifier) = ast
        && source == identifier.name.as_str()
    {
        return !strict_mode_divergent_identifier(identifier.name.as_str());
    }
    if source.contains("<!--") || source.contains("-->") {
        return false;
    }
    let mut scan = JsModuleCompatScan { compatible: true };
    scan.visit_expression(ast);
    scan.compatible
}

fn strict_mode_divergent_identifier(name: &str) -> bool {
    matches!(
        name,
        "implements"
            | "interface"
            | "let"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
            | "yield"
            | "await"
            | "eval"
            | "arguments"
    )
}

fn sloppy_only_numeric_raw(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    matches!(bytes, [b'0', second, ..] if second.is_ascii_digit())
}

fn sloppy_only_string_raw(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut rest = bytes;
    while let [first, second, tail @ ..] = rest {
        if *first == b'\\' {
            if second.is_ascii_digit() {
                return true;
            }
            rest = tail;
        } else if let [_, after @ ..] = rest {
            rest = after;
        }
    }
    false
}

/// Positive, exhaustive admission for the expression enum. A new oxc
/// expression kind must be classified here before it can be treated as
/// JavaScript. TypeScript, JSX, and V8-only syntax are never admitted.
fn expression_kind_is_javascript(expression: &oxc_ast_types::Expression<'_>) -> bool {
    match expression {
        oxc_ast_types::Expression::BooleanLiteral(_)
        | oxc_ast_types::Expression::NullLiteral(_)
        | oxc_ast_types::Expression::NumericLiteral(_)
        | oxc_ast_types::Expression::BigIntLiteral(_)
        | oxc_ast_types::Expression::RegExpLiteral(_)
        | oxc_ast_types::Expression::StringLiteral(_)
        | oxc_ast_types::Expression::TemplateLiteral(_)
        | oxc_ast_types::Expression::Identifier(_)
        | oxc_ast_types::Expression::Super(_)
        | oxc_ast_types::Expression::ArrayExpression(_)
        | oxc_ast_types::Expression::ArrowFunctionExpression(_)
        | oxc_ast_types::Expression::AssignmentExpression(_)
        | oxc_ast_types::Expression::AwaitExpression(_)
        | oxc_ast_types::Expression::BinaryExpression(_)
        | oxc_ast_types::Expression::CallExpression(_)
        | oxc_ast_types::Expression::ChainExpression(_)
        | oxc_ast_types::Expression::ClassExpression(_)
        | oxc_ast_types::Expression::ConditionalExpression(_)
        | oxc_ast_types::Expression::FunctionExpression(_)
        | oxc_ast_types::Expression::ImportExpression(_)
        | oxc_ast_types::Expression::LogicalExpression(_)
        | oxc_ast_types::Expression::NewExpression(_)
        | oxc_ast_types::Expression::ObjectExpression(_)
        | oxc_ast_types::Expression::ParenthesizedExpression(_)
        | oxc_ast_types::Expression::SequenceExpression(_)
        | oxc_ast_types::Expression::TaggedTemplateExpression(_)
        | oxc_ast_types::Expression::ThisExpression(_)
        | oxc_ast_types::Expression::UnaryExpression(_)
        | oxc_ast_types::Expression::UpdateExpression(_)
        | oxc_ast_types::Expression::YieldExpression(_)
        | oxc_ast_types::Expression::PrivateInExpression(_)
        | oxc_ast_types::Expression::ImportMeta(_)
        | oxc_ast_types::Expression::NewTarget(_)
        | oxc_ast_types::Expression::ComputedMemberExpression(_)
        | oxc_ast_types::Expression::StaticMemberExpression(_)
        | oxc_ast_types::Expression::PrivateFieldExpression(_) => true,
        oxc_ast_types::Expression::JSXElement(_)
        | oxc_ast_types::Expression::JSXFragment(_)
        | oxc_ast_types::Expression::TSAsExpression(_)
        | oxc_ast_types::Expression::TSSatisfiesExpression(_)
        | oxc_ast_types::Expression::TSTypeAssertion(_)
        | oxc_ast_types::Expression::TSNonNullExpression(_)
        | oxc_ast_types::Expression::TSInstantiationExpression(_)
        | oxc_ast_types::Expression::V8IntrinsicExpression(_) => false,
    }
}

struct JsModuleCompatScan {
    compatible: bool,
}

impl JsModuleCompatScan {
    #[inline]
    fn reject(&mut self) {
        self.compatible = false;
    }
}

impl<'a> Visit<'a> for JsModuleCompatScan {
    fn visit_expression(&mut self, it: &oxc_ast_types::Expression<'a>) {
        if !expression_kind_is_javascript(it) {
            self.reject();
            return;
        }
        walk::walk_expression(self, it);
    }
    fn visit_statement(&mut self, it: &oxc_ast_types::Statement<'a>) {
        if it.is_typescript_syntax() {
            self.reject();
            return;
        }
        walk::walk_statement(self, it);
    }
    fn visit_ts_type(&mut self, _it: &oxc_ast_types::TSType<'a>) {
        self.reject();
    }
    fn visit_ts_type_parameter_declaration(
        &mut self,
        _it: &oxc_ast_types::TSTypeParameterDeclaration<'a>,
    ) {
        self.reject();
    }
    fn visit_ts_type_parameter_instantiation(
        &mut self,
        _it: &oxc_ast_types::TSTypeParameterInstantiation<'a>,
    ) {
        self.reject();
    }
    fn visit_ts_as_expression(&mut self, _it: &oxc_ast_types::TSAsExpression<'a>) {
        self.reject();
    }
    fn visit_ts_satisfies_expression(&mut self, _it: &oxc_ast_types::TSSatisfiesExpression<'a>) {
        self.reject();
    }
    fn visit_ts_type_assertion(&mut self, _it: &oxc_ast_types::TSTypeAssertion<'a>) {
        self.reject();
    }
    fn visit_ts_non_null_expression(&mut self, _it: &oxc_ast_types::TSNonNullExpression<'a>) {
        self.reject();
    }
    fn visit_ts_instantiation_expression(
        &mut self,
        _it: &oxc_ast_types::TSInstantiationExpression<'a>,
    ) {
        self.reject();
    }
    fn visit_decorator(&mut self, _it: &oxc_ast_types::Decorator<'a>) {
        self.reject();
    }
    fn visit_function(&mut self, _it: &oxc_ast_types::Function<'a>, _flags: ScopeFlags) {
        self.reject();
    }
    fn visit_class(&mut self, _it: &oxc_ast_types::Class<'a>) {
        self.reject();
    }
    fn visit_with_statement(&mut self, _it: &oxc_ast_types::WithStatement<'a>) {
        self.reject();
    }
    fn visit_formal_parameter(&mut self, it: &oxc_ast_types::FormalParameter<'a>) {
        if !it.decorators.is_empty()
            || it.type_annotation.is_some()
            || it.optional
            || it.accessibility.is_some()
            || it.readonly
            || it.r#override
        {
            self.reject();
            return;
        }
        walk::walk_formal_parameter(self, it);
    }
    fn visit_identifier_reference(&mut self, it: &oxc_ast_types::IdentifierReference<'a>) {
        if strict_mode_divergent_identifier(it.name.as_str()) {
            self.reject();
        }
    }
    fn visit_binding_identifier(&mut self, it: &oxc_ast_types::BindingIdentifier<'a>) {
        if strict_mode_divergent_identifier(it.name.as_str()) {
            self.reject();
        }
    }
    fn visit_label_identifier(&mut self, it: &oxc_ast_types::LabelIdentifier<'a>) {
        if strict_mode_divergent_identifier(it.name.as_str()) {
            self.reject();
        }
    }
    fn visit_numeric_literal(&mut self, it: &oxc_ast_types::NumericLiteral<'a>) {
        if it
            .raw
            .as_ref()
            .is_some_and(|raw| sloppy_only_numeric_raw(raw))
        {
            self.reject();
        }
    }
    fn visit_string_literal(&mut self, it: &oxc_ast_types::StringLiteral<'a>) {
        if it
            .raw
            .as_ref()
            .is_some_and(|raw| sloppy_only_string_raw(raw))
        {
            self.reject();
        }
    }
    fn visit_unary_expression(&mut self, it: &oxc_ast_types::UnaryExpression<'a>) {
        if it.operator == oxc_ast_types::UnaryOperator::Delete
            && matches!(
                it.argument.get_inner_expression(),
                oxc_ast_types::Expression::Identifier(_)
            )
        {
            self.reject();
            return;
        }
        walk::walk_unary_expression(self, it);
    }
}

#[cfg(test)]
mod tests {
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_l0::Allocator;

    use super::js_module_compatible;

    fn compatible(source: &str) -> bool {
        let allocator = Allocator::new();
        let expression = Parser::new(allocator.as_oxc(), source, SourceType::ts())
            .parse_expression()
            .expect("TypeScript expression");
        js_module_compatible(&expression, source)
    }

    #[test]
    fn typescript_declarations_in_a_function_body_are_not_javascript() {
        assert!(!compatible("() => { void Child; type T = typeof Other }"));
        assert!(!compatible("() => { interface I { id: number } pick() }"));
        assert!(compatible("() => { void Child }"));
    }

    #[test]
    fn typescript_expression_kinds_and_fields_are_fail_closed() {
        for source in [
            "value as unknown",
            "value satisfies unknown",
            "<unknown>value",
            "value!",
            "value! = next",
            "value?.property!",
            "generic<string>",
            "(value: string) => value",
            "<T>(value: T) => value",
        ] {
            assert!(!compatible(source), "{source}");
        }
        for source in [
            "value",
            "generic(value)",
            "(value) => value",
            "value?.property ?? fallback",
        ] {
            assert!(compatible(source), "{source}");
        }
    }
}
