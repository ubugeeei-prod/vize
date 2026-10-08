use oxc_ast::ast::{
    BindingPattern, Expression, ImportDeclarationSpecifier, Statement, VariableDeclaration,
};
use vize_l0::{CompactString, FxHashMap};

mod hoisted;
pub(super) use hoisted::collect_function_scope;

const REF_FACTORIES: [&str; 5] = ["ref", "computed", "shallowRef", "toRef", "customRef"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Binding {
    Other,
    Factory,
    Namespace,
    Ref,
}

pub(super) type Frame = FxHashMap<CompactString, Binding>;

pub(super) fn resolve(name: &str, local: &Frame, outer: &[Frame]) -> Binding {
    local
        .get(name)
        .copied()
        .or_else(|| {
            outer
                .iter()
                .rev()
                .find_map(|frame| frame.get(name).copied())
        })
        .unwrap_or(Binding::Other)
}

pub(super) fn collect_scope_bindings(statements: &[Statement<'_>], outer: &[Frame]) -> Frame {
    let mut frame = Frame::default();
    merge_scope_bindings(statements, &mut frame, outer);
    frame
}

/// Prime every local name before resolving initializers, so declarations and
/// parameters shadow imported factories throughout their lexical scope. Imports
/// are module bindings and are available independent of their textual position.
pub(super) fn merge_scope_bindings(
    statements: &[Statement<'_>],
    frame: &mut Frame,
    outer: &[Frame],
) {
    prime_scope_bindings(statements, frame);
    for statement in statements {
        if let Statement::VariableDeclaration(declaration) = statement {
            record_declaration(declaration, frame, outer);
        }
    }
}

pub(super) fn record_declaration(
    declaration: &VariableDeclaration<'_>,
    frame: &mut Frame,
    outer: &[Frame],
) {
    // Every loop-header binding shadows an imported factory before any
    // initializer executes, including a later declarator in the same header.
    for declarator in &declaration.declarations {
        record_binding_names(&declarator.id, Binding::Other, frame);
    }
    for declarator in &declaration.declarations {
        let binding = declarator
            .init
            .as_ref()
            .map(|init| expression_binding(init, frame, outer))
            .unwrap_or(Binding::Other);
        if binding == Binding::Namespace
            && let BindingPattern::ObjectPattern(object) = &declarator.id
        {
            for property in &object.properties {
                let kind = if property
                    .key
                    .static_name()
                    .is_some_and(|name| REF_FACTORIES.contains(&name.as_ref()))
                {
                    Binding::Factory
                } else {
                    Binding::Other
                };
                record_binding_names(&property.value, kind, frame);
            }
            if let Some(rest) = &object.rest {
                record_binding_names(&rest.argument, Binding::Other, frame);
            }
        } else {
            record_binding_names(&declarator.id, binding, frame);
        }
    }
}

pub(super) fn record_binding_names(
    pattern: &BindingPattern<'_>,
    binding: Binding,
    frame: &mut Frame,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            frame.insert(id.name.as_str().into(), binding);
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                record_binding_names(&property.value, Binding::Other, frame);
            }
            if let Some(rest) = &object.rest {
                record_binding_names(&rest.argument, Binding::Other, frame);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                record_binding_names(element, Binding::Other, frame);
            }
            if let Some(rest) = &array.rest {
                record_binding_names(&rest.argument, Binding::Other, frame);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            record_binding_names(&assignment.left, binding, frame)
        }
    }
}

fn expression_binding(expression: &Expression<'_>, local: &Frame, outer: &[Frame]) -> Binding {
    match expression {
        Expression::Identifier(id) => resolve(id.name.as_str(), local, outer),
        Expression::CallExpression(call) => {
            if expression_binding(&call.callee, local, outer) == Binding::Factory {
                Binding::Ref
            } else {
                Binding::Other
            }
        }
        Expression::StaticMemberExpression(member) => namespace_member(
            &member.object,
            Some(member.property.name.as_str()),
            local,
            outer,
        ),
        Expression::ComputedMemberExpression(member) => namespace_member(
            &member.object,
            member.static_property_name().as_deref(),
            local,
            outer,
        ),
        Expression::ParenthesizedExpression(paren) => {
            expression_binding(&paren.expression, local, outer)
        }
        Expression::TSAsExpression(ts) => expression_binding(&ts.expression, local, outer),
        Expression::TSSatisfiesExpression(ts) => expression_binding(&ts.expression, local, outer),
        Expression::TSNonNullExpression(ts) => expression_binding(&ts.expression, local, outer),
        _ => Binding::Other,
    }
}

fn namespace_member(
    object: &Expression<'_>,
    name: Option<&str>,
    local: &Frame,
    outer: &[Frame],
) -> Binding {
    if name.is_some_and(|name| REF_FACTORIES.contains(&name))
        && expression_binding(object, local, outer) == Binding::Namespace
    {
        Binding::Factory
    } else {
        Binding::Other
    }
}

pub(super) fn prime_scope_bindings(statements: &[Statement<'_>], frame: &mut Frame) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    record_binding_names(&declarator.id, Binding::Other, frame);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    frame.insert(id.name.as_str().into(), Binding::Other);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    frame.insert(id.name.as_str().into(), Binding::Other);
                }
            }
            Statement::ImportDeclaration(import) => {
                if let Some(specifiers) = &import.specifiers {
                    let vue = matches!(
                        import.source.value.as_str(),
                        "vue" | "@vue/composition-api" | "#imports"
                    );
                    for specifier in specifiers {
                        let (name, binding) = match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => (
                                s.local.name.as_str(),
                                if vue && REF_FACTORIES.contains(&s.imported.name().as_str()) {
                                    Binding::Factory
                                } else {
                                    Binding::Other
                                },
                            ),
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => (
                                s.local.name.as_str(),
                                if vue {
                                    Binding::Namespace
                                } else {
                                    Binding::Other
                                },
                            ),
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                (s.local.name.as_str(), Binding::Other)
                            }
                        };
                        frame.insert(name.into(), binding);
                    }
                }
            }
            _ => {}
        }
    }
}
