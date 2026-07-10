use super::NoRequiredPropWithDefault;
use crate::rules::script::ScriptLinter;

fn errors(source: &str) -> usize {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoRequiredPropWithDefault));
    linter.lint(source, 0).error_count
}

#[test]
fn test_valid_cases() {
    for source in [
        // required without default / optional with default / required:false
        "export default { props: { value: { type: Number, required: true } } }",
        "export default { props: { label: { type: String, default: '' } } }",
        "export default { props: { x: { required: false, default: '' } } }",
        // array props / shorthand ctor / no options object
        "export default { props: ['foo', 'bar'] }",
        "export default { props: { count: Number } }",
        "import { ref } from 'vue'\nconst c = ref(0)",
        // `required: someFlag` is not a literal `true`
        "export default { props: { x: { required: isRequired, default: 0 } } }",
    ] {
        assert_eq!(errors(source), 0, "expected no errors for: {source}");
    }
}

#[test]
fn test_invalid_cases() {
    let cases = [
        // direct form / defineComponent wrapper / identifier-bound options /
        // identifier-bound props object / string-literal prop key
        (
            "export default { props: { v: { type: Number, required: true, default: 0 } } }",
            1,
        ),
        (
            "export default defineComponent({ props: { v: { required: true, default: 1 } } })",
            1,
        ),
        (
            "const c = { props: { v: { required: true, default: 1 } } }\nexport default c",
            1,
        ),
        (
            "const props = { v: { required: true, default: 1 } }\nexport default { props }",
            1,
        ),
        (
            "export default { props: { 'data-id': { required: true, default: 0 } } }",
            1,
        ),
        // multiple offending props reported, valid ones skipped
        (
            "export default { props: { a: { required: true, default: 1 }, \
                 b: { type: String, default: 'x' }, c: { required: true, default: 3 } } }",
            2,
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(errors(source), expected, "for: {source}");
    }
}

#[test]
fn test_diagnostic_snapshot() {
    let source = r#"
export default {
  props: {
    value: { type: Number, required: true, default: 0 }
  }
}
"#;
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoRequiredPropWithDefault));
    let result = linter.lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}
