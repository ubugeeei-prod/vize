use super::compile_script_setup;

#[test]
fn preserves_define_model_setup_runtime_accessors() {
    let content = r#"
const minimum = computed(() => 1)
const [model, modifiers] = defineModel<number>({
  get(value) { return value ?? minimum.value },
  set(value) { return modifiers.number ? Math.max(minimum.value, Number(value)) : value },
})
"#;
    let result = compile_script_setup(content, "Test", false, true, None).unwrap();

    assert!(
        result.code.contains("_useModel(__props, \"modelValue\", {")
            && result.code.contains("return value ?? minimum.value"),
        "{}",
        result.code
    );
    assert!(result.code.contains("modifiers.number"), "{}", result.code);
}

#[test]
fn emits_static_enums_before_the_component() {
    let content = r#"
enum Step { Name, General = 1 + 1 }
const current = Step.Name
"#;
    let result = compile_script_setup(content, "ProfileForm", false, true, None).unwrap();

    let step = result
        .code
        .find("enum Step")
        .expect("enum should be emitted");
    let component = result
        .code
        .find("const __sfc__")
        .expect("component wrapper");
    assert!(step < component, "{}", result.code);
    assert!(
        result.code.contains("const current = Step.Name"),
        "{}",
        result.code
    );
}
