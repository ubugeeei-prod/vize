use std::path::Path;

use super::dts::{parse_declared_global_values_content, parse_interface_members_content};
use super::dts_ast::parse_global_component_members_content;

#[test]
fn parses_interface_members_with_multiline_types() {
    let content = r#"
declare module 'vue' {
  interface ComponentCustomProperties {
    foo: string
    bar:
      typeof import('./bar').bar
  }
}
"#;

    let members = parse_interface_members_content(content, "interface ComponentCustomProperties");
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].0.as_str(), "foo");
    assert_eq!(members[0].1.as_str(), "string");
    assert_eq!(members[1].0.as_str(), "bar");
    assert_eq!(members[1].1.as_str(), "typeof import('./bar').bar");
}

#[test]
fn parses_readonly_and_quoted_members_without_index_signatures() {
    let content = r#"
declare module 'vue' {
  interface ComponentCustomProperties {
    readonly $config?: typeof import('./config').config
    "quoted-key": string
    [key: string]: unknown
  }
}
"#;

    let members = parse_interface_members_content(content, "interface ComponentCustomProperties");

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].0.as_str(), "$config");
    assert_eq!(members[0].1.as_str(), "typeof import('./config').config");
    assert_eq!(members[1].0.as_str(), "quoted-key");
    assert_eq!(members[1].1.as_str(), "string");
}

#[test]
fn parses_single_line_exported_interface_members() {
    let content = r#"
declare module 'vue' {
  export interface ComponentCustomProperties { foo: string; bar: number }
}
"#;

    let members = parse_interface_members_content(content, "interface ComponentCustomProperties");

    assert_eq!(
        members
            .iter()
            .map(|(name, ty)| (name.as_str(), ty.as_str()))
            .collect::<Vec<_>>(),
        vec![("foo", "string"), ("bar", "number")]
    );
}

#[test]
fn ignores_interface_name_inside_string_literal() {
    let content = r#"
declare global {
  const marker: "interface ComponentCustomProperties"
}
declare module 'vue' {
  interface ComponentCustomProperties {
    actual: string
  }
}
"#;

    let members = parse_interface_members_content(content, "interface ComponentCustomProperties");

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].0.as_str(), "actual");
}

#[test]
fn parses_global_components_from_extended_interface() {
    let content = r#"
interface _GlobalComponents {
  GlobalButton: GlobalComponentConstructor<GlobalButtonProps>
  GlobalInput:
    GlobalComponentConstructor<GlobalInputProps>
}

declare module "vue" {
  interface GlobalComponents extends _GlobalComponents {}
}
"#;

    let members = parse_global_component_members_content(content);

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].0.as_str(), "GlobalButton");
    assert_eq!(
        members[0].1.as_str(),
        "GlobalComponentConstructor<GlobalButtonProps>"
    );
    assert_eq!(members[1].0.as_str(), "GlobalInput");
    assert_eq!(
        members[1].1.as_str(),
        "GlobalComponentConstructor<GlobalInputProps>"
    );
}

#[test]
fn parses_declared_globals_and_rewrites_relative_imports() {
    let content = r#"
declare global {
  const currentUser:
    typeof import('../../app/composables/users').currentUser
  var $t: (Composer)['t']
}
"#;

    let values = parse_declared_global_values_content(content, Path::new("/workspace/.nuxt/types"));
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].0.as_str(), "currentUser");
    assert_eq!(
        values[0].1.as_str(),
        "typeof import('/workspace/app/composables/users').currentUser"
    );
    assert_eq!(values[1].0.as_str(), "$t");
    assert_eq!(values[1].1.as_str(), "(Composer)['t']");
}

#[test]
fn parses_single_line_declared_global_values() {
    let content = r#"declare global { const currentUser: typeof import('./user').currentUser }"#;

    let values = parse_declared_global_values_content(content, Path::new("/workspace/.nuxt/types"));

    assert_eq!(
        values
            .iter()
            .map(|(name, ty)| (name.as_str(), ty.as_str()))
            .collect::<Vec<_>>(),
        vec![(
            "currentUser",
            "typeof import('/workspace/.nuxt/types/user').currentUser"
        )]
    );
}
