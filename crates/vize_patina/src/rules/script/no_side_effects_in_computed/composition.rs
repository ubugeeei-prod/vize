//! Vue computed identity and getter-local versus external binding ownership.

use super::{GetterBody, ScriptLintResult, SideEffectVisitor, accessor_object_getter};
use oxc_ast::ast::{
    AssignmentTarget, BindingPattern, CallExpression, Expression, IdentifierReference,
    ImportDeclaration, ImportDeclarationSpecifier, Program, SimpleAssignmentTarget, Statement,
    VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_variable_declaration},
};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum VueBinding {
    Computed,
    Namespace,
}

struct Bindings<'semantic> {
    identities: FxHashMap<SymbolId, VueBinding>,
    scoping: &'semantic Scoping,
}

pub(super) fn check_program<'a>(
    program: &'a Program<'a>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    // Most scripts have no Vue composition imports. Reuse the parsed AST and
    // build binding identities only for the imported Composition API surface.
    if !program.body.iter().any(|statement| {
        matches!(statement,
        Statement::ImportDeclaration(import) if is_vue_import(import))
    }) {
        return;
    }
    let semantic = SemanticBuilder::new().build(program).semantic;
    let mut bindings = Bindings {
        identities: FxHashMap::default(),
        scoping: semantic.scoping(),
    };
    for statement in &program.body {
        if let Statement::ImportDeclaration(import) = statement {
            bindings.visit_import_declaration(import);
        }
    }
    bindings.visit_program(program);
    ComputedVisitor {
        bindings: &bindings,
        scoping: semantic.scoping(),
        offset,
        result,
    }
    .visit_program(program);
}

fn is_vue_import(import: &ImportDeclaration<'_>) -> bool {
    !import.import_kind.is_type()
        && matches!(import.source.value.as_str(), "vue" | "@vue/composition-api")
}

impl<'a> Visit<'a> for Bindings<'_> {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        if !is_vue_import(import) {
            return;
        }
        for specifier in import.specifiers.iter().flatten() {
            let (local, kind) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if !named.import_kind.is_type() && named.imported.name() == "computed" =>
                {
                    (&named.local, VueBinding::Computed)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                    (&namespace.local, VueBinding::Namespace)
                }
                _ => continue,
            };
            if let Some(symbol) = local.symbol_id.get() {
                self.identities.insert(symbol, kind);
            }
        }
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Const {
            for declarator in &declaration.declarations {
                if let BindingPattern::BindingIdentifier(id) = &declarator.id
                    && let Some(symbol) = id.symbol_id.get()
                    && let Some(init) = &declarator.init
                    && let Some(identity) = self.identity(init)
                {
                    self.identities.insert(symbol, identity);
                }
            }
        }
        walk_variable_declaration(self, declaration);
    }
}

impl Bindings<'_> {
    fn identity(&self, expression: &Expression<'_>) -> Option<VueBinding> {
        match expression.get_inner_expression() {
            Expression::Identifier(id) => self.identities.get(&symbol(id, self.scoping)?).copied(),
            Expression::StaticMemberExpression(member) if member.property.name == "computed" => {
                (self.identity(&member.object) == Some(VueBinding::Namespace))
                    .then_some(VueBinding::Computed)
            }
            Expression::ComputedMemberExpression(member)
                if member.static_property_name().as_deref() == Some("computed") =>
            {
                (self.identity(&member.object) == Some(VueBinding::Namespace))
                    .then_some(VueBinding::Computed)
            }
            _ => None,
        }
    }
}

struct ComputedVisitor<'rule> {
    bindings: &'rule Bindings<'rule>,
    scoping: &'rule Scoping,
    offset: usize,
    result: &'rule mut ScriptLintResult,
}

impl<'a> Visit<'a> for ComputedVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.bindings.identity(&call.callee) == Some(VueBinding::Computed)
            && let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression())
            && let Some(getter) = getter(argument.get_inner_expression())
        {
            let (span, statements) = match getter {
                GetterBody::Function(function) => match &function.body {
                    Some(body) => (function.span, &body.statements),
                    None => return,
                },
                GetterBody::Arrow(arrow) => (arrow.span, &arrow.body.statements),
            };
            let mut visitor = SideEffectVisitor {
                offset: self.offset,
                result: self.result,
                scoping: Some(self.scoping),
                getter_span: span,
            };
            for statement in statements {
                visitor.visit_statement(statement);
            }
        }
        walk_call_expression(self, call);
    }
}

fn getter<'a>(expression: &'a Expression<'a>) -> Option<GetterBody<'a>> {
    match expression {
        Expression::FunctionExpression(function) => Some(GetterBody::Function(function)),
        Expression::ArrowFunctionExpression(arrow) => Some(GetterBody::Arrow(arrow)),
        Expression::ObjectExpression(object) => accessor_object_getter(object),
        _ => None,
    }
}

fn symbol(id: &IdentifierReference<'_>, scoping: &Scoping) -> Option<SymbolId> {
    scoping.get_reference(id.reference_id.get()?).symbol_id()
}

pub(super) fn root_is_external(
    expression: &Expression<'_>,
    scoping: Option<&Scoping>,
    getter: Span,
) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => identifier_is_external(id, scoping, getter),
        Expression::StaticMemberExpression(member) => {
            root_is_external(&member.object, scoping, getter)
        }
        Expression::ComputedMemberExpression(member) => {
            root_is_external(&member.object, scoping, getter)
        }
        Expression::PrivateFieldExpression(member) => {
            root_is_external(&member.object, scoping, getter)
        }
        _ => false, // fresh arrays, copies, calls and literal objects own their receiver
    }
}

fn identifier_is_external(
    id: &IdentifierReference<'_>,
    scoping: Option<&Scoping>,
    getter: Span,
) -> bool {
    let Some(scoping) = scoping else {
        return false;
    };
    let Some(symbol) = symbol(id, scoping) else {
        return false;
    };
    let declaration = scoping.symbol_span(symbol);
    declaration.start < getter.start || declaration.end > getter.end
}

pub(super) fn assignment_is_external(
    target: &AssignmentTarget<'_>,
    scoping: Option<&Scoping>,
    getter: Span,
) -> bool {
    target
        .as_simple_assignment_target()
        .is_some_and(|target| simple_target_is_external(target, scoping, getter))
}

pub(super) fn simple_target_is_external(
    target: &SimpleAssignmentTarget<'_>,
    scoping: Option<&Scoping>,
    getter: Span,
) -> bool {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            identifier_is_external(id, scoping, getter)
        }
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            root_is_external(&member.object, scoping, getter)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            root_is_external(&member.object, scoping, getter)
        }
        _ => false,
    }
}
