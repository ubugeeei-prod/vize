use super::compile_script_setup_inline;
use crate::compile_script::TemplateParts;

fn compile(source: &str, preserve_typescript: bool) -> Result<vize_carton::String, String> {
    compile_script_setup_inline(
        source,
        "ProfileForm",
        preserve_typescript,
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
    .map(|result| result.code)
    .map_err(|error| format!("{}:{:?}", error.message, error.code))
}

#[test]
fn static_enum_defaults_are_module_scoped_in_ts_and_js() {
    let source = r#"
enum Step { Name, General = 1 + 1 }
interface Props { step?: Step }
const props = withDefaults(defineProps<Props>(), { step: Step.Name })
"#;

    for preserve_typescript in [true, false] {
        let output = compile(source, preserve_typescript).unwrap();
        let step = output.find("Step").expect("enum should remain at runtime");
        let component = output.find("export default").expect("component wrapper");
        assert!(
            step < component,
            "enum must be hoisted before setup:\n{output}"
        );
        assert!(output.contains("default: Step.Name"), "{output}");
        let expected_ts_enums = if preserve_typescript { 1 } else { 0 };
        assert_eq!(output.matches("enum Step").count(), expected_ts_enums);
    }
}

#[test]
fn runtime_dependent_enum_defaults_remain_rejected() {
    let source = r#"
const seed = runtime()
enum Step { Name = seed }
const props = withDefaults(defineProps<{ step?: Step }>(), { step: Step.Name })
"#;

    let error = compile(source, false).expect_err("dynamic enum cannot be hoisted");
    assert!(error.contains("SCRIPT_SETUP_MACRO_SCOPE"), "{error}");
    assert!(error.contains("`Step`"), "{error}");
}
