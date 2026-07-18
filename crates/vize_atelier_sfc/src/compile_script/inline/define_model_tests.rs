use super::compile_script_setup_inline;
use crate::compile_script::TemplateParts;
use vize_carton::String;

fn compile_setup(script_content: &str) -> String {
    let result = compile_script_setup_inline(
        script_content,
        "TestComponent",
        false,
        true,
        false,
        TemplateParts {
            imports: "",
            hoisted: "",
            render_fn: "",
            render_fn_name: "",
            preamble: "",
            render_body: "null",
            render_is_block: false,
        },
        None,
        &[],
        "",
        None,
    )
    .expect("compilation should succeed");
    result.code
}

#[test]
fn define_model_uses_ast_argument_metadata() {
    let content = r#"
const model = defineModel(('label') as const, {
  default: 'Untitled',
  get(value) {
    return value
  },
  set(value) {
    return value.trim()
  },
})
"#;

    let output = compile_setup(content);
    let normalized: String = output
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .into();

    assert!(
        normalized.contains(r#""label": { default: "Untitled" }"#),
        "expected the AST string argument to drive the model prop name:\n{output}"
    );
    assert!(
        normalized.contains(r#"emits: ["update:label"]"#),
        "expected model emits to use the AST string argument:\n{output}"
    );
    assert!(
        normalized.contains(r#"const model = _useModel(__props, "label", {"#)
            && normalized.contains("get(value) { return value; }")
            && normalized.contains("set(value) { return value.trim(); }"),
        "expected useModel to preserve runtime accessors:\n{output}"
    );
    assert!(
        !normalized.contains("modelValue"),
        "parenthesized/as string argument should not fall back to modelValue:\n{output}"
    );
}

#[test]
fn define_model_runtime_accessors_can_reference_setup_bindings() {
    let content = r#"
const minimum = computed(() => 1)
const [model, modifiers] = defineModel<number>({
  get(value) {
    return value ?? minimum.value
  },
  set(value) {
    return modifiers.number ? Math.max(minimum.value, Number(value)) : value
  },
})
"#;

    let output = compile_setup(content);
    assert!(output.contains("minimum.value"), "{output}");
    assert!(output.contains("modifiers.number"), "{output}");
    assert!(
        output.contains("_useModel(__props, \"modelValue\", {"),
        "{output}"
    );
}
