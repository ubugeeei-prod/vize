//! The absent-value oracle composition prunes with (Davinci FP-3).
//!
//! A child's `<button v-if="copy">` is not rendered at a usage that does not
//! pass `copy` when the value Vue gives an unpassed `copy` is falsy: no
//! default (`undefined`, or `false` for a `Boolean` prop) or a falsy literal
//! default. This reads the `<script setup>` block and answers, per template
//! identifier, whether that holds — only for forms it fully understands:
//!
//! - one `defineProps` — a type literal, a runtime object or array — alone,
//!   under `withDefaults(…, { key: literal })`, or destructured with literal
//!   defaults;
//! - the identifier is no other top-level binding of either script block
//!   (a setup binding of the same name would be what the template reads);
//! - a normal `<script>` has no `export default` (Options-API props).
//!
//! Anything else — a type reference, a spread, a computed key, a factory
//! default, a parse error — answers "not proven", so nothing is pruned.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, Declaration, Expression,
    ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind, Program, PropertyKey,
    PropertyKind, Statement, TSSignature, TSType, UnaryOperator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcDescriptor, SfcScriptBlock};
use vize_s0::{CompactString, FxHashMap, FxHashSet};

/// The template identifiers that name a prop whose unpassed value is proven
/// falsy; empty when anything is outside the understood forms.
pub(super) fn absent_falsy_props(descriptor: &SfcDescriptor<'_>) -> FxHashSet<CompactString> {
    scan(descriptor).unwrap_or_default()
}

fn scan(descriptor: &SfcDescriptor<'_>) -> Option<FxHashSet<CompactString>> {
    let setup = descriptor.script_setup.as_ref()?;
    let mut shadows = FxHashSet::default();
    if let Some(script) = descriptor.script.as_ref() {
        let allocator = Allocator::default();
        let program = parse(&allocator, script)?;
        for statement in &program.body {
            if matches!(statement, Statement::ExportDefaultDeclaration(_)) {
                return None;
            }
            declared(statement, &mut shadows);
        }
    }
    let allocator = Allocator::default();
    let mut props: Option<FxHashMap<CompactString, bool>> = None;
    let mut aliases = Vec::new();
    let program = parse(&allocator, setup)?;
    for statement in &program.body {
        let (call, pattern) = match statement {
            Statement::VariableDeclaration(declaration) => {
                let mut found = None;
                for declarator in &declaration.declarations {
                    match declarator.init.as_ref().and_then(props_macro) {
                        Some(call) if found.is_none() => found = Some((call, Some(&declarator.id))),
                        Some(_) => return None,
                        None => names(&declarator.id, &mut shadows),
                    }
                }
                let Some(found) = found else { continue };
                found
            }
            Statement::ExpressionStatement(statement) => match props_macro(&statement.expression) {
                Some(call) => (call, None),
                None => continue,
            },
            other => {
                declared(other, &mut shadows);
                continue;
            }
        };
        if props.is_some() {
            return None;
        }
        let mut declared = declared_props(call)?;
        match pattern {
            Some(BindingPattern::ObjectPattern(object)) => {
                for property in &object.properties {
                    let key = static_key(&property.key).filter(|_| !property.computed)?;
                    let (local, default) = match &property.value {
                        BindingPattern::BindingIdentifier(id) => (id.name.as_str(), None),
                        BindingPattern::AssignmentPattern(assign) => match &assign.left {
                            BindingPattern::BindingIdentifier(id) => {
                                (id.name.as_str(), Some(&assign.right))
                            }
                            _ => return None,
                        },
                        _ => return None,
                    };
                    let falsy = declared.get_mut(key)?;
                    if let Some(default) = default {
                        *falsy &= is_falsy_literal(default);
                    }
                    if local != key {
                        aliases.push((CompactString::new(local), CompactString::new(key)));
                    }
                }
                if let Some(rest) = &object.rest {
                    names(&rest.argument, &mut shadows);
                }
            }
            Some(other) => names(other, &mut shadows),
            None => {}
        }
        props = Some(declared);
    }
    let props = props?;
    if shadows.contains("undefined") {
        return None;
    }
    let aliased = aliases
        .into_iter()
        .filter_map(|(local, key)| props.get(&key).map(|falsy| (local, *falsy)));
    let all = props.iter().map(|(key, falsy)| (key.clone(), *falsy));
    Some(
        all.chain(aliased)
            .filter(|(name, falsy)| *falsy && !shadows.contains(name))
            .map(|(name, _)| name)
            .collect(),
    )
}

fn parse<'a>(allocator: &'a Allocator, block: &SfcScriptBlock<'_>) -> Option<Program<'a>> {
    let source_type = match block.lang.as_deref() {
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::mjs(),
    };
    let source = allocator.alloc_str(block.content.as_ref());
    let parsed = Parser::new(allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    Some(parsed.program)
}

/// `defineProps(…)`, or `withDefaults(defineProps(…), …)`.
fn props_macro<'a>(expression: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    match call.callee_name()? {
        "defineProps" | "withDefaults" => Some(call),
        _ => None,
    }
}

/// Each declared prop with whether its unpassed value is falsy.
fn declared_props(call: &CallExpression<'_>) -> Option<FxHashMap<CompactString, bool>> {
    let (define, defaults) = match call.callee_name()? {
        "withDefaults" => {
            let [Argument::CallExpression(define), defaults] = call.arguments.as_slice() else {
                return None;
            };
            (define.callee_name() == Some("defineProps")).then_some(())?;
            (&**define, Some(defaults.as_expression()?))
        }
        _ => (call, None),
    };
    let mut props = FxHashMap::default();
    match (define.type_arguments.as_ref(), define.arguments.as_slice()) {
        (Some(types), []) => {
            let [TSType::TSTypeLiteral(literal)] = types.params.as_slice() else {
                return None;
            };
            for member in &literal.members {
                let TSSignature::TSPropertySignature(signature) = member else {
                    return None;
                };
                let key = static_key(&signature.key).filter(|_| !signature.computed)?;
                props.insert(CompactString::new(key), true);
            }
        }
        (None, [Argument::ObjectExpression(object)]) => {
            for (key, value) in properties(object)? {
                let falsy = match value {
                    Expression::ObjectExpression(options) => properties(options)?
                        .find(|(key, _)| *key == "default")
                        .is_none_or(|(_, default)| is_falsy_literal(default)),
                    _ => true,
                };
                props.insert(CompactString::new(key), falsy);
            }
        }
        (None, [Argument::ArrayExpression(array)]) => {
            for element in &array.elements {
                let ArrayExpressionElement::StringLiteral(name) = element else {
                    return None;
                };
                props.insert(CompactString::new(name.value.as_str()), true);
            }
        }
        (None, []) => {}
        _ => return None,
    }
    if let Some(defaults) = defaults {
        let Expression::ObjectExpression(object) = defaults.without_parentheses() else {
            return None;
        };
        for (key, value) in properties(object)? {
            if let Some(falsy) = props.get_mut(key) {
                *falsy &= is_falsy_literal(value);
            }
        }
    }
    // Vue camelizes kebab-case keys; those are left unproven.
    props.retain(|key, _| !key.contains('-'));
    Some(props)
}

/// The `key: value` properties of an object literal; `None` on a spread, a
/// computed key or an accessor.
fn properties<'a>(
    object: &'a ObjectExpression<'a>,
) -> Option<impl Iterator<Item = (&'a str, &'a Expression<'a>)>> {
    let mut out = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property.kind != PropertyKind::Init {
            return None;
        }
        out.push((static_key(&property.key)?, &property.value));
    }
    Some(out.into_iter())
}

fn static_key<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

/// `null`, `undefined`, `void 0`, `false`, `0`, `''`, `` `` ``.
fn is_falsy_literal(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses().get_inner_expression() {
        Expression::NullLiteral(_) => true,
        Expression::BooleanLiteral(literal) => !literal.value,
        Expression::NumericLiteral(literal) => literal.value == 0.0,
        Expression::StringLiteral(literal) => literal.value.is_empty(),
        Expression::TemplateLiteral(literal) => {
            literal.expressions.is_empty()
                && literal
                    .quasis
                    .iter()
                    .all(|quasi| quasi.value.raw.is_empty())
        }
        Expression::Identifier(id) => id.name == "undefined",
        Expression::UnaryExpression(unary) => {
            unary.operator == UnaryOperator::Void
                && matches!(unary.argument, Expression::NumericLiteral(_))
        }
        _ => false,
    }
}

/// Top-level bindings a statement declares.
fn declared(statement: &Statement<'_>, shadows: &mut FxHashSet<CompactString>) {
    match statement {
        Statement::ImportDeclaration(import) => {
            for specifier in import.specifiers.iter().flatten() {
                let local = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(s) => &s.local,
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => &s.local,
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => &s.local,
                };
                shadows.insert(CompactString::new(local.name.as_str()));
            }
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                declaration_names(declaration, shadows);
            }
        }
        other => {
            if let Some(declaration) = other.as_declaration() {
                declaration_names(declaration, shadows);
            }
        }
    }
}

fn declaration_names(declaration: &Declaration<'_>, shadows: &mut FxHashSet<CompactString>) {
    let id = match declaration {
        Declaration::VariableDeclaration(variables) => {
            for declarator in &variables.declarations {
                names(&declarator.id, shadows);
            }
            return;
        }
        Declaration::FunctionDeclaration(function) => function.id.as_ref(),
        Declaration::ClassDeclaration(class) => class.id.as_ref(),
        Declaration::TSEnumDeclaration(enumeration) => Some(&enumeration.id),
        _ => None,
    };
    if let Some(id) = id {
        shadows.insert(CompactString::new(id.name.as_str()));
    }
}

fn names(pattern: &BindingPattern<'_>, shadows: &mut FxHashSet<CompactString>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            shadows.insert(CompactString::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                names(&property.value, shadows);
            }
            if let Some(rest) = &object.rest {
                names(&rest.argument, shadows);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                names(element, shadows);
            }
            if let Some(rest) = &array.rest {
                names(&rest.argument, shadows);
            }
        }
        BindingPattern::AssignmentPattern(assign) => names(&assign.left, shadows),
    }
}
