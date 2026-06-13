//! Declared-emit resolution for `script/require-explicit-emits`.
//!
//! Resolves a component's declared event set from `defineEmits` (`<script
//! setup>`) or the Options API `emits` option, distinguishing a fully-known set
//! from one that cannot be enumerated (so the rule can stay sound). See the
//! parent module for the overall scope.

use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, BindingPattern, CallExpression, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
    TSCallSignatureDeclaration, TSLiteral, TSSignature, TSType,
};

use vize_carton::{CompactString, FxHashSet};

use super::super::emits_source::{EmitsDeclaration, resolve_emits_declaration};

/// The outcome of resolving a component's declared emits.
pub(super) enum Declared<'a> {
    /// The declared set is fully known. `binding` is the identifier the
    /// `defineEmits` return value was assigned to, if any (used to match
    /// `emit('x')` call sites in `<script setup>`).
    Known {
        names: FxHashSet<CompactString>,
        binding: Option<&'a str>,
    },
    /// No declaration was found, or one was found but cannot be fully resolved
    /// (spread / computed key / bare type reference). Either way, do not report.
    Unknown,
}

impl<'a> Declared<'a> {
    /// Map a collected name set (`None` = not fully enumerable) to a [`Declared`].
    fn from_names(names: Option<FxHashSet<CompactString>>, binding: Option<&'a str>) -> Self {
        match names {
            Some(names) => Declared::Known { names, binding },
            None => Declared::Unknown,
        }
    }
}

/// Resolve the declared emit names from `defineEmits` (`<script setup>`) or the
/// Options API `emits` option. Returns [`Declared::Unknown`] when nothing is
/// declared or when the declaration cannot be fully enumerated.
pub(super) fn resolve_declared_emits<'a>(program: &'a Program<'a>) -> Declared<'a> {
    if let Some(declaration) = find_define_emits(program) {
        return Declared::from_names(collect_call_emits(declaration.call), declaration.binding);
    }

    // Options API: `export default { emits: [...] | { ... } }`. The captured
    // binding is irrelevant here (events are emitted via `this.$emit`).
    match resolve_emits_declaration(program) {
        Some(EmitsDeclaration::Array(array)) => {
            Declared::from_names(collect_array_emits(array), None)
        }
        Some(EmitsDeclaration::Object(object)) => {
            Declared::from_names(collect_object_emits(object), None)
        }
        None => Declared::Unknown,
    }
}

/// A resolved `defineEmits(...)` call and the binding it was assigned to.
struct DefineEmitsDeclaration<'a> {
    binding: Option<&'a str>,
    call: &'a CallExpression<'a>,
}

/// Find the first top-level `defineEmits(...)` call, whether assigned to a
/// binding (`const emit = defineEmits(...)`) or bare (`defineEmits(...)`).
fn find_define_emits<'a>(program: &'a Program<'a>) -> Option<DefineEmitsDeclaration<'a>> {
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    let Some(call) = declarator.init.as_ref().and_then(unwrap_call) else {
                        continue;
                    };
                    if is_define_emits(call) {
                        let binding = match &declarator.id {
                            BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
                            _ => None,
                        };
                        return Some(DefineEmitsDeclaration { binding, call });
                    }
                }
            }
            Statement::ExpressionStatement(expression) => {
                if let Some(call) = unwrap_call(&expression.expression)
                    && is_define_emits(call)
                {
                    return Some(DefineEmitsDeclaration {
                        binding: None,
                        call,
                    });
                }
            }
            _ => {}
        }
    }
    None
}

/// Collect the declared events from a `defineEmits(...)` call: its type literal
/// (property / method / call signatures) or its runtime array/object argument.
/// `None` means the declaration cannot be fully enumerated (bare type reference,
/// spread, or computed/non-literal key), so the caller suppresses all reports.
fn collect_call_emits(call: &CallExpression<'_>) -> Option<FxHashSet<CompactString>> {
    if let Some(type_arguments) = &call.type_arguments {
        let mut names = FxHashSet::default();
        for type_param in &type_arguments.params {
            let TSType::TSTypeLiteral(literal) = type_param else {
                // `defineEmits<Emits>()` — the named type is not visible here.
                return None;
            };
            for member in &literal.members {
                // Index signatures etc. cannot be enumerated; treat as unknown.
                if !collect_event_from_signature(member, &mut names) {
                    return None;
                }
            }
        }
        return Some(names);
    }

    match call.arguments.first() {
        Some(Argument::ArrayExpression(array)) => collect_array_emits(array),
        Some(Argument::ObjectExpression(object)) => collect_object_emits(object),
        // No argument at all (`defineEmits()`): no events declared.
        None => Some(FxHashSet::default()),
        // Any other argument shape (identifier, spread, call, ...) is unknown.
        _ => None,
    }
}

/// Collect string-literal names from an array declaration (`['a', 'b']`).
/// Returns `None` if a spread or non-string element makes the set unknown.
fn collect_array_emits(array: &ArrayExpression<'_>) -> Option<FxHashSet<CompactString>> {
    let mut names = FxHashSet::default();
    for element in &array.elements {
        match element {
            ArrayExpressionElement::StringLiteral(literal) => {
                names.insert(CompactString::new(literal.value.as_str()));
            }
            // A hole (`[, 'a']`) declares nothing for that slot; skip it.
            ArrayExpressionElement::Elision(_) => {}
            // Spread or any non-string element: the set is no longer fully known.
            _ => return None,
        }
    }
    Some(names)
}

/// Collect names from an object declaration (`{ a: null, b: validator }`).
/// Returns `None` if a spread or computed key makes the set unknown.
fn collect_object_emits(object: &ObjectExpression<'_>) -> Option<FxHashSet<CompactString>> {
    let mut names = FxHashSet::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    return None;
                }
                match property_key_name(&property.key) {
                    Some(name) => {
                        names.insert(CompactString::new(name));
                    }
                    None => return None,
                }
            }
            // `{ ...others }`: the set is no longer fully known.
            ObjectPropertyKind::SpreadProperty(_) => return None,
        }
    }
    Some(names)
}

/// Extract an event name from a type-literal member. Returns `false` when the
/// member is a shape we cannot enumerate (an index signature, a non-literal
/// key/event), so the caller treats the whole declaration as unknown.
fn collect_event_from_signature(
    member: &TSSignature<'_>,
    names: &mut FxHashSet<CompactString>,
) -> bool {
    let name = match member {
        // `change: [id: number]`
        TSSignature::TSPropertySignature(property) => property_key_name(&property.key),
        // `change(id: number): void`
        TSSignature::TSMethodSignature(method) => property_key_name(&method.key),
        // `(e: 'change', id: number): void`
        TSSignature::TSCallSignatureDeclaration(signature) => call_signature_event(signature),
        // Index / construct signatures, etc.: unknown surface.
        _ => None,
    };
    match name {
        Some(name) => {
            names.insert(CompactString::new(name));
            true
        }
        None => false,
    }
}

/// The string-literal event name of a call-signature whose first parameter is a
/// `'event'` literal type, e.g. `(e: 'change', id: number): void`.
fn call_signature_event<'a>(signature: &'a TSCallSignatureDeclaration<'a>) -> Option<&'a str> {
    let first = signature.params.items.first()?;
    let annotation = first.type_annotation.as_ref()?;
    let TSType::TSLiteralType(literal_type) = &annotation.type_annotation else {
        return None;
    };
    match &literal_type.literal {
        TSLiteral::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

/// Whether the callee is the bare `defineEmits` compiler macro.
fn is_define_emits(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "defineEmits"
    )
}

fn unwrap_call<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => unwrap_call(&paren.expression),
        Expression::TSAsExpression(ts) => unwrap_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => unwrap_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => unwrap_call(&ts.expression),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}
