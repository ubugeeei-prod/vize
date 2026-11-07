//! Conservative same-file resolution for defaults attached to `defineProps`.
//!
//! Both `require-default-prop` and `no-boolean-default` need to correlate a
//! typed prop member with defaults supplied by `withDefaults` or by a props
//! destructuring assignment. Keeping that correlation here prevents the two
//! rules from drifting on aliases, interfaces, and source ranges.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Declaration, Expression, ObjectExpression,
    ObjectPropertyKind, Program, Statement, TSSignature, TSType, TSTypeName,
};
use oxc_span::{GetSpan, Span};
use vize_l0::{CompactString, FxHashSet};

use super::{compiler_macro_is_shadowed, props_source::property_key_name};

#[derive(Clone)]
pub(crate) struct PropDefault {
    pub(super) name: CompactString,
    pub(super) span: Span,
}

#[derive(Clone)]
pub(crate) struct TypedProp {
    pub(crate) name: CompactString,
    pub(crate) span: Span,
    pub(crate) required: bool,
    pub(crate) boolean: bool,
}

pub(crate) struct SetupPropsDeclaration<'a> {
    pub(crate) runtime: Option<&'a ObjectExpression<'a>>,
    pub(crate) typed: Vec<TypedProp>,
    pub(crate) defaults: Vec<PropDefault>,
    pub(crate) uses_destructure: bool,
    pub(crate) uses_with_defaults: bool,
}

impl SetupPropsDeclaration<'_> {
    pub(crate) fn has_default(&self, name: &str) -> bool {
        self.defaults.iter().any(|default| default.name == name)
    }

    pub(crate) fn default_spans<'a>(&'a self, name: &'a str) -> impl Iterator<Item = Span> + 'a {
        self.defaults
            .iter()
            .filter(move |default| default.name == name)
            .map(|default| default.span)
    }
}

pub(crate) fn collect_setup_props<'a>(program: &'a Program<'a>) -> Vec<SetupPropsDeclaration<'a>> {
    if compiler_macro_is_shadowed(program, "defineProps") {
        return Vec::new();
    }

    let mut declarations = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(variable) => {
                for declarator in &variable.declarations {
                    let Some(init) = declarator.init.as_ref() else {
                        continue;
                    };
                    if let Some(declaration) =
                        declaration_from_expression(program, init, Some(&declarator.id))
                    {
                        declarations.push(declaration);
                    }
                }
            }
            Statement::ExpressionStatement(statement) => {
                if let Some(declaration) =
                    declaration_from_expression(program, &statement.expression, None)
                {
                    declarations.push(declaration);
                }
            }
            _ => {}
        }
    }
    declarations
}

fn declaration_from_expression<'a>(
    program: &'a Program<'a>,
    expression: &'a Expression<'a>,
    binding: Option<&'a BindingPattern<'a>>,
) -> Option<SetupPropsDeclaration<'a>> {
    let call = expression_call(expression)?;
    let (props, with_defaults) = if call_is_named(call, "withDefaults")
        && !compiler_macro_is_shadowed(program, "withDefaults")
    {
        let props = call
            .arguments
            .first()?
            .as_expression()
            .and_then(expression_call)?;
        call_is_named(props, "defineProps").then_some((props, Some(call)))?
    } else {
        call_is_named(call, "defineProps").then_some((call, None))?
    };

    let runtime = props
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
        .and_then(|expression| match expression {
            Expression::ObjectExpression(object) => Some(object.as_ref()),
            _ => None,
        });

    let mut typed = Vec::new();
    if let Some(arguments) = &props.type_arguments
        && let Some(ty) = arguments.params.first()
    {
        collect_type_props(ty, program, 0, &mut FxHashSet::default(), &mut typed);
    }

    let mut defaults = Vec::new();
    if let Some(wrapper) = with_defaults
        && let Some(Argument::ObjectExpression(object)) = wrapper.arguments.get(1)
    {
        collect_object_defaults(object, &mut defaults);
    }
    let uses_destructure = matches!(binding, Some(BindingPattern::ObjectPattern(_)));
    if let Some(BindingPattern::ObjectPattern(pattern)) = binding {
        for property in &pattern.properties {
            if property.computed {
                continue;
            }
            let Some(name) = property_key_name(&property.key) else {
                continue;
            };
            let BindingPattern::AssignmentPattern(assignment) = &property.value else {
                continue;
            };
            defaults.push(PropDefault {
                name: CompactString::new(name),
                span: assignment.right.span(),
            });
        }
    }

    Some(SetupPropsDeclaration {
        runtime,
        typed,
        defaults,
        uses_destructure,
        uses_with_defaults: with_defaults.is_some(),
    })
}

fn collect_object_defaults(object: &ObjectExpression<'_>, defaults: &mut Vec<PropDefault>) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = property_key_name(&property.key) else {
            continue;
        };
        defaults.push(PropDefault {
            name: CompactString::new(name),
            span: property.value.span(),
        });
    }
}

fn expression_call<'a>(expression: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_call(&parenthesized.expression)
        }
        Expression::TSAsExpression(as_expression) => expression_call(&as_expression.expression),
        Expression::TSSatisfiesExpression(satisfies) => expression_call(&satisfies.expression),
        Expression::TSNonNullExpression(non_null) => expression_call(&non_null.expression),
        _ => None,
    }
}

fn call_is_named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == name)
}

fn collect_type_props(
    ty: &TSType<'_>,
    program: &Program<'_>,
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    props: &mut Vec<TypedProp>,
) {
    if depth >= 16 {
        return;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => {
            collect_signatures(&literal.members, program, depth, seen, props)
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                collect_type_props(ty, program, depth + 1, seen, props);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => collect_type_props(
            &parenthesized.type_annotation,
            program,
            depth + 1,
            seen,
            props,
        ),
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return;
            };
            collect_named_type(identifier.name.as_str(), program, depth + 1, seen, props);
        }
        _ => {}
    }
}

fn collect_named_type(
    name: &str,
    program: &Program<'_>,
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    props: &mut Vec<TypedProp>,
) {
    let owned = CompactString::new(name);
    if depth >= 16 || !seen.insert(owned.clone()) {
        return;
    }
    for declaration in program.body.iter().filter_map(statement_declaration) {
        match declaration {
            Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                collect_type_props(&alias.type_annotation, program, depth + 1, seen, props);
            }
            Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                for heritage in &interface.extends {
                    if heritage.type_arguments.is_none()
                        && let Expression::Identifier(identifier) = &heritage.expression
                    {
                        collect_named_type(
                            identifier.name.as_str(),
                            program,
                            depth + 1,
                            seen,
                            props,
                        );
                    }
                }
                collect_signatures(&interface.body.body, program, depth, seen, props);
            }
            _ => {}
        }
    }
    seen.remove(&owned);
}

fn collect_signatures(
    members: &[TSSignature<'_>],
    program: &Program<'_>,
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    props: &mut Vec<TypedProp>,
) {
    for member in members {
        let TSSignature::TSPropertySignature(property) = member else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = property_key_name(&property.key) else {
            continue;
        };
        let boolean = property
            .type_annotation
            .as_ref()
            .is_some_and(|annotation| is_boolean_type(&annotation.type_annotation, program, depth));
        props.push(TypedProp {
            name: CompactString::new(name),
            span: property.span(),
            required: !property.optional,
            boolean,
        });
    }
    let _ = seen;
}

fn is_boolean_type(ty: &TSType<'_>, program: &Program<'_>, depth: usize) -> bool {
    if depth >= 16 {
        return false;
    }
    match ty {
        TSType::TSBooleanKeyword(_) => true,
        TSType::TSParenthesizedType(parenthesized) => {
            is_boolean_type(&parenthesized.type_annotation, program, depth + 1)
        }
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return false;
            };
            if identifier.name == "Boolean" {
                return true;
            }
            program
                .body
                .iter()
                .filter_map(statement_declaration)
                .find_map(|declaration| match declaration {
                    Declaration::TSTypeAliasDeclaration(alias)
                        if alias.id.name == identifier.name =>
                    {
                        Some(is_boolean_type(&alias.type_annotation, program, depth + 1))
                    }
                    _ => None,
                })
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn statement_declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}
