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
    insta::assert_snapshot!(compile_inline(content));
}

#[test]
fn rewrites_runtime_destructured_props_in_define_expose() {
    let output = compile_inline(
        r#"
const { foo } = defineProps(["foo"])
defineExpose({ bar: () => foo })
"#,
    );

    insta::assert_snapshot!(output);
}

#[test]
fn rewrites_destructured_props_in_define_expose_function_mode() {
    let content = r#"
const { foo } = defineProps<{ foo: number }>()
defineExpose({ bar: () => foo })
"#;
    let result = compile_script_setup(content, "Test", false, true, None).unwrap();

    insta::assert_snapshot!(result.code);
}
