use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, append, cstr};

/// The template's resolved prop contract for a `withDefaults` declaration.
///
/// `boolean_casting` applies Vue's boolean casting: an absent prop whose type
/// is `boolean` is `false`, never `undefined`, exactly as `vue-tsc`'s
/// `PropsWithDefaults<T, D, BKeys>` resolves it. Its set names the props whose
/// default is literally `undefined`, which Vue leaves `boolean | undefined`.
/// It is only safe when `keyof base` is concrete (not a generic SFC), see
/// `__DefineProps` (#4242).
pub(super) fn template_props_type_ref(
    base_type_ref: &str,
    defaulted_prop_names: &FxHashSet<String>,
    boolean_casting: Option<&FxHashSet<String>>,
) -> String {
    if defaulted_prop_names.is_empty() {
        return base_type_ref.into();
    }

    // Vue accepts surplus defaults, but only declared props belong to the
    // template contract. Let TypeScript resolve the keys of imported and
    // generic prop types instead of guessing their shape in the generator.
    let default_keys = cstr!(
        "Extract<{}, keyof {base_type_ref}>",
        key_union(defaulted_prop_names)
    );
    let Some(undefined_defaults) = boolean_casting else {
        return cstr!(
            "__WithDefaultsResult<{base_type_ref}, Pick<{base_type_ref}, {default_keys}>>"
        );
    };
    let boolean_keys = if undefined_defaults.is_empty() {
        cstr!("__VizeBooleanKey<{base_type_ref}>")
    } else {
        let excluded = key_union(undefined_defaults);
        cstr!("Exclude<__VizeBooleanKey<{base_type_ref}>, {excluded}>")
    };
    cstr!(
        "__WithDefaultsResult<{base_type_ref}, Pick<{base_type_ref}, {default_keys}>, {boolean_keys}>"
    )
}

/// `"a" | "b"` over the names, sorted so the generated text is deterministic.
fn key_union(names: &FxHashSet<String>) -> String {
    let mut sorted: Vec<&str> = names.iter().map(|name| name.as_str()).collect();
    sorted.sort_unstable();
    let mut keys = String::default();
    for name in sorted {
        if !keys.is_empty() {
            keys.push_str(" | ");
        }
        append!(keys, "\"{name}\"");
    }
    keys
}

#[cfg(test)]
pub(super) fn collect_with_defaults_default_names_from_source(
    source: &str,
    names: &mut FxHashSet<String>,
) {
    collect_with_defaults_names_from_source(source, names, &mut FxHashSet::default());
}

/// Collect the props an object-literal `withDefaults` argument defaults:
/// `defined` gets those with a value, `undefined` those whose default is
/// literally `undefined` (Vue then keeps them `T | undefined`, booleans too).
pub(super) fn collect_with_defaults_names_from_source(
    source: &str,
    defined: &mut FxHashSet<String>,
    undefined: &mut FxHashSet<String>,
) {
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
        if is_undefined_default_value(&prop.value) {
            undefined.insert(name.into());
        } else {
            defined.insert(name.into());
        }
    }
}

fn is_undefined_default_value(value: &Expression<'_>) -> bool {
    matches!(value, Expression::Identifier(id) if id.name.as_str() == "undefined")
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_with_defaults_default_names_from_source;
    use vize_carton::FxHashSet;

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
    fn skips_undefined_object_keys() {
        let mut names = FxHashSet::default();
        collect_with_defaults_default_names_from_source(
            r#"withDefaults(defineProps<Props>(), {
  modelValue: undefined,
  label: "ok",
})"#,
            &mut names,
        );

        assert!(!names.contains("modelValue"));
        assert!(names.contains("label"));
        assert_eq!(names.len(), 1);
    }
}
