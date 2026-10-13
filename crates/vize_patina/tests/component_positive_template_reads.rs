//! Positive imported component reads remain valid independently of editor completeness.

use serde_json::Value;
use std::path::Path;
use vize_patina::Linter;

const RULE: &str = "vue/no-unused-components";
const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-positive-template-reads/",
);
const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-positive-template-reads/cases.json",
));

#[test]
fn real_runtime_import_reads_and_unread_or_shadowed_controls_execute() {
    let manifest: Value = serde_json::from_str(CASES).unwrap();
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    for case in manifest["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let source =
            std::fs::read_to_string(Path::new(ROOT).join(case["source"].as_str().unwrap()))
                .unwrap();
        let result = linter.lint_sfc(&source, "PositiveRead.vue");
        assert_eq!(result.error_count, 0, "{id}: {:?}", result.diagnostics);
        assert_eq!(
            result.warning_count as u64,
            case["expectedWarnings"].as_u64().unwrap(),
            "{id}: {:?}",
            result.diagnostics,
        );
        for diagnostic in result.diagnostics {
            assert_eq!(diagnostic.rule_name, RULE, "{id}");
            assert_eq!(
                source.get(diagnostic.start as usize..diagnostic.end as usize),
                Some("Child"),
                "{id}: negative controls retain the original import range",
            );
        }
    }
}

#[test]
fn positive_lint_reads_never_make_incomplete_editor_packets_complete() {
    for id in ["unrelated-if", "unrelated-try", "unrelated-function-throw"] {
        let source =
            std::fs::read_to_string(Path::new(ROOT).join(vize_l0::cstr!("{id}.vue.txt").as_str()))
                .unwrap();
        let descriptor = vize_atelier_sfc::parse_sfc(&source, Default::default()).unwrap();
        let allocator = vize_l0::Allocator::default();
        let (root, errors) = vize_armature::parse(
            &allocator,
            descriptor.template.as_ref().unwrap().content.as_ref(),
        );
        assert!(errors.is_empty(), "{id}");
        let (_, packet) = vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_occurrences(
            &descriptor,
            Some(&root),
            vize_atelier_sfc::croquis::SfcCroquisOptions::lint_demand(),
        );
        assert!(
            packet.is_none(),
            "{id}: editor completeness must stay refused"
        );
    }
}

#[test]
fn foreign_derived_generic_and_external_views_cannot_qualify_positive_import_reads() {
    use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_occurrences};
    let original = "<script setup>import Child from './Child.vue'; if(true){console.log(1)}</script><template><Control :component=\"Child\" /></template>";
    let generic = "<script setup generic=\"T\">import Child from './Child.vue';</script><template><Control :component=\"Child\" /></template>";
    for (source, template, derived, external) in [
        (original, "<Control :component=\"Other\" />", false, false),
        (original, "<Control :component=\"Child\" />", true, false),
        (generic, "<Control :component=\"Child\" />", false, false),
        (original, "<Control :component=\"Child\" />", false, true),
    ] {
        let mut descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        if external {
            descriptor.script_setup.as_mut().unwrap().src = Some("./component.js".into());
        }
        let allocator = vize_l0::Allocator::default();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty());
        let options = if derived {
            SfcCroquisOptions::lint_demand().with_derived_template()
        } else {
            SfcCroquisOptions::lint_demand()
        };
        let (analysis, _) =
            analyze_sfc_descriptor_with_occurrences(&descriptor, Some(&root), options);
        assert!(!analysis.croquis.authored_component_reads_are_qualified());
    }
}
