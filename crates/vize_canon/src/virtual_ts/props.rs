mod keyed_template_names;
mod mappings;
mod model_props;
mod options_api;
mod setup_scoped;
mod template_bindings;
mod template_model;
mod template_model_modifiers;
mod template_names;
mod with_defaults;
use super::generic_parameters::GenericParameters;
pub(crate) use mappings::{PropBindingMappings, PropsSource, prop_source};
pub(crate) use model_props::emitted_model_prop_names;
use model_props::{append_macro_props_type_literal, append_model_props_type_literal};
pub(crate) use options_api::OptionsApiPropsSource;
pub(crate) use options_api::append_default_props;
use options_api::emit_options_api_props_type;
use setup_scoped::unused_generic_comment;
pub(crate) use setup_scoped::{PropsTypeEmission, generate_setup_scoped_props_artifact};
pub(crate) use template_model::{TemplatePropsModel, TemplatePropsScope, generate_props_variables};
pub(crate) use template_names::collect_template_prop_names;
use vize_carton::{CompactString, FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

/// Use the same local/shadow decision and emitted spelling as prop declarations.
/// Authored setup locals, imports and reactive destructures keep their own names.
pub(crate) fn setup_anchor_identifier(
    summary: &Croquis,
    name: String,
    binding_type: Option<vize_croquis::BindingType>,
) -> String {
    if template_bindings::should_skip_template_prop_binding_with_type(
        summary,
        name.as_str(),
        binding_type,
    ) {
        name
    } else {
        mappings::prop_binding_identifier(name.as_str())
    }
}

fn inner_macro_type(type_args: &str) -> &str {
    type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args)
}

pub(crate) fn generate_props_type(
    ts: &mut String,
    summary: &Croquis,
    generic_param: Option<&GenericParameters>,
    options_api_props: Option<&OptionsApiPropsSource>,
    type_only_imported_names: &FxHashSet<CompactString>,
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
    let define_props_inner_type =
        define_props_type_args.map(|type_args| inner_macro_type(type_args.trim()));
    let define_props_is_authored = summary.macros.define_props().is_some();
    let props_imported_type_only = type_only_imported_names.contains("Props");
    let imported_props_would_collide = props_imported_type_only
        && (define_props_is_authored || has_models || options_api_props.is_some());
    let props_already_defined = summary
        .type_exports
        .iter()
        .any(|te| te.name.as_str() == "Props")
        || imported_props_would_collide;

    let generic_decl = generic_param
        .map(|g| cstr!("<{}>", g.alias_declaration))
        .unwrap_or_default();

    ts.push_str("// ========== Exported Types ==========\n");

    if emission == PropsTypeEmission::DeferredToSetup && define_props_type_args.is_some() {
    } else if imported_props_would_collide {
        if let Some(inner_type) = define_props_inner_type {
            ts.push_str("type __VizeResolvedProps = ");
            ts.push_str(inner_type);
            if has_models {
                ts.push_str(" & ");
                append_model_props_type_literal(ts, summary, models, generic_param);
            }
            ts.push_str(";\n");
        } else if has_props || has_models {
            ts.push_str("type __VizeResolvedProps = ");
            append_macro_props_type_literal(ts, summary, models, generic_param);
            ts.push_str(";\n");
        } else if let Some(options_api_props) = options_api_props {
            emit_options_api_props_type(
                ts,
                &generic_decl,
                options_api_props,
                "__VizeResolvedProps",
                false,
            );
        }
    } else if props_already_defined {
        if let Some(inner_type) =
            define_props_inner_type.filter(|_| hoisted_props_alias_replaced(summary))
        {
            append!(*ts, "type __VizeResolvedProps{generic_decl} = {inner_type}");
            if has_models {
                ts.push_str(" & ");
                append_model_props_type_literal(ts, summary, models, generic_param);
            }
            ts.push_str(";\n");
        } else if has_models {
            ts.push_str("type __VizeResolvedProps = Props & ");
            append_model_props_type_literal(ts, summary, models, generic_param);
            ts.push_str(";\n");
        }
    } else if let Some(inner_type) = define_props_inner_type {
        ts.push_str(unused_generic_comment(generic_param, inner_type));
        if has_models {
            append!(*ts, "export type Props{generic_decl} = {inner_type} & ");
            append_model_props_type_literal(ts, summary, models, generic_param);
            ts.push_str(";\n");
        } else {
            append!(*ts, "export type Props{generic_decl} = {inner_type};\n");
        }
    } else if has_props || has_models {
        append!(*ts, "export type Props{generic_decl} = ");
        append_macro_props_type_literal(ts, summary, models, generic_param);
        ts.push_str(";\n");
    } else if let Some(options_api_props) = options_api_props {
        emit_options_api_props_type(ts, &generic_decl, options_api_props, "Props", true);
    } else {
        append!(*ts, "export type Props{generic_decl} = {{}};\n");
    }

    ts.push('\n');
}

/// A hoisted authored `Props` alias is the component contract only while the
/// `defineProps` argument names it; vue-tsc always uses the argument itself,
/// e.g. a typed-page plugin's `props: true` intersection.
pub(crate) fn hoisted_props_alias_replaced(summary: &Croquis) -> bool {
    let Some(inner) = summary
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_deref())
        .map(|type_args| inner_macro_type(type_args.trim()).trim())
    else {
        return false;
    };
    let names_props = inner == "Props"
        || (inner.starts_with("Props<")
            && inner.ends_with('>')
            && !template_model::has_top_level_type_operator(inner));
    !names_props
        && summary
            .type_exports
            .iter()
            .any(|export| export.hoisted && export.name.as_str() == "Props")
}

/// Lookup key for a `defineProps<...>` type argument when resolving its fields
/// through the croquis `TypeResolver`.
///
/// Inline object types (`{ msg: string }`) are passed through verbatim — the
/// resolver parses them directly. A type *reference* may carry a generic
/// instantiation (`Foo<T>`); the resolver registers local types under their
/// bare declaration name, so strip the arguments to recover `Foo`.
pub(crate) fn type_reference_lookup_key(type_name: &str) -> &str {
    if type_name.trim_start().starts_with('{') {
        type_name
    } else {
        strip_generic_params(type_name).trim()
    }
}

/// Strip the outermost `<...>` pair from a type_args string, handling nested generics.
/// e.g., `"<Props>"` → `"Props"`, `"<Foo<T>>"` → `"Foo<T>"`, `"Props"` → `"Props"`
pub(crate) fn strip_outer_angle_brackets(s: &str) -> &str {
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
                    return s.get(1..i).unwrap_or(s);
                }
            }
            _ => {}
        }
    }
    s
}

/// Strip generic parameters from a type name for interface lookup.
/// e.g., `"ContextMenuContentProps<T>"` → `"ContextMenuContentProps"`
pub(super) fn strip_generic_params(type_name: &str) -> &str {
    type_name
        .split_once('<')
        .map_or(type_name, |(name, _)| name)
}

/// First identifier of a generic parameter declaration, skipping the TS 5.0
/// `const` modifier: `const T extends Tab` declares `T`, not `const`.
#[cfg(test)]
fn extract_generic_names(raw: &str) -> String {
    super::generic_parameters::GenericParameters::parse(raw).names
}

#[cfg(test)]
fn strip_const_modifiers(raw: &str) -> String {
    if raw.trim().is_empty() {
        return String::default();
    }
    super::generic_parameters::GenericParameters::parse(raw).alias_authored
}

#[cfg(test)]
fn add_generic_defaults(raw: &str) -> String {
    super::generic_parameters::GenericParameters::parse(raw).declaration
}

#[cfg(test)]
mod tests {
    use super::with_defaults::template_props_type_ref;
    use super::{
        add_generic_defaults, extract_generic_names, strip_const_modifiers,
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
    fn reused_generic_parameter_storage_preserves_complete_output() {
        for (input, expected) in [
            ("", ""),
            (" \t\r\n ", ""),
            ("const T, const U", "T, U"),
            ("constant, constructor", "constant, constructor"),
            (
                "const LongAuthoredGenericParameter extends Record<string, readonly [number, string]>, const T, const 日本 = 'quoted'",
                "LongAuthoredGenericParameter extends Record<string, readonly [number, string]>, T, 日本 = 'quoted'",
            ),
            (
                "const 日本 extends Record<string, '🧭'>, const T, const LongAuthoredGenericParameter = 日本",
                "日本 extends Record<string, '🧭'>, T, LongAuthoredGenericParameter = 日本",
            ),
        ] {
            let actual = strip_const_modifiers(input);
            assert_eq!(actual, expected);
            assert_eq!(strip_const_modifiers(&actual), expected);
        }
        for count in [1, 2, 17, 257] {
            let parameters: Vec<_> = (0..count)
                .map(|index| {
                    vize_carton::cstr!(
                        "const LongAuthoredGenericParameter{index} extends Record<string, number>"
                    )
                })
                .collect();
            let expected: Vec<_> = (0..count)
                .map(|index| {
                    vize_carton::cstr!(
                        "LongAuthoredGenericParameter{index} extends Record<string, number>"
                    )
                })
                .collect();
            assert_eq!(
                strip_const_modifiers(&parameters.join(", ")),
                expected.join(", ")
            );
        }
    }

    #[test]
    fn const_modifier_newlines_preserve_invalid_authored_input() {
        // Both TypeScript 6 and native 7 report TS1359 on this const: a line
        // break ends its modifier role. Do not turn it into a clean alias.
        let raw = "const\r\nLongAuthoredGenericParameter extends Map<string, Set<number>>,\r\nconst T = \"quoted\"";
        let plan = super::super::generic_parameters::GenericParameters::parse(raw);
        assert_ne!(plan.diagnostics, Vec::new());
        assert_eq!(plan.declaration, raw);
        assert_eq!(plan.alias_declaration, raw);
        assert_eq!(plan.alias_authored, raw);
    }

    #[test]
    fn builds_deterministic_with_defaults_props_type() {
        let mut names: FxHashSet<String> = FxHashSet::default();
        names.insert("label".into());
        names.insert("thickness".into());

        assert_eq!(
            template_props_type_ref("Props", &names, None),
            r#"__WithDefaultsResult<Props, Pick<Props, Extract<"label" | "thickness", keyof Props>>>"#
        );
        let no_undefined = FxHashSet::default();
        assert_eq!(
            template_props_type_ref("Props", &names, Some(&no_undefined)),
            r#"__WithDefaultsResult<Props, Pick<Props, Extract<"label" | "thickness", keyof Props>>, __VizeBooleanKey<Props>>"#
        );
        let mut undefined: FxHashSet<String> = FxHashSet::default();
        undefined.insert("open".into());
        assert_eq!(
            template_props_type_ref("Props", &names, Some(&undefined)),
            r#"__WithDefaultsResult<Props, Pick<Props, Extract<"label" | "thickness", keyof Props>>, Exclude<__VizeBooleanKey<Props>, "open">>"#
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
