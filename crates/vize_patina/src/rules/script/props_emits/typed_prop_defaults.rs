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
    pub(super) offset: usize,
}

#[derive(Clone)]
pub(crate) struct TypedProp {
    pub(crate) name: CompactString,
    pub(crate) span: Span,
    pub(crate) key_span: Span,
    pub(crate) required: bool,
    pub(crate) boolean: bool,
    /// Offset of the script block which owns this type member.
    pub(crate) offset: usize,
}

pub(crate) struct SetupPropsDeclaration<'a> {
    pub(crate) runtime: Option<&'a ObjectExpression<'a>>,
    pub(crate) typed: Vec<TypedProp>,
    pub(crate) defaults: Vec<PropDefault>,
    pub(crate) uses_destructure: bool,
    pub(crate) uses_with_defaults: bool,
    pub(crate) has_unknown_defaults: bool,
}

impl SetupPropsDeclaration<'_> {
    pub(crate) fn has_default(&self, name: &str) -> bool {
        self.defaults.iter().any(|default| default.name == name)
    }

    pub(crate) fn default_spans<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = (Span, usize)> + 'a {
        self.defaults
            .iter()
            .filter(move |default| default.name == name)
            .map(|default| (default.span, default.offset))
    }
}

pub(crate) fn collect_setup_props_with_types<'a>(
    program: &'a Program<'a>,
    offset: usize,
    sibling: Option<(&'a Program<'a>, usize)>,
) -> Vec<SetupPropsDeclaration<'a>> {
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
                    if let Some(declaration) = declaration_from_expression(
                        program,
                        offset,
                        sibling,
                        init,
                        Some(&declarator.id),
                    ) {
                        declarations.push(declaration);
                    }
                }
            }
            Statement::ExpressionStatement(statement) => {
                if let Some(declaration) = declaration_from_expression(
                    program,
                    offset,
                    sibling,
                    &statement.expression,
                    None,
                ) {
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
    offset: usize,
    sibling: Option<(&'a Program<'a>, usize)>,
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
        let sources = sibling.map_or_else(
            || vec![(program, offset)],
            |(sibling, sibling_offset)| vec![(program, offset), (sibling, sibling_offset)],
        );
        collect_type_props(
            ty,
            &sources,
            offset,
            0,
            &mut FxHashSet::default(),
            &mut typed,
        );
        let mut names = FxHashSet::default();
        typed.retain(|prop| names.insert(prop.name.clone()));
    }

    let mut defaults = Vec::new();
    let mut has_unknown_defaults = false;
    if let Some(wrapper) = with_defaults
        && let Some(Argument::ObjectExpression(object)) = wrapper.arguments.get(1)
    {
        let sources = sibling.map_or_else(
            || vec![(program, offset)],
            |(sibling, sibling_offset)| vec![(program, offset), (sibling, sibling_offset)],
        );
        has_unknown_defaults = collect_object_defaults(
            object,
            offset,
            &sources,
            &mut FxHashSet::default(),
            &mut defaults,
        );
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
                offset,
            });
        }
    }

    Some(SetupPropsDeclaration {
        runtime,
        typed,
        defaults,
        uses_destructure,
        uses_with_defaults: with_defaults.is_some(),
        has_unknown_defaults,
    })
}

fn collect_object_defaults(
    object: &ObjectExpression<'_>,
    offset: usize,
    sources: &[(&Program<'_>, usize)],
    seen: &mut FxHashSet<CompactString>,
    defaults: &mut Vec<PropDefault>,
) -> bool {
    let mut has_unknown_defaults = false;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                defaults.push(PropDefault {
                    name: CompactString::new(name),
                    span: property.value.span(),
                    offset,
                });
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                has_unknown_defaults |=
                    !collect_spread_defaults(&spread.argument, offset, sources, seen, defaults);
            }
        }
    }
    has_unknown_defaults
}

fn collect_spread_defaults(
    expression: &Expression<'_>,
    offset: usize,
    sources: &[(&Program<'_>, usize)],
    seen: &mut FxHashSet<CompactString>,
    defaults: &mut Vec<PropDefault>,
) -> bool {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => {
            !collect_object_defaults(object, offset, sources, seen, defaults)
        }
        Expression::Identifier(identifier) => {
            let name = CompactString::new(identifier.name.as_str());
            if !seen.insert(name.clone()) {
                return false;
            }
            let resolved = sources.iter().find_map(|(program, offset)| {
                program.body.iter().find_map(|statement| {
                    let Some(Declaration::VariableDeclaration(declaration)) =
                        statement_declaration(statement)
                    else {
                        return None;
                    };
                    if !declaration.kind.is_const() {
                        return None;
                    }
                    declaration.declarations.iter().find_map(|declarator| {
                        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                            return None;
                        };
                        if binding.name != identifier.name {
                            return None;
                        }
                        let Expression::ObjectExpression(object) =
                            declarator.init.as_ref()?.get_inner_expression()
                        else {
                            return None;
                        };
                        Some((object.as_ref(), *offset))
                    })
                })
            });
            let result = resolved.is_some_and(|(object, offset)| {
                !collect_object_defaults(object, offset, sources, seen, defaults)
            });
            seen.remove(&name);
            result
        }
        _ => false,
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
    sources: &[(&Program<'_>, usize)],
    origin_offset: usize,
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    props: &mut Vec<TypedProp>,
) {
    if depth >= 16 {
        return;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => {
            collect_signatures(&literal.members, sources, origin_offset, depth, seen, props)
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                collect_type_props(ty, sources, origin_offset, depth + 1, seen, props);
            }
        }
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_type_props(ty, sources, origin_offset, depth + 1, seen, props);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => collect_type_props(
            &parenthesized.type_annotation,
            sources,
            origin_offset,
            depth + 1,
            seen,
            props,
        ),
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return;
            };
            collect_named_type(identifier.name.as_str(), sources, depth + 1, seen, props);
        }
        _ => {}
    }
}

fn collect_named_type(
    name: &str,
    sources: &[(&Program<'_>, usize)],
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    props: &mut Vec<TypedProp>,
) {
    let owned = CompactString::new(name);
    if depth >= 16 || !seen.insert(owned.clone()) {
        return;
    }
    for (program, origin_offset) in sources {
        for declaration in program.body.iter().filter_map(statement_declaration) {
            match declaration {
                Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                    collect_type_props(
                        &alias.type_annotation,
                        sources,
                        *origin_offset,
                        depth + 1,
                        seen,
                        props,
                    );
                }
                Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                    for heritage in &interface.extends {
                        if let Expression::Identifier(identifier) = &heritage.expression {
                            collect_named_type(
                                identifier.name.as_str(),
                                sources,
                                depth + 1,
                                seen,
                                props,
                            );
                        }
                    }
                    collect_signatures(
                        &interface.body.body,
                        sources,
                        *origin_offset,
                        depth,
                        seen,
                        props,
                    );
                }
                _ => {}
            }
        }
    }
    seen.remove(&owned);
}

fn collect_signatures(
    members: &[TSSignature<'_>],
    sources: &[(&Program<'_>, usize)],
    origin_offset: usize,
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
            .is_some_and(|annotation| is_boolean_type(&annotation.type_annotation, sources, depth));
        props.push(TypedProp {
            name: CompactString::new(name),
            span: property.span(),
            key_span: property.key.span(),
            required: !property.optional,
            boolean,
            offset: origin_offset,
        });
    }
    let _ = seen;
}

fn is_boolean_type(ty: &TSType<'_>, sources: &[(&Program<'_>, usize)], depth: usize) -> bool {
    if depth >= 16 {
        return false;
    }
    match ty {
        TSType::TSBooleanKeyword(_) => true,
        TSType::TSLiteralType(literal) => {
            matches!(literal.literal, oxc_ast::ast::TSLiteral::BooleanLiteral(_))
        }
        TSType::TSParenthesizedType(parenthesized) => {
            is_boolean_type(&parenthesized.type_annotation, sources, depth + 1)
        }
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return false;
            };
            if identifier.name == "Boolean" {
                return true;
            }
            sources.iter().any(|(program, _)| {
                program
                    .body
                    .iter()
                    .filter_map(statement_declaration)
                    .any(|declaration| match declaration {
                        Declaration::TSTypeAliasDeclaration(alias)
                            if alias.id.name == identifier.name =>
                        {
                            is_boolean_type(&alias.type_annotation, sources, depth + 1)
                        }
                        _ => false,
                    })
            })
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
