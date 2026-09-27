//! Statically absent computed setters and module-level `undefined` shadowing.

use oxc_ast::ast::{
    BindingPattern, Class, Declaration, ExportDefaultDeclarationKind, Expression, FunctionBody,
    ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement, TSModuleDeclaration,
    VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::Visit;

/// Vue invokes callable setters; erased TS/parenthesis wrappers do not make
/// a literal callable. Authored references and calls remain the checker's job.
pub(super) fn is_usable_setter(value: &Expression<'_>, undefined_is_bound: bool) -> bool {
    let value = value.get_inner_expression();
    !matches!(
        value,
        Expression::NullLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
    ) && !matches!(value, Expression::Identifier(identifier)
        if identifier.name == "undefined" && !undefined_is_bound)
}

/// Resolved options objects live in the script's module scope. Look only at
/// that scope, so an unrelated function's local `undefined` cannot shadow the
/// builtin used by a module-level descriptor. Reuse the existing parsed AST.
pub(super) fn has_module_undefined_binding(program: &Program<'_>) -> bool {
    let direct_binding = program.body.iter().any(|statement| match statement {
        Statement::ImportDeclaration(import) if import.import_kind != ImportOrExportKind::Type => {
            import.specifiers.as_ref().is_some_and(|specifiers| {
                specifiers.iter().any(|specifier| {
                    specifier.name() == "undefined"
                        && !matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(named)
                            if named.import_kind == ImportOrExportKind::Type)
                })
            })
        }
        Statement::ExportNamedDeclaration(export) => export
            .declaration
            .as_ref()
            .is_some_and(declaration_binds_undefined),
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
                .id
                .as_ref()
                .is_some_and(|id| id.name == "undefined"),
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                class.id.as_ref().is_some_and(|id| id.name == "undefined")
            }
            _ => false,
        },
        _ => statement
            .as_declaration()
            .is_some_and(declaration_binds_undefined),
    });
    if direct_binding {
        return true;
    }
    // A module's `var` declaration can be hoisted out of a block/loop. Ignore
    // function, class and namespace bodies, where the binding stays local.
    let mut hoisted = HoistedUndefined(false);
    hoisted.visit_program(program);
    hoisted.0
}

struct HoistedUndefined(bool);

impl<'a> Visit<'a> for HoistedUndefined {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            self.0 |= declaration
                .declarations
                .iter()
                .any(|declarator| pattern_binds_undefined(&declarator.id));
        }
    }

    fn visit_expression(&mut self, _: &Expression<'a>) {}
    fn visit_function_body(&mut self, _: &FunctionBody<'a>) {}
    fn visit_class(&mut self, _: &Class<'a>) {}
    fn visit_ts_module_declaration(&mut self, _: &TSModuleDeclaration<'a>) {}
}

fn declaration_binds_undefined(declaration: &Declaration<'_>) -> bool {
    match declaration {
        Declaration::VariableDeclaration(variable) => variable
            .declarations
            .iter()
            .any(|declarator| pattern_binds_undefined(&declarator.id)),
        Declaration::FunctionDeclaration(function) => function
            .id
            .as_ref()
            .is_some_and(|id| id.name == "undefined"),
        Declaration::ClassDeclaration(class) => {
            class.id.as_ref().is_some_and(|id| id.name == "undefined")
        }
        Declaration::TSEnumDeclaration(enumeration) => enumeration.id.name == "undefined",
        Declaration::TSImportEqualsDeclaration(import) => {
            import.import_kind != ImportOrExportKind::Type && import.id.name == "undefined"
        }
        _ => false,
    }
}

fn pattern_binds_undefined(pattern: &BindingPattern<'_>) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(id) => id.name == "undefined",
        BindingPattern::ObjectPattern(object) => {
            object
                .properties
                .iter()
                .any(|property| pattern_binds_undefined(&property.value))
                || object
                    .rest
                    .as_ref()
                    .is_some_and(|rest| pattern_binds_undefined(&rest.argument))
        }
        BindingPattern::ArrayPattern(array) => {
            array.elements.iter().flatten().any(pattern_binds_undefined)
                || array
                    .rest
                    .as_ref()
                    .is_some_and(|rest| pattern_binds_undefined(&rest.argument))
        }
        BindingPattern::AssignmentPattern(assignment) => pattern_binds_undefined(&assignment.left),
    }
}
