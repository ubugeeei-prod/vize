use super::ComponentOptionsNameCasing;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(ComponentOptionsNameCasing));
    linter
}

#[test]
fn accepts_pascal_case_registered_components() {
    let source = r#"
export default {
  components: { MyComponent, Heading2, Button }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn reports_static_component_keys_with_upstream_ranges() {
    let source = r#"export default {
  components: { fooBar, "foo-baz": Foo, GoodOne }
}"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);

    let first = source.find("fooBar").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (first, first + "fooBar".len() as u32)
    );
    let second = source.find("\"foo-baz\"").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[1].start, result.diagnostics[1].end),
        (second, second + "\"foo-baz\"".len() as u32)
    );
}

#[test]
fn applies_the_script_offset_to_component_key_ranges() {
    let source = "export default { components: { fooBar } }";
    let result = create_linter().lint(source, 41);
    assert_eq!(result.error_count, 1);
    let local = source.find("fooBar").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, 41 + local);
    assert_eq!(result.diagnostics[0].end, 41 + local + 6);
}

#[test]
fn checks_define_component_and_identifier_bound_options() {
    let direct = r#"
import { defineComponent } from 'vue'
export default defineComponent({ components: { fooBar } })
"#;
    assert_eq!(create_linter().lint(direct, 0).error_count, 1);

    let bound = r#"
const component = { components: { fooBar } }
export default component
"#;
    assert_eq!(create_linter().lint(bound, 0).error_count, 1);
}

#[test]
fn checks_wrapped_options_objects() {
    let source = r#"
export default {
  components: { fooBar }
} satisfies { components: object }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn ignores_the_components_own_name_option() {
    let source = r#"
export default {
  name: 'my-component'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn ignores_define_options_name_and_components() {
    let source = r#"
defineOptions({
  name: 'my-component',
  components: { fooBar }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn ignores_component_maps_held_in_separate_variables() {
    let source = r#"
const components = { fooBar }
export default { components }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn ignores_computed_and_spread_component_keys() {
    let source = r#"
const key = 'fooBar'
const inherited = { fooBar }
export default {
  components: {
    [key]: Foo,
    ...inherited,
    GoodOne
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn ignores_scripts_without_a_component_options_object() {
    let source = r#"
import { ref } from 'vue'
const fooBar = ref(0)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
