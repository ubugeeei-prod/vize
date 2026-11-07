//! The case table is shared with
//! `tests/tooling/canon-implied-guard-equivalence.test.ts`, which proves each
//! rewrite narrows exactly as the authored guard does.
use super::drop_implied_exclusions;

#[test]
fn rewrites_follow_the_shared_case_table() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("cases.json")).unwrap();
    let cases = cases.as_array().unwrap();
    assert_eq!(cases.len(), 19);
    for case in cases {
        let guard = case["guard"].as_str().unwrap();
        assert_eq!(
            drop_implied_exclusions(guard).as_deref(),
            case["simplified"].as_str(),
            "{guard}"
        );
    }
}

#[test]
fn slot_hosts_under_a_v_else_if_chain_keep_only_their_own_test() {
    let script = r#"import Pick from './Pick.vue'
type Item = { type: 'root' } | { type: 'text'; text: string } | { type: 'select'; items: string[] }
const c = { type: 'select', items: [] } as Item
"#;
    let template = r#"<div>
  <span v-if="c.type === 'root'"></span>
  <span v-else-if="c.type === 'text'">{{ c.text }}</span>
  <Pick v-else-if="c.type === 'select'" :items="c.items"><template #label="{ item }">{{ item }}</template></Pick>
</div>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    for hoist_shared_preamble in [false, true] {
        let code = crate::virtual_ts::generate_virtual_ts_with_offsets_and_checks(
            &summary,
            Some(script),
            Some(&root),
            0,
            0,
            &crate::virtual_ts::VirtualTsOptions::default(),
            crate::virtual_ts::VirtualTsGenerationOptions {
                hoist_shared_preamble,
                ..Default::default()
            },
        )
        .code;
        // The slot host restates the guard for the constructor and for the
        // factory; the props pass reads that host instead of declaring one.
        assert_eq!(
            code.matches("((c.type === 'select')) ? {").count(),
            2,
            "{code}"
        );
        assert_eq!(
            code.matches(
                "!(c.type === 'root') && !(c.type === 'text') && (c.type === 'select')) ? {"
            )
            .count(),
            0,
            "{code}"
        );
    }
}
