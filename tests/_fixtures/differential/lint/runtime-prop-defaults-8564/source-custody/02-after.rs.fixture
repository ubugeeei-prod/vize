//! Shared script bindings used to conservatively prove compiler-macro ownership.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Class, Declaration, ExportDefaultDeclarationKind,
    Expression, Function, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
    TSGlobalDeclaration, TSModuleDeclaration, TSModuleDeclarationName, VariableDeclaration,
    VariableDeclarationKind,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;

pub(super) fn shadowed(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| {
        declaration(statement).is_some_and(|d| declaration_binds(d, name))
            || import_binds(statement, name, true)
            || default_export_binds(statement, name)
    }) || hoisted_var_binds(program, name)
}

pub(super) fn declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

pub(super) fn declaration_binds(declaration: &Declaration<'_>, name: &str) -> bool {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            !declaration.declare && declaration.declarations.iter().any(|d| binds(&d.id, name))
        }
        Declaration::FunctionDeclaration(function) => {
            !function.declare && function.id.as_ref().is_some_and(|id| id.name == name)
        }
        Declaration::ClassDeclaration(class) => {
            !class.declare && class.id.as_ref().is_some_and(|id| id.name == name)
        }
        Declaration::TSEnumDeclaration(enumeration) => {
            !enumeration.declare && enumeration.id.name == name
        }
        Declaration::TSModuleDeclaration(namespace) => {
            !namespace.declare
                && matches!(&namespace.id, TSModuleDeclarationName::Identifier(id) if id.name == name)
        }
        Declaration::TSImportEqualsDeclaration(import) => {
            import.import_kind == ImportOrExportKind::Value && import.id.name == name
        }
        _ => false,
    }
}

pub(super) fn default_export_binds(statement: &Statement<'_>, name: &str) -> bool {
    let Statement::ExportDefaultDeclaration(export) = statement else {
        return false;
    };
    match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            !function.declare && function.id.as_ref().is_some_and(|id| id.name == name)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            !class.declare && class.id.as_ref().is_some_and(|id| id.name == name)
        }
        _ => false,
    }
}

pub(super) fn import_binds(statement: &Statement<'_>, name: &str, allow_vue_macro: bool) -> bool {
    let Statement::ImportDeclaration(import) = statement else {
        return false;
    };
    if import.import_kind == ImportOrExportKind::Type {
        return false;
    }
    import
        .specifiers
        .iter()
        .flatten()
        .any(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                specifier.import_kind == ImportOrExportKind::Value
                    && specifier.local.name == name
                    && !(allow_vue_macro
                        && import.source.value == "vue"
                        && specifier.imported.name() == name)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                specifier.local.name == name
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                specifier.local.name == name
            }
        })
}

pub(super) fn binds(pattern: &BindingPattern<'_>, name: &str) -> bool {
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

pub(super) fn hoisted_var_binds(program: &Program<'_>, name: &str) -> bool {
    let mut visitor = HoistedVar { name, found: false };
    for statement in &program.body {
        visitor.visit_statement(statement);
    }
    visitor.found
}

struct HoistedVar<'name> {
    name: &'name str,
    found: bool,
}

impl<'a> Visit<'a> for HoistedVar<'_> {
    fn visit_expression(&mut self, _: &Expression<'a>) {}
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &Class<'a>) {}
    fn visit_ts_module_declaration(&mut self, _: &TSModuleDeclaration<'a>) {}
    fn visit_ts_global_declaration(&mut self, _: &TSGlobalDeclaration<'a>) {}
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var && !declaration.declare {
            self.found |= declaration
                .declarations
                .iter()
                .any(|d| binds(&d.id, self.name));
        }
    }
}
