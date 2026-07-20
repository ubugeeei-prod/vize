use vize_croquis::script_parser::parse_script_setup;
use vize_croquis::virtual_ts::{VirtualTsConfig, VirtualTsGenerator};

#[test]
fn compiler_macro_aliases_use_the_type_argument_body() {
    let script = r#"
const props = defineProps<Record<string, { value: string }>>()
const emit = defineEmits<{ save: [value: string] }>()
defineExpose<{ focus(): void }>()
defineSlots<{ default(props: { value: string }): unknown }>()
"#;
    let parse_result = parse_script_setup(script);
    let mut generator = VirtualTsGenerator::new();
    let output = generator.generate_from_croquis(
        script,
        &parse_result,
        None,
        &VirtualTsConfig::default(),
        None,
    );

    for expected in [
        "type __Props = Record<string, { value: string }>;",
        "type __Emits = { save: [value: string] };",
        "type __Exposed = { focus(): void };",
        "type __Slots = { default(props: { value: string }): unknown };",
    ] {
        assert!(
            output.content.contains(expected),
            "missing `{expected}` from generated output:\n{}",
            output.content
        );
    }
}
