use super::{TemplateParts, compile_script_setup, compile_script_setup_inline};
use vize_carton::String;

fn compile_inline(script_content: &str) -> String {
    let result = compile_script_setup_inline(
        script_content,
        "TestComponent",
        false,
        true,
        false,
        TemplateParts {
            render_body: "null",
            ..Default::default()
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
fn rewrites_destructured_props_in_define_expose() {
    let content = r#"
const { foo: localFoo } = defineProps<{ foo: number }>()
defineExpose({
  arrow: (value: number) => localFoo + value,
  method() { return localFoo },
  functionExpression: function () { return localFoo },
  plain: localFoo,
  get value() { return localFoo },
  object: () => ({ localFoo }),
})
"#;
    let output = compile_inline(content);

    assert!(
        output.contains("arrow: (value) => __props.foo + value"),
        "{output}"
    );
    assert!(output.contains("return __props.foo"), "{output}");
    assert!(output.contains("plain: __props.foo"), "{output}");
    assert!(output.contains("localFoo: __props.foo"), "{output}");
    assert!(!output.contains("=> localFoo"), "{output}");
}

#[test]
fn rewrites_runtime_destructured_props_in_define_expose() {
    let output = compile_inline(
        r#"
const { foo } = defineProps(["foo"])
defineExpose({ bar: () => foo })
"#,
    );

    assert!(
        output.contains("__expose({ bar: () => __props.foo })"),
        "{output}"
    );
}

#[test]
fn rewrites_destructured_props_in_define_expose_function_mode() {
    let content = r#"
const { foo } = defineProps<{ foo: number }>()
defineExpose({ bar: () => foo })
"#;
    let result = compile_script_setup(content, "Test", false, true, None).unwrap();

    assert!(
        result
            .code
            .contains("__expose({ bar: () => __props.foo });"),
        "{}",
        result.code
    );
}
