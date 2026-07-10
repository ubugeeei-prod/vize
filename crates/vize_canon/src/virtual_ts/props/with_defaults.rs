use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, cstr};

pub(super) fn collect_with_defaults_default_names_from_source(
    source: &str,
    names: &mut FxHashSet<String>,
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
        if prop.computed || is_undefined_default_value(&prop.value) {
            continue;
        }
        let Some(name) = property_key_name(&prop.key) else {
            continue;
        };
        names.insert(name.into());
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
