//! Setup constants that hold an SFC, such as `const Same = Child` or
//! `const Lazy = defineAsyncComponent(() => import("./Child.vue"))`.

use std::path::PathBuf;

use oxc_ast::ast::{
    Argument, BindingPattern, Expression, ImportDeclarationSpecifier, ObjectPropertyKind, Program,
    Statement, VariableDeclarationKind,
};
use oxc_span::SourceType;
use vize_l0::Allocator;

use super::{IdeContext, helpers, is_vue_path, resolve_component_file, resolve_exported_component};

const MAX_ALIAS_DEPTH: usize = 4;
/// Modules whose `defineAsyncComponent` returns the loaded component type.
/// An unimported call is a Nuxt or unplugin auto-import.
const VUE_MODULES: [&str; 4] = ["vue", "@vue/runtime-core", "@vue/runtime-dom", "#imports"];

pub(super) fn resolve(ctx: &IdeContext<'_>, name: &str, depth: usize) -> Option<PathBuf> {
    if depth >= MAX_ALIAS_DEPTH {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let script_setup = descriptor.script_setup.as_ref()?;
    let allocator = Allocator::default();
    let parsed = vize_croquis::script_parser::parse_program_for_analysis(
        &allocator,
        &script_setup.content,
        SourceType::ts(),
    );
    if parsed.panicked {
        return None;
    }
    match const_initializer(&parsed.program, name)?.without_parentheses() {
        Expression::Identifier(source) => {
            let source = source.name.as_str();
            resolve_component_file(ctx, source).or_else(|| resolve(ctx, source, depth + 1))
        }
        Expression::CallExpression(call) if async_component_call(&parsed.program, &call.callee) => {
            let specifier = loader_specifier(call.arguments.first()?)?;
            let resolved = helpers::resolve_import_path(ctx.uri, specifier)?;
            if is_vue_path(&resolved) {
                return Some(resolved);
            }
            resolve_exported_component(&resolved, "default", 0)
        }
        _ => None,
    }
}

/// An annotated constant has its declared type, not the component's.
fn const_initializer<'a>(program: &'a Program<'a>, name: &str) -> Option<&'a Expression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::VariableDeclaration(declaration) = statement else {
            return None;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            return None;
        }
        declaration.declarations.iter().find_map(|declarator| {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                return None;
            };
            (id.name == name && declarator.type_annotation.is_none())
                .then_some(declarator.init.as_ref())
                .flatten()
        })
    })
}

fn async_component_call(program: &Program<'_>, callee: &Expression<'_>) -> bool {
    let Expression::Identifier(callee) = callee.without_parentheses() else {
        return false;
    };
    callee.name == "defineAsyncComponent"
        && program.body.iter().all(|statement| {
            let Statement::ImportDeclaration(import) = statement else {
                return true;
            };
            VUE_MODULES.contains(&import.source.value.as_str())
                || import.specifiers.iter().flatten().all(|specifier| {
                    let local = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            &specifier.local
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            &specifier.local
                        }
                    };
                    local.name != callee.name
                })
        })
}

/// `() => import("./X.vue")`, alone or as the `loader` option.
fn loader_specifier<'a>(argument: &'a Argument<'a>) -> Option<&'a str> {
    match argument.as_expression()?.without_parentheses() {
        Expression::ObjectExpression(options) => options.properties.iter().find_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            if property.key.static_name()? != "loader" {
                return None;
            }
            import_specifier(&property.value)
        }),
        loader => import_specifier(loader),
    }
}

fn import_specifier<'a>(loader: &'a Expression<'a>) -> Option<&'a str> {
    let Expression::ArrowFunctionExpression(arrow) = loader.without_parentheses() else {
        return None;
    };
    let Expression::ImportExpression(import) = arrow.get_expression()?.without_parentheses() else {
        return None;
    };
    let Expression::StringLiteral(source) = &import.source else {
        return None;
    };
    Some(source.value.as_str())
}
