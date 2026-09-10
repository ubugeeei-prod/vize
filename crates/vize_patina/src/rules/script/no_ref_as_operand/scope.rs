use oxc_ast::ast::{BindingPattern, Expression, Statement, VariableDeclaration};
use vize_s0::{CompactString, FxHashMap};

/// The ref factories whose direct call result is a single ref object that must
/// be unwrapped with `.value`. Mirrors eslint-plugin-vue's tracked composables;
/// notably excludes `reactive`/`toRefs` (their results are not single refs).
const REF_FACTORIES: [&str; 5] = ["ref", "computed", "shallowRef", "toRef", "customRef"];

/// Collect the top-level bindings of a statement list into a fresh frame.
pub(super) fn collect_scope_bindings(
    statements: &[Statement<'_>],
) -> FxHashMap<CompactString, bool> {
    let mut frame = FxHashMap::default();
    merge_scope_bindings(statements, &mut frame);
    frame
}

/// Record every variable binding declared directly in `statements` into `frame`,
/// marking each as a ref (initializer is a ref factory call) or not. Nested
/// blocks/functions are not descended into: their bindings belong to their own
/// frame, pushed when the visitor enters them.
pub(super) fn merge_scope_bindings(
    statements: &[Statement<'_>],
    frame: &mut FxHashMap<CompactString, bool>,
) {
    for statement in statements {
        if let Statement::VariableDeclaration(declaration) = statement {
            record_declaration(declaration, frame);
        }
    }
}

/// Record the bindings of a single variable declaration. A plain
/// `const NAME = factory(...)` marks `NAME` as a ref; any other initializer (or
/// a destructuring pattern) marks the bound names as non-refs so they shadow.
pub(super) fn record_declaration(
    declaration: &VariableDeclaration<'_>,
    frame: &mut FxHashMap<CompactString, bool>,
) {
    for declarator in &declaration.declarations {
        let is_ref = matches!(&declarator.id, BindingPattern::BindingIdentifier(_))
            && declarator.init.as_ref().is_some_and(is_ref_factory_call);
        record_binding_names(&declarator.id, is_ref, frame);
    }
}

/// Insert the names bound by `pattern` into `frame` with the given ref flag.
/// Only a plain identifier can be a ref binding; destructured names are always
/// recorded as non-refs (so they correctly shadow an outer ref of that name).
pub(super) fn record_binding_names(
    pattern: &BindingPattern<'_>,
    is_ref: bool,
    frame: &mut FxHashMap<CompactString, bool>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            frame.insert(CompactString::from(id.name.as_str()), is_ref);
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                record_binding_names(&property.value, false, frame);
            }
            if let Some(rest) = &object.rest {
                record_binding_names(&rest.argument, false, frame);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                record_binding_names(element, false, frame);
            }
            if let Some(rest) = &array.rest {
                record_binding_names(&rest.argument, false, frame);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            record_binding_names(&assignment.left, false, frame);
        }
    }
}

/// Whether `expression` is a direct call to a known ref factory, e.g. `ref(0)`
/// or `computed(() => ...)`. Parenthesized and TS-cast wrappers are peeled.
fn is_ref_factory_call(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(id) if REF_FACTORIES.contains(&id.name.as_str()))
        }
        Expression::ParenthesizedExpression(paren) => is_ref_factory_call(&paren.expression),
        Expression::TSAsExpression(ts) => is_ref_factory_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => is_ref_factory_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => is_ref_factory_call(&ts.expression),
        _ => false,
    }
}
