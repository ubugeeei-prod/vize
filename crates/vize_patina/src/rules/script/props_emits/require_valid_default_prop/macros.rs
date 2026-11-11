//! `defineModel`, `withDefaults`, and props-destructure defaults.
//!
//! Runtime `default` members are checked in the parent module. These forms
//! write the same value somewhere else: the model options, the `withDefaults`
//! argument, or a destructuring initializer.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
    Program, TSType,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_variable_declarator},
};
use oxc_span::GetSpan;
use vize_l0::CompactString;

use super::{
    DefaultSite, ScriptLintResult, descriptor_member, native_type_names, validate_default,
    validate_descriptor,
};

mod infer;

pub(super) fn check_setup_defaults(
    program: &Program<'_>,
    source: &str,
    offset: usize,
    world: Option<&vize_croquis::types::ResolvedTypeWorld>,
    result: &mut ScriptLintResult,
) {
    let mut visitor = SetupDefaults {
        program,
        source,
        offset,
        world,
        result,
        props_shadowed: super::super::compiler_macro_is_shadowed(program, "defineProps"),
        model_shadowed: super::super::compiler_macro_is_shadowed(program, "defineModel"),
        defaults_shadowed: super::super::compiler_macro_is_shadowed(program, "withDefaults"),
    };
    visitor.visit_program(program);
}

struct SetupDefaults<'a, 'w, 'out> {
    program: &'a Program<'a>,
    source: &'a str,
    offset: usize,
    world: Option<&'w vize_croquis::types::ResolvedTypeWorld>,
    result: &'out mut ScriptLintResult,
    props_shadowed: bool,
    model_shadowed: bool,
    defaults_shadowed: bool,
}

impl<'a> Visit<'a> for SetupDefaults<'a, '_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        match callee_name(call) {
            Some("defineModel") if !self.model_shadowed => {
                check_define_model(self.program, call, self.offset, self.result);
            }
            Some("withDefaults") if !self.defaults_shadowed && !self.props_shadowed => {
                check_with_defaults(
                    self.program,
                    self.source,
                    call,
                    self.offset,
                    self.world,
                    self.result,
                );
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
            let props = prop_types(self.program, self.source, call, self.world);
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
    source: &str,
    call: &CallExpression<'_>,
    offset: usize,
    world: Option<&vize_croquis::types::ResolvedTypeWorld>,
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
    let props = prop_types(program, source, props_call, world);
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
    source: &str,
    call: &CallExpression<'_>,
    world: Option<&vize_croquis::types::ResolvedTypeWorld>,
) -> Vec<(CompactString, Vec<&'static str>)> {
    if let Some(ty) = call
        .type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
    {
        let local = infer::collect_type_props(program, ty);
        // An inline literal or a same-file interface already names every prop.
        // Querying the world as well would report the same default twice.
        if !local.is_empty() {
            return local;
        }
        return imported_prop_types(source, ty, world);
    }
    call.arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(object_expr)
        .map(runtime_prop_types)
        .unwrap_or_default()
}

/// Props of an imported `defineProps<CardProps>()` argument.
///
/// A same-file walk cannot see the module. An unresolved or partial type is
/// skipped: treating it as "no props" would hide a default that should be
/// checked once the import can be read.
fn imported_prop_types(
    source: &str,
    ty: &TSType<'_>,
    world: Option<&vize_croquis::types::ResolvedTypeWorld>,
) -> Vec<(CompactString, Vec<&'static str>)> {
    let Some(world) = world else {
        return Vec::new();
    };
    let span = ty.span();
    let Some(type_source) = source.get(span.start as usize..span.end as usize) else {
        return Vec::new();
    };
    let resolved = world.resolve_properties(type_source);
    if !resolved.complete {
        return Vec::new();
    }
    resolved
        .properties
        .into_iter()
        .filter_map(|property| {
            let types = native_types_from_annotation(property.prop_type.as_deref()?);
            if types.is_empty() {
                None
            } else {
                Some((property.name, types))
            }
        })
        .collect()
}

fn native_types_from_annotation(annotation: &str) -> Vec<&'static str> {
    let mut types = Vec::new();
    for part in annotation.split('|') {
        let Some(name) = native_type_from_annotation(part) else {
            continue;
        };
        if !types.contains(&name) {
            types.push(name);
        }
    }
    types
}

fn native_type_from_annotation(part: &str) -> Option<&'static str> {
    let mut text = part.trim();
    while text.len() >= 2 && text.starts_with('(') && text.ends_with(')') {
        text = text[1..text.len() - 1].trim();
    }
    match text {
        "string" | "String" => Some("String"),
        "number" | "Number" => Some("Number"),
        "boolean" | "Boolean" => Some("Boolean"),
        "bigint" | "BigInt" => Some("BigInt"),
        "symbol" | "Symbol" => Some("Symbol"),
        "object" | "Object" => Some("Object"),
        _ => None,
    }
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
