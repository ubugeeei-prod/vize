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

#[test]
fn a_module_level_template_ref_has_no_instance_to_register_on() {
    let source = r#"
import { defineComponent, useTemplateRef } from "vue"
const el = useTemplateRef("el")

export default defineComponent({
    setup() {
        const inside = useTemplateRef("inside")
        return { inside }
    },
})
"#;
    let result = parse_script(source);
    let violations: Vec<_> = result
        .setup_context
        .violations()
        .iter()
        .map(|violation| (violation.kind, violation.api_name.as_str(), violation.start))
        .collect();
    assert_eq!(
        violations,
        [(
            crate::setup_context::SetupContextViolationKind::ModuleLevelTemplateRef,
            "useTemplateRef",
            source
                .find("useTemplateRef(\"el\")")
                .expect("module-level call") as u32,
        )]
    );
}
