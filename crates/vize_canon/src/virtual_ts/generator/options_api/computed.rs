//! Resolve the writability of local and inherited Options API computed members.

use oxc_ast::ast::{
    ArrayExpression, Declaration, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKind, Statement,
};
use vize_carton::{FxHashMap, FxHashSet, String};

use crate::virtual_ts::helpers::to_camel_case;

use super::{
    component_options_from_call, object_expression_from_expression, option_expression_property,
    option_object_property, property_key_name,
};

mod setters;

/// Names of `computed` members that declare a setter.
///
/// Vue exposes a `{ get, set }` computed (or a `get`/`set` accessor pair) as a
/// writable instance property, so a template assignment such as
/// `@input="ratio = $event"` is valid where a getter-only computed stays
/// read-only. Same-file `extends` / `mixins` objects contribute their members
/// with Vue's option precedence: a later source replaces an earlier one, and
/// the component's own declaration wins, so a local getter-only computed
/// shadows an inherited writable one.
pub(super) fn writable_computed_names<'a>(
    program: &'a Program<'a>,
    options: &'a ObjectExpression<'a>,
) -> FxHashSet<String> {
    let object_bindings = collect_object_expression_values(program);
    let mut seen = FxHashSet::default();
    let undefined_is_bound = setters::has_module_undefined_binding(program);
    let resolved =
        resolved_computed_writability(options, &object_bindings, &mut seen, undefined_is_bound);
    // Vue reads a template name from `data`, then `props`, and only then from
    // the context that exposes computed members, so a computed sharing a
    // prop's name never provides the value: the prop does, and it is
    // read-only. (`data` sharing the name is writable in its own right.)
    let props = resolved_prop_names(options, &object_bindings, &mut seen);
    resolved
        .into_iter()
        .filter_map(|(name, writable)| (writable && !props.contains(&name)).then_some(name))
        .collect()
}

/// Every prop name an options object resolves to through `extends`, `mixins`
/// and its own `props`, in array (`['a']`) or object (`{ a: ... }`) form.
fn resolved_prop_names<'a>(
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen: &mut FxHashSet<u32>,
) -> FxHashSet<String> {
    let mut props = FxHashSet::default();
    if !seen.insert(options.span.start) {
        return props;
    }
    for source in inherited_options_objects(options, object_bindings) {
        props.extend(resolved_prop_names(source, object_bindings, seen));
    }
    // A prop is declared in either case form and read by its camelCase name,
    // so `props: ['foo-bar']` shadows a computed `fooBar`; keep both spellings.
    let mut add_prop = |name: &str| {
        props.insert(String::from(name));
        props.insert(to_camel_case(name));
    };
    if let Some(expression) = option_expression_property(options, "props") {
        if let Some(names) = array_expression_from_expression(expression) {
            for element in names.elements.iter() {
                if let Some(Expression::StringLiteral(name)) = element.as_expression() {
                    add_prop(name.value.as_str());
                }
            }
        } else if let Some(object) = object_expression_from_expression(expression) {
            for property in object.properties.iter() {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed {
                    continue;
                }
                if let Some(name) = property_key_name(&property.key) {
                    add_prop(name);
                }
            }
        }
    }
    seen.remove(&options.span.start);
    props
}

/// The options objects an object inherits from, in Vue's merge order:
/// `extends` first, then each `mixins` entry.
fn inherited_options_objects<'a>(
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Vec<&'a ObjectExpression<'a>> {
    let mut sources = Vec::new();
    if let Some(extends) = option_expression_property(options, "extends")
        && let Some(target) = resolve_options_object(extends, object_bindings)
    {
        sources.push(target);
    }
    if let Some(mixins) =
        option_expression_property(options, "mixins").and_then(array_expression_from_expression)
    {
        for element in mixins.elements.iter() {
            // Spreads and holes are not options objects.
            let Some(expression) = element.as_expression() else {
                continue;
            };
            if let Some(target) = resolve_options_object(expression, object_bindings) {
                sources.push(target);
            }
        }
    }
    sources
}

/// The array an option holds, looking through parentheses and TypeScript
/// wrappers such as `(['a'] as const)`, like the object-resolution helpers.
fn array_expression_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ArrayExpression<'a>> {
    match expression {
        Expression::ArrayExpression(array) => Some(array.as_ref()),
        Expression::ParenthesizedExpression(parenthesized) => {
            array_expression_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => array_expression_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            array_expression_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            array_expression_from_expression(&ts_non_null.expression)
        }
        Expression::TSTypeAssertion(assertion) => {
            array_expression_from_expression(&assertion.expression)
        }
        _ => None,
    }
}

/// Every `computed` name an options object resolves to, with whether it is
/// writable, after applying `extends`, then each `mixins` entry in order, then
/// the object's own `computed`. Each later source replaces the entry of an
/// earlier one, which is how Vue merges the option.
fn resolved_computed_writability<'a>(
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen: &mut FxHashSet<u32>,
    undefined_is_bound: bool,
) -> FxHashMap<String, bool> {
    let mut resolved = FxHashMap::default();
    // `seen` holds the objects on the current resolution path: a cycle stops,
    // but the same mixin listed twice (or reached through two chains) is
    // applied at each position, as Vue does.
    if !seen.insert(options.span.start) {
        return resolved;
    }
    for source in inherited_options_objects(options, object_bindings) {
        resolved.extend(resolved_computed_writability(
            source,
            object_bindings,
            seen,
            undefined_is_bound,
        ));
    }
    for (name, writable) in local_computed_writability(options, undefined_is_bound) {
        resolved.insert(name, writable);
    }
    seen.remove(&options.span.start);
    resolved
}

/// The `computed` members an options object declares itself, with whether
/// each is writable. A `get`/`set` accessor pair declares the name twice, so
/// the entries are folded per name: any setter makes the name writable.
fn local_computed_writability<'a>(
    options: &'a ObjectExpression<'a>,
    undefined_is_bound: bool,
) -> Vec<(String, bool)> {
    let mut local: Vec<(String, bool)> = Vec::new();
    let Some(computed) = option_object_property(options, "computed") else {
        return local;
    };
    for property in computed.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = property_key_name(&property.key) else {
            continue;
        };
        let writable = match property.kind {
            PropertyKind::Set => true,
            PropertyKind::Get => false,
            PropertyKind::Init => object_expression_from_expression(&property.value)
                .and_then(|descriptor| option_expression_property(descriptor, "set"))
                .is_some_and(|value| setters::is_usable_setter(value, undefined_is_bound)),
        };
        match local.iter_mut().find(|(existing, _)| existing == name) {
            Some((_, existing_writable)) => *existing_writable |= writable,
            None => local.push((String::from(name), writable)),
        }
    }
    local
}

/// Module-scope `const name = { ... }` objects, exported or not, the
/// same-file targets a `mixins` / `extends` entry can name.
fn collect_object_expression_values<'a>(
    program: &'a Program<'a>,
) -> FxHashMap<&'a str, &'a ObjectExpression<'a>> {
    let mut bindings = FxHashMap::default();
    for statement in program.body.iter() {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration,
            Statement::ExportNamedDeclaration(export) => {
                let Some(Declaration::VariableDeclaration(declaration)) =
                    export.declaration.as_ref()
                else {
                    continue;
                };
                declaration
            }
            _ => continue,
        };
        for declarator in declaration.declarations.iter() {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let Some(object) = resolve_options_object(init, &FxHashMap::default()) {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }
    bindings
}

/// The options object an `extends` / `mixins` entry names: an inline object,
/// a same-file `const` bound to one, or a `defineComponent({ ... })` call
/// around one.
fn resolve_options_object<'a>(
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::Identifier(identifier) => {
            object_bindings.get(identifier.name.as_str()).copied()
        }
        Expression::CallExpression(call) => component_options_from_call(call),
        _ => object_expression_from_expression(expression),
    }
}
