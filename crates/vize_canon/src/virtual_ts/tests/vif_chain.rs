use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_prefixed_v_else_branch_keeps_negated_branch_guard() {
    let script = r#"const isOpen = true
type Item =
  | { kind: 'anchor'; hash: string; key: string }
  | { kind: 'page'; to: string; key: string }
const navItems: Item[] = []
"#;
    let template = r#"<div v-if="isOpen">
  <div v-for="item in navItems" :key="item.key">
    <span v-if="item.kind === 'page'">{{ item.to }}</span>
    <span v-else>{{ item.hash }}</span>
  </div>
</div>"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("} else if ((isOpen) && !(item.kind === 'page')) {"),
        "prefixed v-else branch should retain the negated discriminant guard:\n{}",
        output.code
    );
}
