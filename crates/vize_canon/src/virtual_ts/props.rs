mod setup_scoped;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_croquis::BindingType;
use vize_croquis::Croquis;
use vize_croquis::macros::{MacroKind, ModelDefinition};

use super::helpers::{is_reserved_identifier, to_safe_identifier};
use setup_scoped::props_type_ref;
pub(crate) use setup_scoped::{PropsTypeEmission, generate_setup_scoped_props_artifact};

#[inline]
fn should_skip_template_prop_binding(summary: &Croquis, prop_name: &str) -> bool {
    if summary
        .macros
        .props_destructure()
        .and_then(|destructure| destructure.get(prop_name))
        .is_some()
    {
        return true;
    }

    summary
        .bindings
        .get(prop_name)
        .is_some_and(|binding_type| !matches!(binding_type, BindingType::Props))
}

fn emit_template_prop_binding(
    ts: &mut String,
    props_type_ref: &str,
    prop_name: &str,
    has_default: bool,
) {
    let binding_name = to_safe_identifier(prop_name);
    if has_default {
        append!(
            *ts,
            "  const {binding_name} = props[\"{prop_name}\"] as Exclude<{props_type_ref}[\"{prop_name}\"], undefined>;\n"
        );
    } else {
        append!(*ts, "  const {binding_name} = props[\"{prop_name}\"];\n");
    }
    append!(*ts, "  void {binding_name};\n");
}

fn emit_keyed_template_prop_binding(
    ts: &mut String,
    props_type_ref: &str,
    prop_name: &str,
    has_default: bool,
) {
    let binding_name = to_safe_identifier(prop_name);
    if has_default {
        append!(
            *ts,
            "  const {binding_name} = props[(\"{prop_name}\" satisfies keyof {props_type_ref})] as Exclude<{props_type_ref}[\"{prop_name}\"], undefined>;\n"
        );
    } else {
        append!(
            *ts,
            "  const {binding_name} = props[(\"{prop_name}\" satisfies keyof {props_type_ref})];\n"
        );
    }
    append!(*ts, "  void {binding_name};\n");
}

fn can_emit_keyed_template_prop_binding(prop_name: &str) -> bool {
    let mut chars = prop_name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        && !prop_name.starts_with('$')
        && !is_reserved_identifier(prop_name)
}

fn collect_keyed_template_prop_names(
    summary: &Croquis,
    emitted_names: &FxHashSet<String>,
) -> Vec<String> {
    let mut names = FxHashSet::default();
    for undef in &summary.undefined_refs {
        let name = undef.name.as_str();
        if emitted_names.contains(name)
            || should_skip_template_prop_binding(summary, name)
            || !can_emit_keyed_template_prop_binding(name)
        {
            continue;
        }
        names.insert(name.into());
    }

    let mut names: Vec<String> = names.into_iter().collect();
    names.sort_unstable();
    names
}

fn should_emit_keyed_template_prop_bindings(
    summary: &Croquis,
    type_name: &str,
    emitted_names: &FxHashSet<String>,
) -> bool {
    if has_top_level_type_operator(type_name) {
        return true;
    }
    if is_plain_inline_type_literal(type_name) {
        return false;
    }

    let base_name = strip_generic_params(type_name).trim();
    if let Some(body) = summary.types.definitions().resolve(base_name) {
        return has_top_level_type_operator(body.as_str())
            || (emitted_names.is_empty() && !is_plain_inline_type_literal(body.as_str()));
    }

    emitted_names.is_empty() && !summary.types.definitions().is_defined(base_name)
}

fn is_plain_inline_type_literal(type_name: &str) -> bool {
    let type_name = type_name.trim();
    if !type_name.starts_with('{') {
        return false;
    }

    let mut depth = 0i32;
    for (idx, c) in type_name.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return type_name[idx + c.len_utf8()..].trim().is_empty();
                }
            }
            _ => {}
        }
    }
    false
}

fn has_top_level_type_operator(type_name: &str) -> bool {
    let mut angle_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;

    for c in type_name.chars() {
        match c {
            '<' => angle_depth += 1,
            '>' => angle_depth -= 1,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '&' | '|'
                if angle_depth == 0
                    && brace_depth == 0
                    && paren_depth == 0
                    && bracket_depth == 0 =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn collect_with_defaults_default_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for call in summary.macros.all_calls() {
        if call.kind != MacroKind::WithDefaults {
            continue;
        }
        let Some(runtime_args) = &call.runtime_args else {
            continue;
        };
        collect_with_defaults_default_names_from_source(runtime_args.as_str(), &mut names);
    }
    names
}

fn collect_with_defaults_default_names_from_source(source: &str, names: &mut FxHashSet<String>) {
    let source = source.trim();
    let expression_source = if source.starts_with("withDefaults") {
        String::from(source)
    } else {
        cstr!("withDefaults({source})")
    };

    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let Ok(Expression::CallExpression(call)) =
        Parser::new(&allocator, expression_source.as_str(), source_type).parse_expression()
    else {
        return;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    if callee.name.as_str() != "withDefaults" {
        return;
    }
    let Some(Argument::ObjectExpression(defaults)) = call.arguments.get(1) else {
        return;
    };

    for prop in &defaults.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if prop.computed {
            continue;
        }
        let Some(name) = property_key_name(&prop.key) else {
            continue;
        };
        names.insert(name.into());
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

fn template_props_type_ref(
    base_type_ref: &str,
    defaulted_prop_names: &FxHashSet<String>,
) -> String {
    if defaulted_prop_names.is_empty() {
        return base_type_ref.into();
    }

    let mut names: Vec<&str> = defaulted_prop_names
        .iter()
        .map(|name| name.as_str())
        .collect();
    names.sort_unstable();

    let mut default_keys = String::default();
    for name in names {
        if !default_keys.is_empty() {
            default_keys.push_str(" | ");
        }
        append!(default_keys, "\"{name}\"");
    }

    cstr!("__WithDefaultsResult<{base_type_ref}, Pick<{base_type_ref}, {default_keys}>>")
}

fn model_prop_type(model: &ModelDefinition) -> &str {
    model.model_type.as_deref().unwrap_or("unknown")
}

fn emit_model_prop_member(ts: &mut String, model: &ModelDefinition) {
    let optional = if model.required { "" } else { "?" };
    let name = model.name.as_str();
    let prop_type = model_prop_type(model);
    append!(*ts, "  \"{name}\"{optional}: {prop_type};\n");
}

fn append_model_props_type_literal(ts: &mut String, models: &[ModelDefinition]) {
    ts.push_str("{\n");
    for model in models {
        emit_model_prop_member(ts, model);
    }
    ts.push('}');
}

/// Source of an Options API `props:` option, used to emit a real `export type
/// Props` for plain `<script>` (Options API) components. `Object` is a raw
/// runtime props object literal (`{ initial: Number, ... }`) fed to
/// `__RuntimePropShape<...>`; `Names` is the array form (`['a', 'b']`) which
/// carries no runtime type info and is emitted as optional `unknown` members.
pub(crate) enum OptionsApiPropsSource {
    Object(String),
    Names(Vec<String>),
}

/// Generate Props type definition at module level.
/// When `generic_param` is present (e.g., `"T extends Foo, P extends Bar"`),
/// the Props type is emitted with generic parameters: `export type Props<T, P> = ...;`
///
/// `options_api_props` carries the Options API runtime `props:` declaration when
/// the component is a plain `<script>` Options API component with no
/// `defineProps` macro. It lets cross-file prop checking see real prop types
/// instead of the historical `export type Props = {}` no-op.
pub(crate) fn generate_props_type(
    ts: &mut String,
    summary: &Croquis,
    generic_param: Option<&str>,
    options_api_props: Option<&OptionsApiPropsSource>,
    emission: PropsTypeEmission,
) {
    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let models = summary.macros.models();
    let has_models = !models.is_empty();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref());
    let props_already_defined = summary
        .type_exports
        .iter()
        .any(|te| te.name.as_str() == "Props");

    // Build generic suffix for Props type declaration (with `= any` defaults).
    // `const` modifiers are illegal on type-alias parameters (TS1277).
    let generic_decl = generic_param
        .map(|g| {
            let with_defaults = strip_const_modifiers(&add_generic_defaults(g));
            cstr!("<{with_defaults}>")
        })
        .unwrap_or_default();

    ts.push_str("// ========== Exported Types ==========\n");

    if emission == PropsTypeEmission::DeferredToSetup && define_props_type_args.is_some() {
    } else if props_already_defined {
        // User defined Props, no need to re-export
    } else if let Some(type_args) = define_props_type_args {
        let inner_type = type_args
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(type_args.as_str());
        // Always emit Props alias so it's available in template and default export.
        if has_models {
            append!(*ts, "export type Props{generic_decl} = {inner_type} & ");
            append_model_props_type_literal(ts, models);
            ts.push_str(";\n");
        } else {
            append!(*ts, "export type Props{generic_decl} = {inner_type};\n");
        }
    } else if has_props || has_models {
        append!(*ts, "export type Props{generic_decl} = {{\n");
        let mut emitted_names: FxHashSet<String> = FxHashSet::default();
        for prop in props {
            let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
            let optional = if prop.required { "" } else { "?" };
            append!(*ts, "  {}{optional}: {prop_type};\n", prop.name);
            emitted_names.insert(prop.name.as_str().into());
        }
        for model in models {
            if emitted_names.contains(model.name.as_str()) {
                continue;
            }
            emit_model_prop_member(ts, model);
        }
        ts.push_str("};\n");
    } else if let Some(options_api_props) = options_api_props {
        emit_options_api_props_type(ts, &generic_decl, options_api_props);
    } else {
        append!(*ts, "export type Props{generic_decl} = {{}};\n");
    }

    ts.push('\n');
}

/// Emit a real `export type Props` for an Options API component, derived from
/// its runtime `props:` option. The object form reuses the shared
/// `__RuntimePropShape<...>` mapped type (the same machinery `defineProps`
/// runtime forms use), so runtime ctors and `{ type, required }` shapes resolve
/// to real prop types with correct optionality.
fn emit_options_api_props_type(
    ts: &mut String,
    generic_decl: &str,
    options_api_props: &OptionsApiPropsSource,
) {
    match options_api_props {
        OptionsApiPropsSource::Object(source) => {
            append!(
                *ts,
                "export type Props{generic_decl} = __RuntimePropShape<{source}>;\n"
            );
        }
        OptionsApiPropsSource::Names(names) => {
            append!(*ts, "export type Props{generic_decl} = {{\n");
            for name in names {
                append!(*ts, "  \"{name}\"?: unknown;\n");
            }
            ts.push_str("};\n");
        }
    }
}

/// Generate props variables inside template closure.
/// When `generic_param` is present, uses `Props<T, P>` instead of `Props`.
pub(crate) fn generate_props_variables(
    ts: &mut String,
    summary: &Croquis,
    generic_param: Option<&str>,
    props_type_ref_override: Option<&str>,
) {
    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let models = summary.macros.models();
    let has_models = !models.is_empty();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref());

    let props_type_ref = props_type_ref(generic_param, props_type_ref_override);
    let mut defaulted_prop_names = collect_with_defaults_default_names(summary);
    for model in models {
        if model.default_value.is_some() {
            defaulted_prop_names.insert(model.name.as_str().into());
        }
    }
    let template_props_type_ref =
        template_props_type_ref(props_type_ref.as_str(), &defaulted_prop_names);

    if has_props || define_props_type_args.is_some() || has_models {
        ts.push_str("  // Props are available in template as variables\n");
        ts.push_str("  // Access via `propName` or `props.propName`\n");
        append!(
            *ts,
            "  const props: {template_props_type_ref} = {{}} as {template_props_type_ref};\n"
        );
        ts.push_str("  void props; // Mark as used to avoid TS6133\n");

        let mut emitted_names = FxHashSet::default();
        if has_props {
            // Runtime-declared props: generate individual variables
            for prop in props {
                if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                    continue;
                }
                emit_template_prop_binding(
                    ts,
                    template_props_type_ref.as_str(),
                    prop.name.as_str(),
                    prop.default_value.is_some() || defaulted_prop_names.contains(&prop.name),
                );
                emitted_names.insert(prop.name.as_str().into());
            }
        } else if let Some(type_args) = define_props_type_args {
            // Type-only defineProps<TypeName>(): extract fields
            // type_args may include angle brackets (e.g., "<Props>", "<Foo<T>>"), strip outer pair
            let type_name = strip_outer_angle_brackets(type_args.trim());

            // Resolve the named type's fields through the croquis TypeResolver,
            // which the script analysis populates from the OXC AST (local
            // interfaces/type literals included). This handles inline object
            // types, registered local types, nested braces, generics, and
            // comments — no raw-text scanning.
            let type_properties = summary
                .types
                .extract_properties(type_reference_lookup_key(type_name));
            for prop in &type_properties {
                if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                    continue;
                }
                emit_template_prop_binding(
                    ts,
                    template_props_type_ref.as_str(),
                    prop.name.as_str(),
                    defaulted_prop_names.contains(&prop.name),
                );
                emitted_names.insert(prop.name.as_str().into());
            }

            if should_emit_keyed_template_prop_bindings(summary, type_name, &emitted_names) {
                for name in collect_keyed_template_prop_names(summary, &emitted_names) {
                    emit_keyed_template_prop_binding(
                        ts,
                        template_props_type_ref.as_str(),
                        name.as_str(),
                        defaulted_prop_names.contains(&name),
                    );
                }
            }
        }
        for model in models {
            if emitted_names.contains(model.name.as_str())
                || should_skip_template_prop_binding(summary, model.name.as_str())
            {
                continue;
            }
            emit_template_prop_binding(
                ts,
                template_props_type_ref.as_str(),
                model.name.as_str(),
                model.default_value.is_some(),
            );
            emitted_names.insert(model.name.as_str().into());
        }
        ts.push('\n');
    }
}

pub(crate) fn collect_template_prop_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let props = summary.macros.props();
    if !props.is_empty() {
        for prop in props {
            if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                continue;
            }
            if !is_reserved_identifier(prop.name.as_str()) {
                continue;
            }
            names.insert(prop.name.as_str().into());
        }
        return names;
    }

    for model in summary.macros.models() {
        if should_skip_template_prop_binding(summary, model.name.as_str()) {
            continue;
        }
        if !is_reserved_identifier(model.name.as_str()) {
            continue;
        }
        names.insert(model.name.as_str().into());
    }

    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
    else {
        return names;
    };
    let type_name = strip_outer_angle_brackets(type_args.trim());
    let type_properties = summary
        .types
        .extract_properties(type_reference_lookup_key(type_name));
    for prop in &type_properties {
        if should_skip_template_prop_binding(summary, prop.name.as_str()) {
            continue;
        }
        if !is_reserved_identifier(prop.name.as_str()) {
            continue;
        }
        names.insert(prop.name.as_str().into());
    }
    names
}

/// Lookup key for a `defineProps<...>` type argument when resolving its fields
/// through the croquis `TypeResolver`.
///
/// Inline object types (`{ msg: string }`) are passed through verbatim — the
/// resolver parses them directly. A type *reference* may carry a generic
/// instantiation (`Foo<T>`); the resolver registers local types under their
/// bare declaration name, so strip the arguments to recover `Foo`.
fn type_reference_lookup_key(type_name: &str) -> &str {
    if type_name.trim_start().starts_with('{') {
        type_name
    } else {
        strip_generic_params(type_name).trim()
    }
}

/// Strip the outermost `<...>` pair from a type_args string, handling nested generics.
/// e.g., `"<Props>"` → `"Props"`, `"<Foo<T>>"` → `"Foo<T>"`, `"Props"` → `"Props"`
fn strip_outer_angle_brackets(s: &str) -> &str {
    let s = s.trim();
    if !s.starts_with('<') {
        return s;
    }
    // Find the matching '>' for the opening '<'
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 && i == s.len() - 1 {
                    // The opening '<' matches the final '>' — strip them
                    return &s[1..i];
                }
            }
            _ => {}
        }
    }
    s
}

/// Strip generic parameters from a type name for interface lookup.
/// e.g., `"ContextMenuContentProps<T>"` → `"ContextMenuContentProps"`
fn strip_generic_params(type_name: &str) -> &str {
    match type_name.find('<') {
        Some(pos) => &type_name[..pos],
        None => type_name,
    }
}

/// First identifier of a generic parameter declaration, skipping the TS 5.0
/// `const` modifier: `const T extends Tab` declares `T`, not `const`.
fn generic_param_name(param: &str) -> &str {
    let mut tokens = param.split_whitespace();
    match tokens.next() {
        Some("const") => tokens.next().unwrap_or(param),
        Some(token) => token,
        None => param,
    }
}

/// Extract just the generic parameter names from a full generic declaration.
/// e.g., `"T extends Foo, P extends Bar"` → `"T, P"`
/// e.g., `"T"` → `"T"`
/// e.g., `"T extends Record<string, any>, U"` → `"T, U"`
/// e.g., `"const T extends Tab"` → `"T"`
pub(crate) fn extract_generic_names(generic_param: &str) -> String {
    let mut names = String::default();
    let mut depth = 0i32; // track <> nesting
    let mut current_name = String::default();
    let mut in_extends = false;

    for ch in generic_param.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let trimmed = current_name.trim();
                if !trimmed.is_empty() {
                    // Extract just the name (before "extends")
                    let name = generic_param_name(trimmed);
                    if !names.is_empty() {
                        names.push_str(", ");
                    }
                    names.push_str(name);
                }
                current_name = String::default();
                in_extends = false;
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            current_name.push(ch);
        }
    }

    // Handle the last parameter
    let trimmed = current_name.trim();
    if !trimmed.is_empty() {
        let name = generic_param_name(trimmed);
        if !names.is_empty() {
            names.push_str(", ");
        }
        names.push_str(name);
    }

    let _ = in_extends;
    names
}

/// Drop TS 5.0 `const` modifiers from a generic parameter list.
/// The modifier is only legal on function/method/class type parameters
/// (TS1277), so callers that splice parameters into `type`/`interface`
/// declarations must strip it first.
/// e.g., `"const T extends Tab = any"` → `"T extends Tab = any"`
pub(crate) fn strip_const_modifiers(generic_param: &str) -> String {
    let mut result = String::default();
    let mut depth = 0i32;
    let mut current_param = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                append_param_without_const(&mut result, current_param.trim());
                result.push_str(", ");
                current_param = String::default();
                continue;
            }
            _ => {}
        }
        current_param.push(ch);
    }

    let trimmed = current_param.trim();
    if !trimmed.is_empty() {
        append_param_without_const(&mut result, trimmed);
    }

    result
}

/// Append a single generic parameter with its leading `const` modifier removed.
fn append_param_without_const(result: &mut String, param: &str) {
    let stripped = param
        .strip_prefix("const")
        .filter(|rest| rest.starts_with(|ch: char| ch.is_ascii_whitespace()))
        .map(str::trim_start)
        .unwrap_or(param);
    result.push_str(stripped);
}

/// Add `= any` defaults to each generic parameter that doesn't already have a default.
/// e.g., `"T extends Foo, P"` → `"T extends Foo = any, P = any"`
/// e.g., `"T = string"` → `"T = string"` (unchanged, already has default)
pub(crate) fn add_generic_defaults(generic_param: &str) -> String {
    let mut result = String::default();
    let mut depth = 0i32;
    let mut current_param = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => {
                depth += 1;
                current_param.push(ch);
            }
            '>' => {
                depth -= 1;
                current_param.push(ch);
            }
            ',' if depth == 0 => {
                append_param_with_default(&mut result, current_param.trim());
                result.push_str(", ");
                current_param = String::default();
            }
            _ => {
                current_param.push(ch);
            }
        }
    }

    // Handle the last parameter
    let trimmed = current_param.trim();
    if !trimmed.is_empty() {
        append_param_with_default(&mut result, trimmed);
    }

    result
}

/// Append a single generic parameter with `= any` default if it doesn't have one.
fn append_param_with_default(result: &mut String, param: &str) {
    result.push_str(param);
    // Check if this param already has a default (contains `=` at depth 0)
    let mut depth = 0i32;
    let has_default = param.chars().any(|ch| {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            '=' if depth == 0 => return true,
            _ => {}
        }
        false
    });
    if !has_default {
        result.push_str(" = any");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        add_generic_defaults, collect_with_defaults_default_names_from_source,
        extract_generic_names, strip_const_modifiers, template_props_type_ref,
        type_reference_lookup_key,
    };
    use vize_carton::{FxHashSet, String};

    #[test]
    fn extracts_generic_names_skipping_const_modifier() {
        assert_eq!(
            extract_generic_names("T extends Foo, P extends Bar"),
            "T, P"
        );
        assert_eq!(extract_generic_names("const T extends Tab"), "T");
        assert_eq!(
            extract_generic_names("const T extends Record<string, any>, U"),
            "T, U"
        );
        // A prop named `constant` must not lose its prefix.
        assert_eq!(extract_generic_names("constant extends Foo"), "constant");
    }

    #[test]
    fn strips_const_modifiers_for_type_declarations() {
        assert_eq!(
            strip_const_modifiers("const T extends Tab = any").as_str(),
            "T extends Tab = any"
        );
        assert_eq!(
            strip_const_modifiers("const T extends Record<string, any>, const U = any").as_str(),
            "T extends Record<string, any>, U = any"
        );
        assert_eq!(
            strip_const_modifiers("T extends Tab = any").as_str(),
            "T extends Tab = any"
        );
        assert_eq!(
            strip_const_modifiers("constant extends Foo").as_str(),
            "constant extends Foo"
        );
        assert_eq!(
            strip_const_modifiers(add_generic_defaults("const T extends Tab").as_str()).as_str(),
            "T extends Tab = any"
        );
    }

    #[test]
    fn collects_with_defaults_object_keys() {
        let mut names = FxHashSet::default();
        collect_with_defaults_default_names_from_source(
            r#"withDefaults(defineProps<Props>(), {
  thickness: 0.1,
  "label": "ok",
  ...moreDefaults,
  [dynamicKey]: 1,
})"#,
            &mut names,
        );

        assert!(names.contains("thickness"));
        assert!(names.contains("label"));
        assert!(!names.contains("dynamicKey"));
        assert_eq!(names.len(), 2);

        let mut arg_names = FxHashSet::default();
        collect_with_defaults_default_names_from_source(
            r#"defineProps<Props>(), {
  count: 0,
  "title": "Counter",
}"#,
            &mut arg_names,
        );

        assert!(arg_names.contains("count"));
        assert!(arg_names.contains("title"));
        assert_eq!(arg_names.len(), 2);
    }

    #[test]
    fn builds_deterministic_with_defaults_props_type() {
        let mut names: FxHashSet<String> = FxHashSet::default();
        names.insert("label".into());
        names.insert("thickness".into());

        assert_eq!(
            template_props_type_ref("Props", &names),
            r#"__WithDefaultsResult<Props, Pick<Props, "label" | "thickness">>"#
        );
    }

    #[test]
    fn type_reference_lookup_key_strips_generics_but_preserves_inline_literals() {
        // Type references drop their generic instantiation so the resolver can
        // find the local declaration registered under its bare name.
        assert_eq!(type_reference_lookup_key("Props"), "Props");
        assert_eq!(type_reference_lookup_key("Foo<T>"), "Foo");
        assert_eq!(
            type_reference_lookup_key("ContextMenuContentProps<T, U>"),
            "ContextMenuContentProps"
        );
        // Inline object literals are passed through verbatim — the `<` inside a
        // property type must not be mistaken for a generic argument list.
        assert_eq!(
            type_reference_lookup_key("{ items: Array<{ id: string }> }"),
            "{ items: Array<{ id: string }> }"
        );
        assert_eq!(
            type_reference_lookup_key("  { msg: string }"),
            "  { msg: string }"
        );
    }
}
