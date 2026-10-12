//! Actual ordinary lint dispatch over the finite component-value corpus.

use serde_json::{Value, json};
use std::path::Path;
use vize_croquis::binding_occurrences::OccurrenceBlock;
use vize_patina::Linter;
use vize_patina::RuleRegistry;

const RULE: &str = "vue/no-unused-components";
const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-import-value-reads/",
);
const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-import-value-reads/cases.json",
));

fn whole_diagnostics(linter: &Linter, source: &str) -> Value {
    let result = linter.lint_sfc(source, "ComponentValue.vue");
    Value::Array(
        result
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                json!({
                    "rule": diagnostic.rule_name,
                    "message": diagnostic.message,
                    "start": diagnostic.start,
                    "end": diagnostic.end,
                    "severity": diagnostic.severity,
                    "help": diagnostic.help,
                    "labels": diagnostic.labels.into_iter().map(|label| json!({
                        "message": label.message, "start": label.start, "end": label.end,
                    })).collect::<Vec<_>>(),
                    "fix": diagnostic.fix,
                })
            })
            .collect(),
    )
}

#[test]
fn all_authored_and_whole_pinned_component_value_cases_execute() {
    let manifest: Value = serde_json::from_str(CASES).unwrap();
    assert_eq!(manifest["schema"], "vize.lint.component-import-value-reads");
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["rules"], json!([RULE]));
    let cases = manifest["cases"].as_array().unwrap();
    let ids: Vec<_> = cases
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "passed-component-prop",
            "passed-component-in-object-value",
            "unused-setup-import",
            "comment-only-component",
            "literal-is-not-read",
            "object-key-is-not-read",
            "member-name-is-not-read",
            "for-shadow-keeps-import-unread",
            "slot-shadow-keeps-import-unread",
            "arrow-shadow-keeps-import-unread",
            "script-only-use-retained",
            "generic-capture-refusal-retained",
            "unused-options-alias",
            "used-options-alias",
            "whole-tiny-rdm",
            "whole-frappe-builder",
            "whole-vue-data-ui",
        ]
    );
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let file = case["source"].as_str().unwrap();
        let expected_file = vize_l0::cstr!("{id}.vue.txt");
        assert_eq!(file, expected_file.as_str());
        let source = std::fs::read_to_string(Path::new(ROOT).join(file)).unwrap();
        assert_eq!(source.len() as u64, case["bytes"].as_u64().unwrap());
        assert_eq!(
            whole_diagnostics(&linter, &source),
            case["expected"],
            "{id}"
        );
    }
}

#[test]
fn the_explicit_unused_binding_owner_still_reports_real_unread_imports() {
    let source =
        std::fs::read_to_string(Path::new(ROOT).join("unused-setup-import.vue.txt")).unwrap();
    let rule = "vue/no-unused-setup-bindings";
    let linter = Linter::with_registry(RuleRegistry::with_opt_in_rules())
        .with_enabled_rules(Some(vec![rule.into()]));
    assert_eq!(
        whole_diagnostics(&linter, &source),
        json!([{
            "rule": rule,
            "message": "Setup binding 'Child' is never read",
            "start": 21,
            "end": 26,
            "severity": "warning",
            "help": "Remove the binding or prefix its name with underscore if intentionally unused",
            "labels": [],
            "fix": null,
        }])
    );
    let passed =
        std::fs::read_to_string(Path::new(ROOT).join("passed-component-prop.vue.txt")).unwrap();
    assert_eq!(whole_diagnostics(&linter, &passed), json!([]));
}

#[test]
fn configured_unused_import_severity_and_disabling_remain_authoritative() {
    let source =
        std::fs::read_to_string(Path::new(ROOT).join("unused-setup-import.vue.txt")).unwrap();
    let configured = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_rule_severity_overrides(vec![(RULE.into(), vize_patina::Severity::Error)]);
    assert_eq!(
        whole_diagnostics(&configured, &source),
        json!([{
            "rule": RULE,
            "message": "Component 'Child' is registered but never used in template",
            "start": 21,
            "end": 26,
            "severity": "error",
            "help": "Remove the unused import or use the component in your template",
            "labels": [],
            "fix": null,
        }])
    );
    let disabled = configured.with_disabled_rules(vec![RULE.into()]);
    assert_eq!(whole_diagnostics(&disabled, &source), json!([]));
}

#[test]
fn a_raw_rule_context_cannot_borrow_a_captured_sfc_read() {
    use vize_patina::Rule;
    let allocator = vize_l0::Allocator::default();
    let source = "import Child from './Child.vue';";
    let template = "<Control :component=\"Child\" />";
    let (root, errors) = vize_armature::Parser::new(&allocator, template).parse();
    assert!(errors.is_empty());
    let mut drawer = vize_croquis::Drawer::for_lint().with_binding_occurrences();
    drawer.draw_script_setup(source).draw_template(&root);
    let (mut analysis, packet) = drawer.finish_with_binding_occurrences();
    assert!(packet.is_some());
    analysis.retain_authored_binding_reads(None);
    let sfc = vize_l0::cstr!("<script setup>{source}</script><template>{template}</template>");
    let descriptor = vize_atelier_sfc::parse_sfc(sfc.as_str(), Default::default()).unwrap();
    assert_eq!(sfc.as_str().get(21..26), Some("Child"));
    let mut context = vize_patina::LintContext::new(&allocator, template, "Raw.vue");
    context.set_sfc_template_descriptor(&descriptor);
    context.current_rule = RULE;
    context.set_analysis(&analysis);
    vize_patina::rules::vue::NoUnusedComponents::default().run_on_template(&mut context, &root);
    let actual: Vec<_> = context
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| {
            json!({
                "rule": diagnostic.rule_name,
                "message": diagnostic.message,
                "start": diagnostic.start,
                "end": diagnostic.end,
                "severity": diagnostic.severity,
                "help": diagnostic.help,
                    "labels": diagnostic.labels.into_iter().map(|label| json!({
                        "message": label.message, "start": label.start, "end": label.end,
                    })).collect::<Vec<_>>(),
                    "fix": diagnostic.fix,
            })
        })
        .collect();
    assert_eq!(
        actual,
        vec![json!({
            "rule": RULE,
            "message": "Component 'Child' is registered but never used in template",
            "start": 21,
            "end": 26,
            "severity": "warning",
            "help": "Remove the unused import or use the component in your template",
            "labels": [],
            "fix": null,
        })]
    );
}

#[test]
fn a_positive_read_requires_the_exact_live_declaration_and_can_be_cleared() {
    let allocator = vize_l0::Allocator::default();
    let source = "import Child from './Child.vue';";
    let template = "<Control :component=\"Child\" />";
    let (root, errors) = vize_armature::Parser::new(&allocator, template).parse();
    assert!(errors.is_empty());
    let mut drawer = vize_croquis::Drawer::for_lint().with_binding_occurrences();
    drawer.draw_script_setup(source).draw_template(&root);
    let (mut analysis, packet) = drawer.finish_with_binding_occurrences();
    let scope = analysis
        .scopes
        .iter()
        .find(|scope| scope.kind == vize_croquis::ScopeKind::ScriptSetup)
        .unwrap()
        .id;
    let (start, end) = analysis.binding_spans["Child"];
    analysis.retain_authored_binding_reads(packet);
    assert!(analysis.has_authored_template_binding_read(scope, "Child", start, end));
    assert!(!analysis.has_authored_template_binding_read(scope, "Other", start, end));
    assert!(!analysis.has_authored_template_binding_read(scope, "Child", start + 1, end));
    assert!(!analysis.has_authored_template_binding_read(scope, "Child", start, end + 1));
    let foreign_scope = analysis
        .scopes
        .iter()
        .find(|candidate| candidate.kind != vize_croquis::ScopeKind::ScriptSetup)
        .unwrap()
        .id;
    assert!(!analysis.has_authored_template_binding_read(foreign_scope, "Child", start, end));
    analysis.retain_authored_binding_reads(None);
    assert!(!analysis.has_authored_template_binding_read(scope, "Child", start, end));
}

fn assert_target_retains_actual_occurrence_without_read_credit(source: &str, id: &str) {
    let setup = source
        .strip_prefix("<script setup lang=\"ts\">\n")
        .unwrap()
        .split_once("</script>")
        .unwrap()
        .0;
    let template = source
        .split_once("<template>")
        .unwrap()
        .1
        .strip_suffix("</template>\n")
        .unwrap();
    let allocator = vize_l0::Allocator::default();
    let (root, errors) = vize_armature::Parser::new(&allocator, template).parse();
    assert!(errors.is_empty(), "{id}: actual template must parse");
    let mut drawer = vize_croquis::Drawer::for_lint().with_binding_occurrences();
    drawer.draw_script_setup(setup).draw_template(&root);
    let (mut analysis, packet) = drawer.finish_with_binding_occurrences();
    let packet = packet.expect("target controls require an admitted original packet");
    let identity = packet
        .bindings()
        .find(|binding| binding.name == "Child")
        .unwrap()
        .identity;
    assert_eq!(identity.block, OccurrenceBlock::Script, "{id}");
    assert_eq!(
        analysis.scopes.get_scope(identity.scope).unwrap().kind,
        vize_croquis::ScopeKind::ScriptSetup,
        "{id}"
    );
    assert_eq!(
        (identity.start, identity.end),
        analysis.binding_spans["Child"],
        "{id}"
    );
    assert!(
        packet.occurrences().iter().any(|reference| {
            reference.binding == identity
                && reference.block == OccurrenceBlock::Template
                && template.get(reference.start as usize..reference.end as usize) == Some("Child")
        }),
        "{id}: retain the exact original authored Child occurrence"
    );
    analysis.retain_authored_binding_reads(Some(packet));
    assert!(
        !analysis.has_authored_template_binding_read(
            identity.scope,
            "Child",
            identity.start,
            identity.end,
        ),
        "{id}: target-role expression supplies no new runtime-read proof"
    );
}

#[test]
fn runtime_reads_and_type_write_mixed_refusals_execute_independently() {
    const REFUSALS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/component-template-runtime-reads/cases.json",
    ));
    let root = Path::new(ROOT)
        .parent()
        .unwrap()
        .join("component-template-runtime-reads");
    let manifest: Value = serde_json::from_str(REFUSALS).unwrap();
    assert_eq!(
        manifest["schema"],
        "vize.lint.component-template-runtime-reads"
    );
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["rules"], json!([RULE]));
    let cases = manifest["cases"].as_array().unwrap();
    let ids: Vec<_> = cases
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "ordinary-type-cast-keeps-warning",
            "retained-type-query-keeps-warning",
            "statement-type-query-keeps-warning",
            "javascript-typeof-is-a-read",
            "javascript-typeof-statement-is-a-read",
            "write-only-template-keeps-warning",
            "mixed-arrow-type-child-runtime-other-remains-unproved",
            "mixed-arrow-runtime-child-type-other-remains-unproved",
            "mixed-statement-type-child-runtime-other-remains-unproved",
            "mixed-statement-runtime-child-type-other-remains-unproved",
            "array-target-write-only-keeps-warning",
            "array-rest-target-write-only-keeps-warning",
            "array-default-target-write-only-keeps-warning",
            "object-shorthand-target-write-only-keeps-warning",
            "object-alias-target-write-only-keeps-warning",
            "nonnull-target-write-only-keeps-warning",
            "as-target-write-only-keeps-warning",
            "satisfies-target-write-only-keeps-warning",
            "type-assertion-target-write-only-keeps-warning",
            "assignment-with-rhs-read-remains-unproved",
        ]
    );
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let source_file = case["source"].as_str().unwrap();
        let expected_file = vize_l0::cstr!("{id}.vue.txt");
        assert_eq!(source_file, expected_file.as_str());
        let source = std::fs::read_to_string(root.join(source_file)).unwrap();
        assert_eq!(source.len() as u64, case["bytes"].as_u64().unwrap());
        assert_eq!(
            whole_diagnostics(&linter, &source),
            case["expected"],
            "{id}"
        );
        if ids[10..].contains(&id) {
            assert_target_retains_actual_occurrence_without_read_credit(&source, id);
        }
    }
}
