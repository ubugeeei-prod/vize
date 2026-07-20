use super::parse_script;

#[test]
fn test_plain_script_setup_context_violations_stay_at_module_scope() {
    let source = r#"
const shared = ref(0)

export function installState() {
    provide("state", shared)
}
"#;
    let result = parse_script(source);
    let violations = result.setup_context.violations();

    assert_eq!(violations.len(), 1, "unexpected violations: {violations:?}");
    assert_eq!(violations[0].api_name, "ref");
    assert_eq!(
        violations[0].start as usize,
        source.find("ref(0)").expect("module-level ref call")
    );
}
