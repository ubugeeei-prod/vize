use super::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn slot_payload_preserves_the_actual_value_provider_and_missing_registry_fallback() {
    let template =
        r#"<WitnessChild><template #default="{ item }">{{ item }}</template></WitnessChild>"#;
    for (script, provider) in [
        (
            "import WitnessChild from './component';",
            Some("WitnessChild"),
        ),
        ("import type { WitnessChild } from './component';", None),
        ("", None),
    ] {
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let scope = summary
            .scopes
            .iter()
            .find_map(|scope| {
                matches!(scope.data(), vize_croquis::ScopeData::VSlot(_))
                    .then_some(scope.id.as_u32())
            })
            .expect("one authored slot scope");
        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script),
            Some(&root),
            0,
            0,
            &Default::default(),
        );
        let payloads = output
            .code
            .lines()
            .filter_map(|line| {
                line.split_once(": $SlotPayload<")
                    .map(|(_, suffix)| vize_carton::String::from(suffix))
            })
            .collect::<Vec<_>>();
        let provider = provider.map_or_else(
            || vize_carton::String::from(
                ", $RegisteredSlotProvider<import(\"vue\").GlobalComponents, \"WitnessChild\", \"WitnessChild\", \"WitnessChild\">",
            ),
            |provider| vize_carton::cstr!(", typeof {provider}"),
        );
        assert_eq!(
            payloads,
            vec![
                vize_carton::cstr!("typeof __vize_slot_host_{scope}, \"default\"{provider}>) {{"),
                vize_carton::cstr!(
                    "typeof __vize_slot_props_host_{scope}, \"default\"{provider}>) {{"
                ),
            ],
            "complete payload annotations for {script}"
        );
    }
}
