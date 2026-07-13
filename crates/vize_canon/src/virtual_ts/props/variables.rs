//! Setup/template-scope prop binding emission.

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, macros::MacroKind};

use super::generics::strip_generic_params;
use super::template_bindings::{
    emit_macro_template_prop_bindings, should_skip_template_prop_binding,
};
use super::with_defaults::collect_with_defaults_default_names_from_source;
use super::{
    is_reserved_identifier, props_type_ref, strip_outer_angle_brackets, to_safe_identifier,
    type_reference_lookup_key,
};

pub(super) fn emit_template_prop_binding(
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
    key_type_ref: &str,
    prop_name: &str,
    has_default: bool,
) {
    let binding_name = to_safe_identifier(prop_name);
    if has_default {
        append!(
            *ts,
            "  const {binding_name} = props[(\"{prop_name}\" satisfies keyof {key_type_ref})] as Exclude<{props_type_ref}[\"{prop_name}\"], undefined>;\n"
        );
    } else {
        append!(
            *ts,
            "  const {binding_name} = props[(\"{prop_name}\" satisfies keyof {key_type_ref})];\n"
        );
    }
    append!(*ts, "  void {binding_name};\n");
}

fn emit_unchecked_template_prop_binding(ts: &mut String, prop_name: &str) {
    let binding_name = to_safe_identifier(prop_name);
    append!(
        *ts,
        "  const {binding_name} = (props as Record<string, unknown>)[\"{prop_name}\"];\n"
    );
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
    for undefined in &summary.undefined_refs {
        let name = undefined.name.as_str();
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
    if summary.types.definitions().has_interface_extends(base_name) {
        return true;
    }
    if let Some(body) = summary.types.definitions().resolve(base_name) {
        return has_top_level_type_operator(body.as_str())
            || !is_plain_inline_type_literal(body.as_str());
    }
    emitted_names.is_empty() && !summary.types.definitions().is_defined(base_name)
}

fn is_plain_inline_type_literal(type_name: &str) -> bool {
    let type_name = type_name.trim();
    if !type_name.starts_with('{') {
        return false;
    }
    let mut depth = 0i32;
    for (idx, character) in type_name.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return type_name[idx + character.len_utf8()..].trim().is_empty();
                }
            }
            _ => {}
        }
    }
    false
}

fn has_top_level_type_operator(type_name: &str) -> bool {
    let (mut angle, mut brace, mut paren, mut bracket) = (0i32, 0i32, 0i32, 0i32);
    for character in type_name.chars() {
        match character {
            '<' => angle += 1,
            '>' => angle -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            '&' | '|' if angle == 0 && brace == 0 && paren == 0 && bracket == 0 => return true,
            _ => {}
        }
    }
    false
}

fn collect_with_defaults_default_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for call in summary.macros.all_calls() {
        if call.kind == MacroKind::WithDefaults
            && let Some(runtime_args) = &call.runtime_args
        {
            collect_with_defaults_default_names_from_source(runtime_args.as_str(), &mut names);
        }
    }
    names
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

pub(crate) fn generate_props_variables(
    ts: &mut String,
    summary: &Croquis,
    generic_param: Option<&str>,
    props_type_ref_override: Option<&str>,
    check_props: bool,
) {
    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let models = summary.macros.models();
    let has_models = !models.is_empty();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|macro_call| macro_call.type_args.as_ref());
    let props_type_ref = props_type_ref(generic_param, props_type_ref_override);
    let mut defaulted_prop_names = collect_with_defaults_default_names(summary);
    for model in models {
        if model.default_value.is_some() {
            defaulted_prop_names.insert(model.name.as_str().into());
        }
    }
    let template_base_props_type_ref = if define_props_type_args.is_some() {
        cstr!("__DefineProps<{props_type_ref}>")
    } else {
        props_type_ref.clone()
    };
    let template_props_type_ref =
        template_props_type_ref(template_base_props_type_ref.as_str(), &defaulted_prop_names);

    if !(has_props || define_props_type_args.is_some() || has_models) {
        return;
    }
    ts.push_str("  // Props are available in template as variables\n");
    ts.push_str("  // Access via `propName` or `props.propName`\n");
    append!(
        *ts,
        "  const props: {template_props_type_ref} = {{}} as {template_props_type_ref};\n"
    );
    ts.push_str("  void props; // Mark as used to avoid TS6133\n");

    let mut emitted_names = FxHashSet::default();
    if let Some(type_args) = define_props_type_args {
        let type_name = strip_outer_angle_brackets(type_args.trim());
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
        if has_props {
            emit_macro_template_prop_bindings(
                ts,
                summary,
                template_props_type_ref.as_str(),
                props,
                &defaulted_prop_names,
                &mut emitted_names,
            );
        }
        if should_emit_keyed_template_prop_bindings(summary, type_name, &emitted_names) {
            for name in collect_keyed_template_prop_names(summary, &emitted_names) {
                if check_props {
                    emit_keyed_template_prop_binding(
                        ts,
                        template_props_type_ref.as_str(),
                        props_type_ref.as_str(),
                        name.as_str(),
                        defaulted_prop_names.contains(&name),
                    );
                } else {
                    emit_unchecked_template_prop_binding(ts, name.as_str());
                }
            }
        }
    } else if has_props {
        emit_macro_template_prop_bindings(
            ts,
            summary,
            template_props_type_ref.as_str(),
            props,
            &defaulted_prop_names,
            &mut emitted_names,
        );
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
    }
    ts.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
