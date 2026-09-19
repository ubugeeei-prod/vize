//! Typed macro defaults. Unknown/imported type shapes are left to the checker.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Declaration, Expression, ImportDeclarationSpecifier,
    ObjectPropertyKind, Program, PropertyKey, Statement, TSSignature, TSType, TSTypeName,
};
use oxc_span::{GetSpan, Span};

use super::{LintDiagnostic, META, ScriptLintResult, property_key_name};

pub(super) fn check(program: &Program<'_>, offset: usize, result: &mut ScriptLintResult) {
    if shadowed(program, "defineProps") {
        return;
    }
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(Expression::CallExpression(call)) = &declarator.init else {
                continue;
            };
            if named(call, "defineProps") {
                let BindingPattern::ObjectPattern(pattern) = &declarator.id else {
                    continue;
                };
                for property in &pattern.properties {
                    if property.computed {
                        continue;
                    }
                    let BindingPattern::AssignmentPattern(default) = &property.value else {
                        continue;
                    };
                    check_default(
                        program,
                        call,
                        &property.key,
                        default.right.span(),
                        offset,
                        result,
                    );
                }
            } else if named(call, "withDefaults") && !shadowed(program, "withDefaults") {
                let (
                    Some(Argument::CallExpression(props)),
                    Some(Argument::ObjectExpression(defaults)),
                ) = (call.arguments.first(), call.arguments.get(1))
                else {
                    continue;
                };
                if !named(props, "defineProps") {
                    continue;
                }
                for property in &defaults.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        continue;
                    };
                    if !property.computed {
                        check_default(
                            program,
                            props,
                            &property.key,
                            property.value.span(),
                            offset,
                            result,
                        );
                    }
                }
            }
        }
    }
}

fn named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == name)
}

fn check_default(
    program: &Program<'_>,
    call: &CallExpression<'_>,
    key: &PropertyKey<'_>,
    default: Span,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let Some(name) = property_key_name(key) else {
        return;
    };
    let Some(ty) = call
        .type_arguments
        .as_ref()
        .and_then(|args| args.params.first())
    else {
        return;
    };
    let Some(required) = required_member(ty, program, name, 0) else {
        return;
    };
    let base = offset as u32;
    let key = key.span();
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            vize_s0::cstr!("Prop '{name}' is required but also declares a default"),
            base + key.start,
            base + key.end,
        )
        .with_label(
            "marked required here",
            base + required.start,
            base + required.end,
        )
        .with_label(
            "default declared here",
            base + default.start,
            base + default.end,
        )
        .with_help("Make the typed prop optional with `?` or remove the default."),
    );
}

fn required_member(
    ty: &TSType<'_>,
    program: &Program<'_>,
    name: &str,
    depth: usize,
) -> Option<Span> {
    if depth >= 16 {
        return None;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => required_signature(&literal.members, name),
        TSType::TSIntersectionType(intersection) => intersection
            .types
            .iter()
            .find_map(|ty| required_member(ty, program, name, depth + 1)),
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(id) = &reference.type_name else {
                return None;
            };
            program
                .body
                .iter()
                .filter_map(declaration)
                .find_map(|declaration| match declaration {
                    Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == id.name => {
                        required_member(&alias.type_annotation, program, name, depth + 1)
                    }
                    Declaration::TSInterfaceDeclaration(interface)
                        if interface.id.name == id.name =>
                    {
                        required_signature(&interface.body.body, name)
                    }
                    _ => None,
                })
        }
        _ => None,
    }
}

fn required_signature(members: &[TSSignature<'_>], name: &str) -> Option<Span> {
    members.iter().find_map(|member| {
        let TSSignature::TSPropertySignature(property) = member else {
            return None;
        };
        (!property.optional && !property.computed && property_key_name(&property.key) == Some(name))
            .then(|| property.key.span())
    })
}

// Only top-level macro calls are considered. A local value or a foreign import
// with the same spelling is an ordinary function, even before its declaration.
fn shadowed(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| {
        if let Some(declaration) = declaration(statement) {
            return match declaration {
                Declaration::VariableDeclaration(declaration) => declaration
                    .declarations
                    .iter()
                    .any(|declarator| binds(&declarator.id, name)),
                Declaration::FunctionDeclaration(function) => {
                    function.id.as_ref().is_some_and(|id| id.name == name)
                }
                Declaration::ClassDeclaration(class) => {
                    class.id.as_ref().is_some_and(|id| id.name == name)
                }
                Declaration::TSEnumDeclaration(enumeration) => enumeration.id.name == name,
                _ => false,
            };
        }
        match statement {
            Statement::ImportDeclaration(import) => {
                import
                    .specifiers
                    .iter()
                    .flatten()
                    .any(|specifier| match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                            specifier.local.name == name
                                && (import.source.value != "vue"
                                    || specifier.imported.name() != name)
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            specifier.local.name == name
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            specifier.local.name == name
                        }
                    })
            }
            _ => false,
        }
    })
}

fn declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

fn binds(pattern: &BindingPattern<'_>, name: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(id) => id.name == name,
        BindingPattern::AssignmentPattern(default) => binds(&default.left, name),
        BindingPattern::ObjectPattern(object) => {
            object.properties.iter().any(|p| binds(&p.value, name))
                || object
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binds(&rest.argument, name))
        }
        BindingPattern::ArrayPattern(array) => {
            array.elements.iter().flatten().any(|p| binds(p, name))
                || array
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binds(&rest.argument, name))
        }
    }
}
