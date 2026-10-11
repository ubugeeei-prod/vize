//! Compiler-owned runtime validators in actual SFC script-setup blocks.
//!
//! This rule-specific resolver deliberately leaves the shared Options resolver
//! and standalone ScriptRule API unchanged. Both programs use the existing
//! script pass's parse; a malformed sibling never proves absence of shadowing.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, CallExpression, Class, Declaration,
    ExportDefaultDeclarationKind, Expression, Function, ImportDeclarationSpecifier,
    ImportOrExportKind, Program, Statement, TSGlobalDeclaration, TSModuleDeclaration,
    TSModuleDeclarationName, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;

use super::{ScriptLintResult, check_emits_object};

/// Ownership evidence for the normal script block, distinct from its absence.
#[derive(Clone, Copy)]
pub enum EmitsModuleScript<'a> {
    Absent,
    Unavailable,
    Parsed {
        program: &'a Program<'a>,
        offset: usize,
    },
}

const MACRO: &str = "defineEmits";

/// Inspect the already parsed setup AST before requesting sibling evidence.
pub(crate) fn has_candidate(program: &Program<'_>) -> bool {
    !shadowed(program)
        && program.body.iter().any(|statement| {
            for_each_call(statement, |call| {
                is_macro_call(call) && call.type_arguments.is_none() && !call.arguments.is_empty()
            })
        })
}

pub(super) fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    module: EmitsModuleScript<'a>,
    result: &mut ScriptLintResult,
) {
    if shadowed(program) {
        return;
    }
    match module {
        EmitsModuleScript::Unavailable => return,
        EmitsModuleScript::Parsed { program, .. } if shadowed(program) => return,
        _ => {}
    }
    for statement in &program.body {
        for_each_call(statement, |call| {
            if is_macro_call(call)
                && call.type_arguments.is_none()
                && let Some(argument) = call.arguments.first().and_then(|a| a.as_expression())
            {
                check_object(argument, program, offset, module, 0, result);
            }
            false
        });
    }
}

// Vue's compiler examines direct expression statements and non-ambient
// variable initializers, unwrapping parentheses and TypeScript assertions.
fn for_each_call<'a>(
    statement: &'a Statement<'a>,
    mut inspect: impl FnMut(&'a CallExpression<'a>) -> bool,
) -> bool {
    match statement {
        Statement::ExpressionStatement(statement) => {
            match statement.expression.get_inner_expression() {
                Expression::CallExpression(call) => inspect(call),
                _ => false,
            }
        }
        Statement::VariableDeclaration(declaration) if !declaration.declare => {
            declaration.declarations.iter().any(|declarator| {
                match declarator
                    .init
                    .as_ref()
                    .map(Expression::get_inner_expression)
                {
                    Some(Expression::CallExpression(call)) => inspect(call),
                    _ => false,
                }
            })
        }
        _ => false,
    }
}

fn is_macro_call(call: &CallExpression<'_>) -> bool {
    !call.optional && matches!(&call.callee, Expression::Identifier(id) if id.name == MACRO)
}

fn check_object<'a>(
    expression: &'a Expression<'a>,
    program: &'a Program<'a>,
    offset: usize,
    module: EmitsModuleScript<'a>,
    depth: usize,
    result: &mut ScriptLintResult,
) {
    // A fixed bound terminates cycles without allocating a name set. Resolution
    // never evaluates calls, follows mutable bindings, or imports dependencies.
    if depth >= 16 {
        return;
    }
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => check_emits_object(object, offset, result),
        Expression::Identifier(identifier) => {
            match binding(program, identifier.name.as_str()) {
                Binding::Constant(initializer) => {
                    check_object(initializer, program, offset, module, depth + 1, result);
                }
                Binding::Absent => {
                    if let EmitsModuleScript::Parsed { program, offset } = module
                        && let Binding::Constant(initializer) =
                            binding(program, identifier.name.as_str())
                    {
                        // A normal-script initializer resolves in its own
                        // module scope and retains its original SFC offset.
                        check_object(
                            initializer,
                            program,
                            offset,
                            EmitsModuleScript::Absent,
                            depth + 1,
                            result,
                        );
                    }
                }
                Binding::Other => {}
            }
        }
        _ => {}
    }
}

enum Binding<'a> {
    Absent,
    Constant(&'a Expression<'a>),
    Other,
}

fn binding<'a>(program: &'a Program<'a>, name: &str) -> Binding<'a> {
    for statement in &program.body {
        if let Some(declaration) = declaration(statement) {
            if let Declaration::VariableDeclaration(declaration) = declaration {
                if !declaration.declare {
                    for declarator in &declaration.declarations {
                        if binds(&declarator.id, name) {
                            return match (&declarator.id, &declarator.init) {
                                (BindingPattern::BindingIdentifier(_), Some(initializer))
                                    if declaration.kind == VariableDeclarationKind::Const =>
                                {
                                    Binding::Constant(initializer)
                                }
                                _ => Binding::Other,
                            };
                        }
                    }
                }
            } else if declaration_binds(declaration, name) {
                return Binding::Other;
            }
        }
        if import_binds(statement, name, false) || default_export_binds(statement, name) {
            return Binding::Other;
        }
    }
    if hoisted_var_binds(program, name) {
        Binding::Other
    } else {
        Binding::Absent
    }
}

fn shadowed(program: &Program<'_>) -> bool {
    program.body.iter().any(|statement| {
        declaration(statement).is_some_and(|d| declaration_binds(d, MACRO))
            || import_binds(statement, MACRO, true)
            || default_export_binds(statement, MACRO)
    }) || hoisted_var_binds(program, MACRO)
}

fn declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

fn declaration_binds(declaration: &Declaration<'_>, name: &str) -> bool {
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

fn default_export_binds(statement: &Statement<'_>, name: &str) -> bool {
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

fn import_binds(statement: &Statement<'_>, name: &str, allow_vue_macro: bool) -> bool {
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
                        && specifier.imported.name() == MACRO)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                specifier.local.name == name
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                specifier.local.name == name
            }
        })
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

fn hoisted_var_binds(program: &Program<'_>, name: &str) -> bool {
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
