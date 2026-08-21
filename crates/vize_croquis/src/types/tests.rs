use super::{TypeDefinitions, TypeResolver};

#[test]
fn test_extract_inline_props() {
    let resolver = TypeResolver::new();
    let props = resolver.extract_properties("{ msg: string, count?: number }");

    assert_eq!(props.len(), 2);
    assert_eq!(props[0].name.as_str(), "msg");
    assert!(!props[0].optional);
    assert_eq!(props[1].name.as_str(), "count");
    assert!(props[1].optional);
}

#[test]
fn extract_properties_ignores_leading_comments() {
    let resolver = TypeResolver::new();
    let props = resolver.extract_properties(
        r#"{
  /* café 🌱 */ title: string;
  /** A URL literal follows. */
  epilogueText?: "https://example.com/a/*b*/";
  // The colon here is trivia: not a property.
  count: number;
}"#,
    );

    let names = props
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["title", "epilogueText", "count"]);
    assert_eq!(
        props[1].prop_type.as_deref(),
        Some("\"https://example.com/a/*b*/\"")
    );
}

#[test]
fn extract_properties_strips_comments_only_inside_template_substitutions() {
    let resolver = TypeResolver::new();
    let props = resolver.extract_properties(
        r#"{
  value: `${string /* } */}`;
  raw: `/* raw } comment marker */`;
  count: number;
}"#,
    );

    let names = props
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["value", "raw", "count"]);
    assert_eq!(props[0].prop_type.as_deref(), Some("`${string  }`"));
    assert_eq!(
        props[1].prop_type.as_deref(),
        Some("`/* raw } comment marker */`")
    );
}

#[test]
fn test_extract_props_from_reference() {
    let mut resolver = TypeResolver::new();
    resolver.add_interface("Props", "{ foo: string; bar: number }");

    let props = resolver.extract_properties("Props");
    assert_eq!(props.len(), 2);
    assert_eq!(props[0].name.as_str(), "foo");
    assert_eq!(props[1].name.as_str(), "bar");
}

#[test]
fn extract_properties_includes_interface_pick_heritage() {
    let mut resolver = TypeResolver::new();
    resolver.add_interface("BaseProps", "{ required?: boolean; disabled?: boolean }");
    resolver.add_interface_with_extends(
        "FooProps",
        "{ value?: T }",
        vec!["Pick<BaseProps, \"required\">".into()],
    );

    let props = resolver.extract_properties("FooProps<T>");
    let names = props
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, ["required", "value"]);
}

#[test]
fn extract_properties_includes_direct_interface_heritage() {
    let mut resolver = TypeResolver::new();
    resolver.add_interface("BaseProps", "{ required?: boolean }");
    resolver.add_interface_with_extends("FooProps", "{ label: string }", vec!["BaseProps".into()]);

    let props = resolver.extract_properties("FooProps");
    let names = props
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, ["required", "label"]);
}

#[test]
fn extract_properties_keeps_union_members_nested() {
    let mut resolver = TypeResolver::new();
    resolver.add_interface(
        "Props",
        r#"{
  isOpened: boolean
  title: string
  timeout?: number
  interaction?:
    | {
        text: string
        to: string
        event?: never
      }
    | {
        text: string
        event: () => void
        to?: never
      }
}"#,
    );

    let names = resolver
        .extract_properties("Props")
        .into_iter()
        .map(|prop| prop.name)
        .collect::<Vec<_>>();

    assert_eq!(
        names.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
        ["isOpened", "title", "timeout", "interaction"],
        "nested union members must not become top-level props"
    );
}

#[test]
fn test_extract_emits_call_signature() {
    let resolver = TypeResolver::new();
    let emits =
        resolver.extract_emits("{ (e: 'click'): void; (e: 'update', value: number): void }");

    assert_eq!(emits.len(), 2);
    assert_eq!(emits[0].as_str(), "click");
    assert_eq!(emits[1].as_str(), "update");
}

#[test]
fn test_extract_emits_object_type() {
    let resolver = TypeResolver::new();
    let emits = resolver.extract_emits("{ click: []; update: [value: number] }");

    assert_eq!(emits.len(), 2);
    assert_eq!(emits[0].as_str(), "click");
    assert_eq!(emits[1].as_str(), "update");
}

#[test]
fn test_type_definitions() {
    let mut defs = TypeDefinitions::new();
    defs.add_interface("Props", "{ msg: string }");
    defs.add_type_alias("Count", "number");

    assert!(defs.is_defined("Props"));
    assert!(defs.is_defined("Count"));
    assert!(!defs.is_defined("Unknown"));

    assert!(defs.resolve("Props").is_some());
    assert!(defs.resolve("Count").is_some());
}
