//! Options API template-binding emission for the virtual TypeScript generator.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ArrowFunctionExpression, CallExpression,
    ExportDefaultDeclarationKind, Expression, Function, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_croquis::{BindingType, Croquis};

use crate::virtual_ts::types::VirtualTsOptions;
use vize_carton::CompactString;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;

// Emit declarations for Options API template bindings
// (`data`/`computed`/`methods`/`inject`/`setup`/`props`, plus any Nuxt 2 globals
// the legacy path collected). Options API is officially supported in Vue 3, so
// this is part of the standard build and driven by a runtime opt-in — it costs
// nothing unless the caller enables Options API / legacy checking.
pub(super) fn generate_options_api_variables(
    mut ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
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
    let mut names: Vec<&str> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data | BindingType::Options | BindingType::VueGlobal => Some(name),
                BindingType::Props if !macro_prop_names.contains(name) => Some(name),
                _ => None,
            }
        })
        .filter(|name| !configured_globals.contains(name))
        .filter(|name| is_safe_value_identifier(name))
        .collect();
    names.sort_unstable();
    names.dedup();

    if names.is_empty() {
        return;
    }

    ts.push_str("  // Options API template bindings\n");
    ts.push_str(
        "  type __VizeOptionsInstance<T> = T extends abstract new (...args: any) => infer I ? I : any;\n",
    );
    ts.push_str(
        "  type __VizeOptionsBinding<T, K extends string> = K extends keyof __VizeOptionsInstance<T> ? __VizeOptionsInstance<T>[K] : any;\n",
    );
    for name in &names {
        append!(
            ts,
            "  const {name}: __VizeOptionsBinding<typeof __default__, \"{name}\"> = undefined as any;\n"
        );
    }
    ts.push_str("  ");
    for name in &names {
        append!(ts, "void {name};");
    }
    ts.push('\n');
}

pub(super) fn generate_options_api_bridge(mut ts: &mut String, summary: &Croquis, script: &str) {
    // Matches the gate in `generate_options_api_variables`: the typed-instance
    // bridge is only meaningful for non-`<script setup>` components.
    if summary.bindings.is_script_setup {
        return;
    }

    let Some(bridge) = collect_options_api_bridge(script) else {
        return;
    };

    let mut names: Vec<&str> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data | BindingType::Options | BindingType::Props => {
                    is_safe_value_identifier(name).then_some(name)
                }
                _ => None,
            }
        })
        .collect();
    names.sort_unstable();
    names.dedup();

    if names.is_empty()
        && bridge.computed.is_empty()
        && bridge.methods.is_empty()
        && bridge.mapped_types.is_empty()
    {
        return;
    }

    ts.push_str("  // Options API typed instance bridge\n");
    for (index, mapped_type) in bridge.mapped_types.iter().enumerate() {
        append!(
            ts,
            "  type __VizeOptionsMap{index} = {{ {mapped_type} }};\n"
        );
    }
    ts.push_str("  type __VizeThis = {\n");
    for name in names {
        append!(ts, "    {name}: any;\n");
    }
    ts.push_str("  }");
    for index in 0..bridge.mapped_types.len() {
        append!(ts, " & __VizeOptionsMap{index}");
    }
    ts.push_str(";\n");

    for function in &bridge.computed {
        emit_bridge_function(ts, "computed", function);
    }
    for function in &bridge.methods {
        emit_bridge_function(ts, "method", function);
    }

    if !bridge.computed.is_empty() || !bridge.methods.is_empty() {
        ts.push_str("  ");
        let mut first = true;
        for function in bridge.computed.iter().chain(bridge.methods.iter()) {
            if !first {
                ts.push(' ');
            }
            append!(
                ts,
                "void __vize_{}_{};",
                function.kind_prefix(),
                function.safe_name
            );
            first = false;
        }
        ts.push('\n');
    }
    ts.push('\n');
}

fn emit_bridge_function(mut ts: &mut String, kind: &str, function: &OptionsFunction) {
    let params = if function.params.is_empty() {
        String::from("this: __VizeThis")
    } else {
        let mut params = String::from("this: __VizeThis, ");
        params.push_str(&function.params);
        params
    };
    append!(
        ts,
        "  function __vize_{kind}_{}({params}) ",
        function.safe_name
    );
    ts.push_str(&function.body);
    ts.push('\n');
}

#[derive(Debug, Default)]
struct OptionsApiBridge {
    computed: Vec<OptionsFunction>,
    methods: Vec<OptionsFunction>,
    mapped_types: Vec<String>,
}

#[derive(Debug)]
struct OptionsFunction {
    kind: OptionsFunctionKind,
    safe_name: CompactString,
    params: String,
    body: String,
}

impl OptionsFunction {
    fn kind_prefix(&self) -> &'static str {
        match self.kind {
            OptionsFunctionKind::Computed => "computed",
            OptionsFunctionKind::Method => "method",
        }
    }
}

#[derive(Debug)]
enum OptionsFunctionKind {
    Computed,
    Method,
}

fn collect_options_api_bridge(script: &str) -> Option<OptionsApiBridge> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }

    let options = component_options_from_program(&parsed.program)?;
    let mut bridge = OptionsApiBridge::default();
    collect_function_bridge(
        script,
        options,
        "computed",
        OptionsFunctionKind::Computed,
        &mut bridge.computed,
        &mut bridge.mapped_types,
    );
    collect_function_bridge(
        script,
        options,
        "methods",
        OptionsFunctionKind::Method,
        &mut bridge.methods,
        &mut bridge.mapped_types,
    );
    Some(bridge)
}

fn collect_function_bridge(
    script: &str,
    options: &ObjectExpression<'_>,
    option_name: &str,
    kind: OptionsFunctionKind,
    output: &mut Vec<OptionsFunction>,
    mapped_types: &mut Vec<String>,
) {
    let Some(object) = option_object_property(options, option_name) else {
        return;
    };

    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                let Some(function) = options_function_from_expression(
                    script,
                    name,
                    &property.value,
                    match kind {
                        OptionsFunctionKind::Computed => OptionsFunctionKind::Computed,
                        OptionsFunctionKind::Method => OptionsFunctionKind::Method,
                    },
                ) else {
                    continue;
                };
                output.push(function);
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Expression::CallExpression(call) = &spread.argument {
                    collect_mapped_type(call, mapped_types);
                }
            }
        }
    }
}

fn options_function_from_expression(
    script: &str,
    name: &str,
    expression: &Expression<'_>,
    kind: OptionsFunctionKind,
) -> Option<OptionsFunction> {
    let (params, body) = match expression {
        Expression::FunctionExpression(function) => function_parts(script, function)?,
        Expression::ArrowFunctionExpression(arrow) => arrow_function_parts(script, arrow)?,
        Expression::ParenthesizedExpression(parenthesized) => {
            return options_function_from_expression(script, name, &parenthesized.expression, kind);
        }
        Expression::TSAsExpression(ts_as) => {
            return options_function_from_expression(script, name, &ts_as.expression, kind);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            return options_function_from_expression(script, name, &ts_satisfies.expression, kind);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            return options_function_from_expression(script, name, &ts_non_null.expression, kind);
        }
        _ => return None,
    };

    Some(OptionsFunction {
        kind,
        safe_name: CompactString::new(safe_identifier(name).as_str()),
        params,
        body,
    })
}

fn function_parts(script: &str, function: &Function<'_>) -> Option<(String, String)> {
    let params = params_source(script, &function.params)?;
    let body = function.body.as_ref()?;
    let body_source = source_slice(script, body.span())?;
    Some((params, String::from(body_source.trim())))
}

fn arrow_function_parts(
    script: &str,
    arrow: &ArrowFunctionExpression<'_>,
) -> Option<(String, String)> {
    let params = params_source(script, &arrow.params)?;
    let body_source = source_slice(script, arrow.body.span())?.trim();
    if arrow.expression {
        let mut body = String::from("{ return ");
        body.push_str(body_source.trim_end_matches(';'));
        body.push_str("; }");
        Some((params, body))
    } else {
        Some((params, String::from(body_source)))
    }
}

fn params_source(script: &str, params: &oxc_ast::ast::FormalParameters<'_>) -> Option<String> {
    let mut result = String::default();
    let mut first = true;
    for param in params.items.iter() {
        if !first {
            result.push_str(", ");
        }
        first = false;
        result.push_str(source_slice(script, param.span())?.trim());
    }
    if let Some(rest) = params.rest.as_ref() {
        if !first {
            result.push_str(", ");
        }
        result.push_str(source_slice(script, rest.span())?.trim());
    }
    Some(result)
}

fn collect_mapped_type(call: &CallExpression<'_>, mapped_types: &mut Vec<String>) {
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    if !matches!(
        callee.name.as_str(),
        "mapState" | "mapGetters" | "mapWritableState" | "mapActions"
    ) {
        return;
    }

    let Some(Argument::Identifier(store)) = call.arguments.first() else {
        return;
    };
    let Some(Argument::ArrayExpression(keys)) = call.arguments.get(1) else {
        return;
    };
    let keys: Vec<&str> = keys
        .elements
        .iter()
        .filter_map(|element| {
            let ArrayExpressionElement::StringLiteral(literal) = element else {
                return None;
            };
            Some(literal.value.as_str())
        })
        .collect();
    if keys.is_empty() {
        return;
    }

    let mut key_union = String::default();
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            key_union.push_str(" | ");
        }
        append!(key_union, "'{key}'");
    }

    let mut mapped_type = String::default();
    append!(
        mapped_type,
        "[K in {key_union}]: ReturnType<typeof {}>[K]",
        store.name.as_str()
    );
    mapped_types.push(mapped_type);
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

use crate::virtual_ts::props::OptionsApiPropsSource;

/// Parse a plain `<script>` and extract its Options API `props:` declaration.
///
/// Returns `None` when there is no resolvable component options object or no
/// `props:` option (or it is an unrecognized expression form). Object and array
/// literals are recognized directly; an identifier whose initializer object can
/// be resolved is not chased here because the runtime prop ctors would not be in
/// scope inside the emitted `__RuntimePropShape<...>` reference.
///
/// The result feeds canon's real `export type Props` for Options API
/// components: object form maps through the shared `__RuntimePropShape<...>`
/// machinery (runtime ctors and `{ type, required }` shapes resolve to TS prop
/// types with correct optionality), while the array form has no runtime type
/// info so each prop is emitted as optional `unknown`.
pub(super) fn find_options_api_props(script: &str) -> Option<OptionsApiPropsSource> {
    if !script.contains("export default") {
        return None;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }
    let options = component_options_from_program(&parsed.program)?;
    let props = option_expression_property(options, "props")?;
    options_api_props_from_expression(script, props)
}

fn options_api_props_from_expression(
    script: &str,
    expression: &Expression<'_>,
) -> Option<OptionsApiPropsSource> {
    match expression {
        Expression::ObjectExpression(object) => {
            let source = source_slice(script, object.span())?;
            Some(OptionsApiPropsSource::Object(String::from(source)))
        }
        Expression::ArrayExpression(array) => {
            let mut names = Vec::new();
            for element in &array.elements {
                if let ArrayExpressionElement::StringLiteral(literal) = element {
                    names.push(String::from(literal.value.as_str()));
                }
            }
            (!names.is_empty()).then_some(OptionsApiPropsSource::Names(names))
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            options_api_props_from_expression(script, &parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => {
            options_api_props_from_expression(script, &ts_as.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            options_api_props_from_expression(script, &ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            options_api_props_from_expression(script, &ts_non_null.expression)
        }
        _ => None,
    }
}

fn option_expression_property<'a>(
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

fn component_options_from_program<'a>(
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

fn option_object_property<'a>(
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

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn source_slice(script: &str, span: oxc_span::Span) -> Option<&str> {
    script.get(span.start as usize..span.end as usize)
}

fn safe_identifier(name: &str) -> String {
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

fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
