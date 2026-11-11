//! `defineModel`, `withDefaults`, and props-destructure defaults.
//!
//! Runtime `default` members are checked in the parent module. These forms
//! write the same value somewhere else: the model options, the `withDefaults`
//! argument, or a destructuring initializer.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
    Program,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_variable_declarator},
};
use vize_l0::CompactString;

use super::{
    DefaultSite, ScriptLintResult, descriptor_member, native_type_names, validate_default,
    validate_descriptor,
};

mod infer;

pub(super) fn check_setup_defaults(
    program: &Program<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let mut visitor = SetupDefaults {
        program,
        offset,
        result,
        props_shadowed: super::super::compiler_macro_is_shadowed(program, "defineProps"),
        model_shadowed: super::super::compiler_macro_is_shadowed(program, "defineModel"),
        defaults_shadowed: super::super::compiler_macro_is_shadowed(program, "withDefaults"),
    };
    visitor.visit_program(program);
}

struct SetupDefaults<'a, 'out> {
    program: &'a Program<'a>,
    offset: usize,
    result: &'out mut ScriptLintResult,
    props_shadowed: bool,
    model_shadowed: bool,
    defaults_shadowed: bool,
}

impl<'a> Visit<'a> for SetupDefaults<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        match callee_name(call) {
            Some("defineModel") if !self.model_shadowed => {
                check_define_model(self.program, call, self.offset, self.result);
            }
            Some("withDefaults") if !self.defaults_shadowed && !self.props_shadowed => {
                check_with_defaults(self.program, call, self.offset, self.result);
            }
            _ => {}
        }
        walk_call_expression(self, call);
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'a>) {
        if !self.props_shadowed
            && let BindingPattern::ObjectPattern(pattern) = &declarator.id
            && let Some(init) = declarator.init.as_ref()
            && let Some(call) = define_props_call(init)
        {
            let props = prop_types(self.program, call);
            for property in &pattern.properties {
                if property.computed {
                    continue;
                }
                let Some(name) = key_name(&property.key) else {
                    continue;
                };
                let BindingPattern::AssignmentPattern(assignment) = &property.value else {
                    continue;
                };
                let Some(types) = props.iter().find(|(prop, _)| prop == name) else {
                    continue;
                };
                validate_default(
                    name,
                    &types.1,
                    &assignment.right,
                    DefaultSite::Assignment,
                    self.offset,
                    self.result,
                );
            }
        }
        walk_variable_declarator(self, declarator);
    }
}

fn check_define_model(
    program: &Program<'_>,
    call: &CallExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let mut name = "modelValue";
    let mut options_index = 0;
    if let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
        && let Some(literal) = static_string(argument)
    {
        name = literal;
        options_index = 1;
    }
    let options = call
        .arguments
        .get(options_index)
        .and_then(Argument::as_expression)
        .and_then(object_expr);
    if let Some(ty) = call
        .type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
    {
        let Some(options) = options else {
            return;
        };
        let Some(default_value) = descriptor_member(options, "default") else {
            return;
        };
        let types = infer::infer_types(program, ty);
        validate_default(
            name,
            &types,
            default_value,
            DefaultSite::Property,
            offset,
            result,
        );
        return;
    }
    if let Some(options) = options {
        validate_descriptor(name, options, offset, result);
    }
}

fn check_with_defaults(
    program: &Program<'_>,
    call: &CallExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let Some(props_call) = call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(define_props_call)
    else {
        return;
    };
    let Some(defaults) = call
        .arguments
        .get(1)
        .and_then(Argument::as_expression)
        .and_then(object_expr)
    else {
        return;
    };
    let props = prop_types(program, props_call);
    for property in &defaults.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = key_name(&property.key) else {
            continue;
        };
        let Some(types) = props.iter().find(|(prop, _)| prop == name) else {
            continue;
        };
        validate_default(
            name,
            &types.1,
            &property.value,
            DefaultSite::Property,
            offset,
            result,
        );
    }
}

fn prop_types(
    program: &Program<'_>,
    call: &CallExpression<'_>,
) -> Vec<(CompactString, Vec<&'static str>)> {
    if let Some(ty) = call
        .type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
    {
        return infer::collect_type_props(program, ty);
    }
    call.arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(object_expr)
        .map(runtime_prop_types)
        .unwrap_or_default()
}

fn runtime_prop_types(object: &ObjectExpression<'_>) -> Vec<(CompactString, Vec<&'static str>)> {
    let mut props = Vec::new();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = key_name(&property.key) else {
            continue;
        };
        let types = match property.value.get_inner_expression() {
            Expression::ObjectExpression(descriptor) => descriptor_member(descriptor, "type")
                .map(native_type_names)
                .unwrap_or_default(),
            value => native_type_names(value),
        };
        props.push((CompactString::new(name), types));
    }
    props
}

fn define_props_call<'a>(expression: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    match callee_name(call)? {
        "defineProps" => Some(call),
        "withDefaults" => call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .and_then(define_props_call),
        _ => None,
    }
}

fn callee_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn object_expr<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn static_string<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => Some(literal.value.as_str()),
        Expression::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.as_str())
        }
        _ => None,
    }
}

fn key_name<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        oxc_ast::ast::PropertyKey::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.as_str())
        }
        _ => None,
    }
}
