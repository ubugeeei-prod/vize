//! Component option objects shared by the Vue options-API rules.
//!
//! A definition is an exported options object, a `defineComponent` /
//! `defineOptions` argument, the options object passed to `*.component()`, or
//! an object value written inside a `components` map. Objects that appear only
//! in a `mixins` array are not definitions.

use oxc_ast::ast::{
    BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, Statement,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use vize_l0::FxHashMap;

pub(super) fn component_definitions<'a>(program: &'a Program<'a>) -> Vec<&'a ObjectExpression<'a>> {
    let bindings = option_bindings(program);
    let mut objects = Vec::new();
    if let Some(object) = export_options(program, &bindings) {
        push_unique(&mut objects, object);
    }
    let mut collector = Collector {
        bindings: &bindings,
        objects,
    };
    collector.visit_program(program);
    expand_components(&mut collector.objects, &bindings);
    collector.objects
}

fn option_bindings<'a>(program: &'a Program<'a>) -> FxHashMap<&'a str, &'a ObjectExpression<'a>> {
    let mut bindings = FxHashMap::default();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let (Some(init), BindingPattern::BindingIdentifier(identifier)) =
                (declarator.init.as_ref(), &declarator.id)
            else {
                continue;
            };
            if let Some(object) = object_from_expression(init, &bindings) {
                bindings.insert(identifier.name.as_str(), object);
            }
        }
    }
    bindings
}

fn export_options<'a>(
    program: &'a Program<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, bindings) {
            return Some(object);
        }
    }
    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => object_from_call(call, bindings),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            object_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            object_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            object_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            object_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn object_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => object_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        _ => None,
    }
}

fn object_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
        return None;
    };
    if !matches!(
        callee.name.as_str(),
        "defineComponent" | "_defineComponent" | "defineOptions"
    ) {
        return None;
    }
    let argument = call.arguments.first()?;
    let expression = argument.as_expression()?;
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::CallExpression(nested) => object_from_call(nested, bindings),
        _ => None,
    }
}

fn registered_component<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let member = call.callee.get_inner_expression().as_member_expression()?;
    if member.static_property_name() != Some("component") {
        return None;
    }
    let argument = call.arguments.get(1)?;
    let expression = argument.as_expression()?;
    object_from_expression(expression, bindings)
}

fn expand_components<'a>(
    objects: &mut Vec<&'a ObjectExpression<'a>>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    let mut index = 0;
    while let Some(options) = objects.get(index).copied() {
        for extra in component_map_values(options, bindings) {
            push_unique(objects, extra);
        }
        index += 1;
    }
}

fn component_map_values<'a>(
    options: &'a ObjectExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Vec<&'a ObjectExpression<'a>> {
    let mut found = Vec::new();
    let Some(value) = static_property(options, "components") else {
        return found;
    };
    let Some(map) = as_object_expression(value, bindings) else {
        return found;
    };
    for property in &map.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if let Some(object) = object_from_expression(&property.value, bindings)
            && object.span.start != map.span.start
        {
            found.push(object);
        }
    }
    found
}

fn as_object_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        _ => None,
    }
}

pub(super) fn static_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if property_key_name(&property.key) == Some(name) {
            return Some(&property.value);
        }
    }
    None
}

pub(super) fn property_key_name<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn push_unique<'a>(objects: &mut Vec<&'a ObjectExpression<'a>>, object: &'a ObjectExpression<'a>) {
    if objects
        .iter()
        .any(|existing| existing.span.start == object.span.start)
    {
        return;
    }
    objects.push(object);
}

struct Collector<'ast, 'map> {
    bindings: &'map FxHashMap<&'ast str, &'ast ObjectExpression<'ast>>,
    objects: Vec<&'ast ObjectExpression<'ast>>,
}

impl<'ast> Visit<'ast> for Collector<'ast, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'ast>) {
        // SAFETY: `call` points at a node inside the `Program` borrowed for
        // `'ast`. The visitor only reborrows it for this callback. Extending
        // that reborrow back to `'ast` is sound because the collected objects
        // are returned only for the same program, the walk is single-threaded,
        // and nothing moves or frees the arena node while those references live.
        let call = unsafe { &*(call as *const CallExpression<'ast>) };
        if let Some(object) = object_from_call(call, self.bindings)
            .or_else(|| registered_component(call, self.bindings))
        {
            push_unique(&mut self.objects, object);
        }
        walk_call_expression(self, call);
    }
}
