use super::super::{ScriptParseResult, parse_script, parse_script_setup};
use crate::reactivity::ReactiveKind;

#[test]
fn plain_script_exported_bindings_are_collected() {
    let result = parse_script(
        r#"
export const foo = 'bar'
export function hello() {}
export class MyClass {}
"#,
    );

    assert!(result.bindings.contains("foo"));
    assert!(result.bindings.contains("hello"));
    assert!(result.bindings.contains("MyClass"));
    assert!(result.invalid_exports.is_empty());
}

pub(super) fn assert_reactive_source(
    result: &ScriptParseResult,
    source: &str,
    name: &str,
    expected_kind: ReactiveKind,
) {
    let reactive_source = result
        .reactivity
        .lookup(name)
        .unwrap_or_else(|| panic!("expected reactive source `{name}`"));
    let expected_offset = source
        .find(name)
        .unwrap_or_else(|| panic!("expected `{name}` in test source"));

    assert_eq!(reactive_source.kind, expected_kind, "kind for `{name}`");
    assert_eq!(
        reactive_source.declaration_offset as usize, expected_offset,
        "declaration byte offset for `{name}`"
    );
    assert_eq!(
        source.get(expected_offset..expected_offset + name.len()),
        Some(name),
        "offset for `{name}` should select its identifier"
    );
}

#[test]
fn declaration_offsets_are_utf8_bytes_for_multiple_declarators() {
    let source = r#"const unicode_prefix = '東京🦀'
const first_ref = ref(0), second_state = reactive({}), third_computed = computed(() => first_ref.value)
"#;
    let result = parse_script_setup(source);

    let first_offset = source.find("first_ref").unwrap();
    assert_ne!(
        first_offset,
        source[..first_offset].chars().count(),
        "the fixture must distinguish UTF-8 byte and character offsets"
    );
    assert_reactive_source(&result, source, "first_ref", ReactiveKind::Ref);
    assert_reactive_source(&result, source, "second_state", ReactiveKind::Reactive);
    assert_reactive_source(&result, source, "third_computed", ReactiveKind::Computed);
}

#[test]
fn alias_offset_is_not_confused_by_parameter_shadowing() {
    let source = r#"import { ref as make_ref } from 'vue'
const aliased_ref = make_ref(0)
const invoke_shadowed = (make_ref: () => unknown) => {
    const shadowed_result = make_ref()
    return shadowed_result
}
"#;
    let result = parse_script_setup(source);

    assert_reactive_source(&result, source, "aliased_ref", ReactiveKind::Ref);
    assert_eq!(result.reactivity.count(), 1);
    assert!(result.reactivity.lookup("shadowed_result").is_none());
}

#[test]
fn wrapper_kinds_keep_declaration_offsets() {
    let source = r#"const ref_value = ref(0)
const shallow_ref_value = shallowRef(0)
const reactive_value = reactive({ count: 0 })
const shallow_reactive_value = shallowReactive({ count: 0 })
const computed_value = computed(() => ref_value.value)
const to_ref_value = toRef(reactive_value, 'count')
const to_refs_value = toRefs(reactive_value)
const readonly_value = readonly(reactive_value)
const shallow_readonly_value = shallowReadonly(reactive_value)
const custom_ref_value = customRef(() => ({}))
const template_ref_value = useTemplateRef('input')
const model_value = defineModel<number>()
"#;
    let result = parse_script_setup(source);

    for (name, kind) in [
        ("ref_value", ReactiveKind::Ref),
        ("shallow_ref_value", ReactiveKind::ShallowRef),
        ("reactive_value", ReactiveKind::Reactive),
        ("shallow_reactive_value", ReactiveKind::ShallowReactive),
        ("computed_value", ReactiveKind::Computed),
        ("to_ref_value", ReactiveKind::ToRef),
        ("to_refs_value", ReactiveKind::ToRefs),
        ("readonly_value", ReactiveKind::Readonly),
        ("shallow_readonly_value", ReactiveKind::ShallowReadonly),
        ("custom_ref_value", ReactiveKind::Ref),
        ("template_ref_value", ReactiveKind::ShallowRef),
        ("model_value", ReactiveKind::Ref),
    ] {
        assert_reactive_source(&result, source, name, kind);
    }
    assert_eq!(result.reactivity.count(), 12);
}
