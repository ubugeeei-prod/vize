//! Same-file names of a type-based `defineProps` call.
//!
//! `props_source` keeps the runtime object and array forms. This module reads
//! only the type argument, including a `withDefaults` wrapper, a local
//! interface or alias, an intersection, and a method signature. A type imported
//! from another file stays unresolved: eslint-plugin-vue reports that form
//! only when `parserOptions.project` supplies the other file.

use oxc_ast::ast::{
    CallExpression, Declaration, Expression, Program, Statement, TSSignature, TSType, TSTypeName,
};
use oxc_span::{GetSpan, Span};
use vize_l0::{CompactString, FxHashSet};

use super::compiler_macro_is_shadowed;
use super::props_source::property_key_name;

pub(crate) fn same_file_type_prop_names(program: &Program<'_>) -> Vec<(CompactString, Span)> {
    if compiler_macro_is_shadowed(program, "defineProps") {
        return Vec::new();
    }
    let mut names = Vec::new();
    for statement in &program.body {
        for call in statement_calls(statement) {
            let Some(props) = define_props_call(program, call) else {
                continue;
            };
            let Some(ty) = props
                .type_arguments
                .as_ref()
                .and_then(|arguments| arguments.params.first())
            else {
                continue;
            };
            collect_type(ty, program, 0, &mut FxHashSet::default(), &mut names);
        }
    }
    names
}

fn statement_calls<'a>(statement: &'a Statement<'a>) -> Vec<&'a CallExpression<'a>> {
    match statement {
        Statement::ExpressionStatement(expression) => expression_call(&expression.expression)
            .into_iter()
            .collect(),
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .filter_map(expression_call)
            .collect(),
        Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
            Some(Declaration::VariableDeclaration(declaration)) => declaration
                .declarations
                .iter()
                .filter_map(|declarator| declarator.init.as_ref())
                .filter_map(expression_call)
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn define_props_call<'a>(
    program: &Program<'a>,
    call: &'a CallExpression<'a>,
) -> Option<&'a CallExpression<'a>> {
    if call_is_named(call, "withDefaults") && !compiler_macro_is_shadowed(program, "withDefaults") {
        let inner = call
            .arguments
            .first()?
            .as_expression()
            .and_then(expression_call)?;
        return call_is_named(inner, "defineProps").then_some(inner);
    }
    call_is_named(call, "defineProps").then_some(call)
}

fn expression_call<'a>(expression: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}

fn call_is_named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == name)
}

fn collect_type(
    ty: &TSType<'_>,
    program: &Program<'_>,
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    names: &mut Vec<(CompactString, Span)>,
) {
    if depth >= 16 {
        return;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => collect_signatures(&literal.members, names),
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                collect_type(ty, program, depth + 1, seen, names);
            }
        }
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_type(ty, program, depth + 1, seen, names);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            collect_type(
                &parenthesized.type_annotation,
                program,
                depth + 1,
                seen,
                names,
            );
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return;
            };
            collect_named(identifier.name.as_str(), program, depth + 1, seen, names);
        }
        _ => {}
    }
}

fn collect_named(
    name: &str,
    program: &Program<'_>,
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    names: &mut Vec<(CompactString, Span)>,
) {
    if depth >= 16 || imported(program, name) {
        return;
    }
    let owned = CompactString::new(name);
    if !seen.insert(owned.clone()) {
        return;
    }
    for declaration in program.body.iter().filter_map(statement_declaration) {
        match declaration {
            Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                collect_type(&alias.type_annotation, program, depth + 1, seen, names);
            }
            Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                for heritage in &interface.extends {
                    if let Expression::Identifier(identifier) = &heritage.expression {
                        collect_named(identifier.name.as_str(), program, depth + 1, seen, names);
                    }
                }
                collect_signatures(&interface.body.body, names);
            }
            _ => {}
        }
    }
    seen.remove(&owned);
}

fn collect_signatures(members: &[TSSignature<'_>], names: &mut Vec<(CompactString, Span)>) {
    for member in members {
        let key = match member {
            TSSignature::TSPropertySignature(property) if !property.computed => &property.key,
            TSSignature::TSMethodSignature(method) if !method.computed => &method.key,
            _ => continue,
        };
        let Some(name) = property_key_name(key) else {
            continue;
        };
        names.push((CompactString::new(name), key.span()));
    }
}

fn imported(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        import
            .specifiers
            .iter()
            .flatten()
            .any(|specifier| specifier.local().name.as_str() == name)
    })
}

fn statement_declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}
