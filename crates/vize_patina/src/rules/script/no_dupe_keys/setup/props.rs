//! Local runtime/type prop names with lexical declaration ownership.

use super::NoDupeKeysModuleScript;
use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, CallExpression, Declaration, Expression, ObjectPropertyKind, Program,
        Statement, TSInterfaceDeclaration, TSSignature, TSType, TSTypeName,
    },
};
use oxc_semantic::Semantic;
use vize_l0::{CompactString, FxHashSet};

#[derive(Clone, Copy)]
struct Scope<'s, 'a> {
    program: &'a Program<'a>,
    semantic: &'s Semantic<'a>,
}

#[derive(Clone, Copy)]
struct Inputs<'s, 'a> {
    current: Scope<'s, 'a>,
    module: Option<Scope<'s, 'a>>,
}

pub(super) fn names<'s, 'a>(
    call: &'a CallExpression<'a>,
    setup: &'a Program<'a>,
    semantic: &'s Semantic<'a>,
    module: NoDupeKeysModuleScript<'a>,
    module_semantic: Option<&'s Semantic<'a>>,
) -> Vec<CompactString> {
    let inputs = Inputs {
        current: Scope {
            program: setup,
            semantic,
        },
        module: match (module, module_semantic) {
            (NoDupeKeysModuleScript::Parsed { program, .. }, Some(semantic)) => {
                Some(Scope { program, semantic })
            }
            _ => None,
        },
    };
    let mut names = Vec::new();
    if let Some(argument) = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
    {
        runtime_names(argument, inputs, 0, &mut names);
    } else if let Some(ty) = call
        .type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
    {
        type_names(ty, inputs, 0, &mut names);
    }
    // Inherited/merged/union type declarations describe one prop namespace.
    // An override of the same prop cannot create another colliding binding.
    let mut seen = FxHashSet::default();
    names.retain(|name| seen.insert(name.clone()));
    names
}

fn runtime_names<'s, 'a>(
    expression: &'a Expression<'a>,
    inputs: Inputs<'s, 'a>,
    depth: usize,
    names: &mut Vec<CompactString>,
) {
    if depth >= 16 {
        return;
    }
    match expression.get_inner_expression() {
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(Expression::StringLiteral(string)) = element.as_expression() {
                    names.push(CompactString::new(string.value.as_str()));
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        if let Some(name) = property.key.static_name() {
                            names.push(CompactString::new(name.as_ref()));
                        }
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        runtime_names(&spread.argument, inputs, depth + 1, names);
                    }
                }
            }
        }
        Expression::Identifier(identifier) => {
            match local_constant(inputs.current, identifier.name.as_str()) {
                Binding::Constant(expression) => {
                    runtime_names(expression, inputs, depth + 1, names)
                }
                Binding::Absent => {
                    if let Some(module) = inputs.module
                        && let Binding::Constant(expression) =
                            local_constant(module, identifier.name.as_str())
                    {
                        runtime_names(
                            expression,
                            Inputs {
                                current: module,
                                module: None,
                            },
                            depth + 1,
                            names,
                        );
                    }
                }
                Binding::Other => {}
            }
        }
        _ => {}
    }
}

fn type_names<'s, 'a>(
    ty: &'a TSType<'a>,
    inputs: Inputs<'s, 'a>,
    depth: usize,
    names: &mut Vec<CompactString>,
) {
    if depth >= 16 {
        return;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => signature_names(&literal.members, names),
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                type_names(ty, inputs, depth + 1, names);
            }
        }
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                type_names(ty, inputs, depth + 1, names);
            }
        }
        TSType::TSParenthesizedType(paren) => {
            type_names(&paren.type_annotation, inputs, depth + 1, names)
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return;
            };
            named_type(identifier.name.as_str(), inputs, depth + 1, names);
        }
        _ => {}
    }
}

fn named_type<'s, 'a>(
    name: &str,
    inputs: Inputs<'s, 'a>,
    depth: usize,
    names: &mut Vec<CompactString>,
) {
    if depth >= 16 {
        return;
    }
    match local_type(inputs.current.program, name) {
        LocalType::Alias(ty) => type_names(ty, inputs, depth + 1, names),
        LocalType::Interfaces(interfaces) => {
            for interface in interfaces {
                signature_names(&interface.body.body, names);
                for heritage in &interface.extends {
                    if let Expression::Identifier(identifier) = &heritage.expression {
                        named_type(identifier.name.as_str(), inputs, depth + 1, names);
                    }
                }
            }
        }
        LocalType::Absent => {
            if let Some(module) = inputs.module {
                named_type(
                    name,
                    Inputs {
                        current: module,
                        module: None,
                    },
                    depth + 1,
                    names,
                );
            }
        }
        LocalType::Other => {}
    }
}

fn signature_names(members: &[TSSignature<'_>], names: &mut Vec<CompactString>) {
    for member in members {
        let key = match member {
            TSSignature::TSPropertySignature(property) => &property.key,
            TSSignature::TSMethodSignature(method) => &method.key,
            _ => continue,
        };
        if let Some(name) = key.static_name() {
            names.push(CompactString::new(name.as_ref()));
        }
    }
}

enum LocalType<'a> {
    Absent,
    Other,
    Alias(&'a TSType<'a>),
    Interfaces(Vec<&'a TSInterfaceDeclaration<'a>>),
}

fn local_type<'a>(program: &'a Program<'a>, name: &str) -> LocalType<'a> {
    let mut interfaces = Vec::new();
    for statement in &program.body {
        if let Some(declaration) = declaration(statement) {
            match declaration {
                Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                    return LocalType::Alias(&alias.type_annotation);
                }
                Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                    interfaces.push(interface.as_ref())
                }
                Declaration::ClassDeclaration(class)
                    if class.id.as_ref().is_some_and(|id| id.name == name) =>
                {
                    return LocalType::Other;
                }
                Declaration::TSImportEqualsDeclaration(import) if import.id.name == name => {
                    return LocalType::Other;
                }
                Declaration::TSModuleDeclaration(module) if matches!(&module.id, oxc_ast::ast::TSModuleDeclarationName::Identifier(id) if id.name == name) =>
                {
                    return LocalType::Other;
                }
                _ => {}
            }
        }
        // A setup type import owns the name and cannot fall through to an
        // unrelated sibling declaration with the same spelling.
        if let Statement::ImportDeclaration(import) = statement
            && import
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| specifier.local().name == name)
        {
            return LocalType::Other;
        }
    }
    if interfaces.is_empty() {
        LocalType::Absent
    } else {
        LocalType::Interfaces(interfaces)
    }
}

enum Binding<'a> {
    Absent,
    Other,
    Constant(&'a Expression<'a>),
}

fn local_constant<'s, 'a>(scope: Scope<'s, 'a>, name: &str) -> Binding<'a> {
    let scoping = scope.semantic.scoping();
    let Some(symbol) = scoping.get_binding(scoping.root_scope_id(), name.into()) else {
        return Binding::Absent;
    };
    let flags = scoping.symbol_flags(symbol);
    if !flags.is_value() || flags.is_type_import() {
        return Binding::Absent;
    }
    if !flags.is_const_variable() || flags.is_ambient() {
        return Binding::Other;
    }
    let AstKind::VariableDeclarator(declaration) = scope.semantic.symbol_declaration(symbol).kind()
    else {
        return Binding::Other;
    };
    match (&declaration.id, &declaration.init) {
        (BindingPattern::BindingIdentifier(_), Some(initializer)) => Binding::Constant(initializer),
        _ => Binding::Other,
    }
}

fn declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}
