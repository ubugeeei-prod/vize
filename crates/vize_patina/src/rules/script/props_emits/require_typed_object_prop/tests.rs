use crate::rules::script::RequireTypedObjectProp;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireTypedObjectProp));
    linter
}

// ---------------------------------------------------------------------------
// Valid
// ---------------------------------------------------------------------------

#[test]
fn test_valid_shorthand_object_with_prop_type() {
    let source = r#"
const props = defineProps({
  foo: Object as PropType<Foo>
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_shorthand_array_with_prop_type() {
    let source = r#"
const props = defineProps({
  foo: Array as PropType<string[]>
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_descriptor_type_with_prop_type() {
    let source = r#"
const props = defineProps({
  foo: { type: Object as PropType<Foo> },
  bar: { type: Array as PropType<Bar[]>, default: () => [] }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_other_constructors() {
    // String / Number / Boolean / a custom class carry their own type and are
    // never flagged.
    let source = r#"
const props = defineProps({
  a: String,
  b: { type: Number },
  c: Boolean,
  d: Date,
  e: { type: MyClass }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_array_of_constructors() {
    // `[Object, Array]` is a union of accepted constructors, not a bare typeless
    // `Object`/`Array`; left alone (matches vue/require-typed-object-prop).
    let source = r#"
const props = defineProps({
  foo: [Object, Array],
  bar: { type: [String, Object] }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_type_based_define_props_ignored() {
    let source = r#"
const props = defineProps<{ foo: Foo; bar: Bar[] }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_array_form_define_props_ignored() {
    // Array form declares only names — no runtime type to inspect here.
    let source = r#"
const props = defineProps(['foo', 'bar'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_props_option() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_validator_function() {
    // A function value sits in the type position (a validator / PropType
    // factory) and is not the bare `Object`/`Array` constructor.
    let source = r#"
const props = defineProps({
  foo: { type: makeType() }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_object_options_api_with_prop_type() {
    let source = r#"
export default {
  props: {
    foo: { type: Object as PropType<Foo> }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

// ---------------------------------------------------------------------------
// Invalid
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_shorthand_object() {
    let source = r#"
const props = defineProps({
  foo: Object
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_shorthand_array() {
    let source = r#"
const props = defineProps({
  foo: Array
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_descriptor_type_object() {
    let source = r#"
const props = defineProps({
  foo: { type: Object }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_descriptor_type_array() {
    let source = r#"
const props = defineProps({
  foo: { type: Array, default: () => [] }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_options_api_props() {
    let source = r#"
export default {
  props: {
    foo: Object,
    bar: { type: Array }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_invalid_define_component() {
    let source = r#"
export default defineComponent({
  props: {
    foo: { type: Object }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_mixed_counts() {
    let source = r#"
const props = defineProps({
  typed: Object as PropType<Foo>,
  bad1: Object,
  bad2: { type: Array },
  ok: String
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_offset_applied() {
    let source = r#"const props = defineProps({ foo: Object })"#;
    let result = create_linter().lint(source, 100);
    assert_eq!(result.warning_count, 1);
    let object_start = source.find("Object").unwrap() as u32 + 100;
    assert_eq!(result.diagnostics[0].start, object_start);
}
