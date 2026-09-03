//! The JS-module dialect gate (`vize_atelier_core::retained::js_module_compatible`,
//! Davinci P1-7), ported verbatim: the retained TS-dialect AST drives the
//! rewrite only when the legacy `SourceType::default().with_module(true)`
//! parse of the same bytes is provably the same parse; everything the scan
//! rejects keeps the legacy re-parse chain, whose *failure* modes (raw
//! passthrough) are part of the shipped bytes.

use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;

pub(super) fn js_module_compatible(ast: &oxc_ast_types::Expression<'_>, source: &str) -> bool {
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
    bytes.len() > 1 && bytes[0] == b'0' && bytes[1].is_ascii_digit()
}

fn sloppy_only_string_raw(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'\\' {
            if bytes[index + 1].is_ascii_digit() {
                return true;
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    false
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
    fn visit_ts_type_annotation(&mut self, _it: &oxc_ast_types::TSTypeAnnotation<'a>) {
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
    fn visit_ts_enum_declaration(&mut self, _it: &oxc_ast_types::TSEnumDeclaration<'a>) {
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
