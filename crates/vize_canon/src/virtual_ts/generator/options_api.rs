//! Options API template-binding emission for the virtual TypeScript generator.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_croquis::{BindingType, Croquis};

use super::options_api_support::is_safe_value_identifier;
use crate::virtual_ts::types::VirtualTsOptions;
use vize_carton::{FxHashSet, String, append};

// Emit declarations for Options API template bindings (`data`/`computed`/
// `methods`/`inject`/`setup`/`props`, plus legacy globals) when the caller
// enables Options API / legacy checking.
pub(super) fn generate_options_api_variables(
    mut ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    script: Option<&str>,
) {
    // The Options API bridge only runs for non-`<script setup>` components.
    // `<script setup>` already exposes its bindings (refs, props, setup
    // returns) in template scope via the normal generator, and a
    // `defineProps<Props>()` whose argument is a type reference (not an inline
    // `TSTypeLiteral`) still registers destructured names as
    // `BindingType::Props` without populating `summary.macros.props()`, which
    // would otherwise let those names slip through the filter below and
    // produce spurious `__VizeOptionsBinding` declarations.
    if summary.bindings.is_script_setup {
        return;
    }

    let macro_prop_names: FxHashSet<&str> = summary
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str())
        .collect();
    let configured_globals: FxHashSet<&str> = options
        .template_globals
        .iter()
        .map(|global| global.name.as_str())
        .collect();
    let mut names: Vec<(&str, bool)> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data => Some((name, true)),
                BindingType::Options | BindingType::VueGlobal => Some((name, false)),
                BindingType::Props if !macro_prop_names.contains(name) => Some((name, false)),
                _ => None,
            }
        })
        .filter(|(name, _)| !configured_globals.contains(name))
        .filter(|(name, _)| is_safe_value_identifier(name))
        .collect();
    names.sort_unstable_by_key(|(name, _)| *name);
    names.dedup_by(|left, right| left.0 == right.0);
    let inherited_unknown_names =
        unresolved_extends_template_names(summary, &configured_globals, script);

    if names.is_empty() && inherited_unknown_names.is_empty() {
        return;
    }

    ts.push_str("  // Options API template bindings\n");
    ts.push_str(
        "  type __VizeOptionsInstance<T> = T extends abstract new (...args: any) => infer I ? I : any;\n",
    );
    ts.push_str(
        "  type __VizeOptionsBinding<T, K extends string> = K extends keyof __VizeOptionsInstance<T> ? __VizeOptionsInstance<T>[K] : any;\n",
    );
    for (name, mutable) in &names {
        append!(
            ts,
            "  {} {name}: __VizeOptionsBinding<typeof __default__, \"{name}\"> = undefined as any;\n",
            if *mutable { "var" } else { "const" }
        );
    }
    if !inherited_unknown_names.is_empty() {
        ts.push_str("  // Unresolved imported Options API extends bindings\n");
        for name in &inherited_unknown_names {
            append!(ts, "  const {name}: any = undefined as any;\n");
        }
    }
    ts.push_str("  ");
    for (name, _) in &names {
        append!(ts, "void {name};");
    }
    for name in &inherited_unknown_names {
        append!(ts, "void {name};");
    }
    ts.push('\n');
}

fn unresolved_extends_template_names(
    summary: &Croquis,
    configured_globals: &FxHashSet<&str>,
    script: Option<&str>,
) -> Vec<String> {
    if !script.is_some_and(has_unresolved_extends) {
        return Vec::new();
    }

    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|export| export.name.as_str())
        .collect();
    let used_components: FxHashSet<&str> = summary
        .used_components
        .iter()
        .map(|component| component.as_str())
        .collect();
    let mut names = summary
        .undefined_refs
        .iter()
        .filter_map(|reference| {
            let name = reference.name.as_str();
            if summary.bindings.bindings.contains_key(name)
                || configured_globals.contains(name)
                || type_export_names.contains(name)
                || used_components.contains(name)
                || !is_safe_value_identifier(name)
            {
                return None;
            }
            Some(String::from(name))
        })
        .collect::<Vec<_>>();
    for expression in &summary.template_expressions {
        collect_unresolved_extends_expression_names(
            &mut names,
            expression.content.as_str(),
            summary,
            configured_globals,
            &type_export_names,
            &used_components,
        );
        if let Some(guard) = expression.vif_guard.as_ref() {
            collect_unresolved_extends_expression_names(
                &mut names,
                guard.as_str(),
                summary,
                configured_globals,
                &type_export_names,
                &used_components,
            );
        }
    }
    names.sort();
    names.dedup();
    names
}

fn has_unresolved_extends(script: &str) -> bool {
    if !script.contains("extends") || !script.contains("export default") {
        return false;
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return false;
    }

    let Some(options) = component_options_from_program(&parsed.program) else {
        return false;
    };
    let Some(extends) = option_expression_property(options, "extends") else {
        return false;
    };

    let object_bindings = collect_object_expression_bindings(&parsed.program);
    !is_resolved_options_target(extends, &object_bindings)
}

fn collect_object_expression_bindings<'a>(program: &'a Program<'a>) -> FxHashSet<&'a str> {
    let mut bindings = FxHashSet::default();
    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in declaration.declarations.iter() {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if object_expression_from_expression(init).is_some() {
                bindings.insert(id.name.as_str());
            }
        }
    }
    bindings
}

fn is_resolved_options_target<'a>(
    expression: &'a Expression<'a>,
    object_bindings: &FxHashSet<&'a str>,
) -> bool {
    match expression {
        Expression::ObjectExpression(_) => true,
        Expression::Identifier(identifier) => object_bindings.contains(identifier.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            is_resolved_options_target(&parenthesized.expression, object_bindings)
        }
        Expression::TSAsExpression(ts_as) => {
            is_resolved_options_target(&ts_as.expression, object_bindings)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_resolved_options_target(&ts_satisfies.expression, object_bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_resolved_options_target(&ts_non_null.expression, object_bindings)
        }
        _ => false,
    }
}

fn collect_unresolved_extends_expression_names(
    names: &mut Vec<String>,
    expression: &str,
    summary: &Croquis,
    configured_globals: &FxHashSet<&str>,
    type_export_names: &FxHashSet<&str>,
    used_components: &FxHashSet<&str>,
) {
    for identifier in vize_croquis::drawer::extract_identifiers_oxc(expression) {
        let name = identifier.as_str();
        if summary.bindings.bindings.contains_key(name)
            || configured_globals.contains(name)
            || type_export_names.contains(name)
            || used_components.contains(name)
            || !is_safe_value_identifier(name)
        {
            continue;
        }
        names.push(String::from(name));
    }
}

/// Byte offsets locating the rewriteable shape of a `<script>` default export.
///
/// All fields are offsets into the parsed `script`. A single default export is
/// at most one of these (an SFC module has one default export), so at most one
/// field is `Some`.
#[derive(Default, Clone, Copy)]
pub(super) struct DefaultExportTargets {
    /// A plain object-literal default export (`export default { ... }`) — the
    /// Options API shape — as `(export_start, object_start, object_end)`. Used
    /// to wrap the object in `defineComponent` so `this` in computed/methods
    /// gets Vue's instance typing. Anything else (already-wrapped
    /// `defineComponent({...})`, identifiers, calls, `as`/`satisfies`) stays
    /// `None` so only the bare options object is wrapped.
    pub object: Option<(usize, usize, usize)>,
    /// A class-declaration default export (`export default class Foo {}`, the
    /// class-component shape — vue-class-component / vue-property-decorator) as
    /// `(export_start, class_start, class_end, name_start, name_end)`.
    /// `export_start..class_start` is the `export default ` keyword (stripped);
    /// `class_start..class_end` is the class declaration; `name_start..name_end`
    /// is the class identifier. Decorators written before `export default` sit
    /// ahead of `export_start`; decorators after it fall inside the class span —
    /// so stripping only the keyword run keeps `@Component()` on a real class
    /// declaration either way (the line-based fallback would move it onto a
    /// `const`, which TypeScript rejects with TS1206). Anonymous default classes
    /// stay `None` (no name to alias by) and fall through to the generic
    /// `expr` rewrite below.
    pub class: Option<(usize, usize, usize, usize, usize)>,
    /// Any other default-export shape, rewritten to a bare
    /// `const __default__ = <expr>` at module scope, as
    /// `(export_start, expr_start, expr_end)`. Covers
    /// `export default defineComponent({...})`, identifiers, parenthesized /
    /// `as` / `satisfies` expressions, anonymous classes/functions, and
    /// `export default{` with no space — including multi-line / awkwardly
    /// formatted variants. `export_start..expr_start` is the `export default`
    /// keyword run that is dropped; `expr_start..expr_end` is the exported
    /// expression copied verbatim. This is the span-based replacement for the
    /// former line-scanning fallback, so it is only populated when neither
    /// `object` nor `class` applies.
    pub expr: Option<(usize, usize, usize)>,
}

/// Classify a `<script>` default export in a single parse. Parsing once keeps
/// the virtual-TS hot path free of a second full OXC parse per plain-`<script>`
/// component.
pub(super) fn find_default_export_targets(script: &str) -> DefaultExportTargets {
    let mut targets = DefaultExportTargets::default();
    if !script.contains("export default") {
        return targets;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return targets;
    }
    for statement in parsed.program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => {
                let object_span = object.span();
                targets.object = Some((
                    export.span.start as usize,
                    object_span.start as usize,
                    object_span.end as usize,
                ));
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) if class.id.is_some() => {
                let id = class.id.as_ref().expect("class id checked by guard");
                targets.class = Some((
                    export.span.start as usize,
                    class.span.start as usize,
                    class.span.end as usize,
                    id.span.start as usize,
                    id.span.end as usize,
                ));
            }
            // Every other default-export shape (already-wrapped
            // `defineComponent(...)`, identifiers, `as`/`satisfies`,
            // anonymous classes/functions, ...) is rewritten verbatim to a
            // bare `const __default__ = <expr>` using the declaration span.
            // Slicing on these AST offsets is correct regardless of source
            // formatting (`export default{` with no space, multi-line calls),
            // which the previous line scanner mishandled.
            other => {
                let declaration_span = other.span();
                targets.expr = Some((
                    export.span.start as usize,
                    declaration_span.start as usize,
                    declaration_span.end as usize,
                ));
            }
        }
        // A module has a single default export; stop at the first one.
        break;
    }
    targets
}

pub(super) fn option_expression_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        Some(&property.value)
    })
}

pub(super) fn component_options_from_program<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        component_options_from_export(&export.declaration)
    })
}

fn component_options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
        ExportDefaultDeclarationKind::CallExpression(call) => component_options_from_call(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::CallExpression(call) => component_options_from_call(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    let first = call.arguments.first()?;
    match first {
        Argument::ObjectExpression(object) => Some(object.as_ref()),
        Argument::CallExpression(call) => component_options_from_call(call),
        Argument::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Argument::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            matches!(callee.name.as_str(), "defineComponent" | "_defineComponent")
        }
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.property.name.as_str(),
                "defineComponent" | "_defineComponent"
            )
        }
        _ => false,
    }
}

pub(super) fn option_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        object_expression_from_expression(&property.value)
    })
}

fn object_expression_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_expression_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => object_expression_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            object_expression_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            object_expression_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

pub(super) fn source_slice(script: &str, span: oxc_span::Span) -> Option<&str> {
    script.get(span.start as usize..span.end as usize)
}

pub(super) fn safe_identifier(name: &str) -> String {
    let mut result = String::default();
    for (index, ch) in name.chars().enumerate() {
        if (index == 0 && (ch.is_ascii_alphabetic() || ch == '_' || ch == '$'))
            || (index > 0 && (ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
        {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() {
        result.push('_');
    }
    result
}
