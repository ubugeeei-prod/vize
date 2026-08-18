use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_prefixed_v_else_branch_reuses_the_enclosing_v_for_guard() {
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

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("if ((isOpen)) {\n\n    // v-for scope"),
        "the common outer guard should wrap the complete v-for:\n{}",
        output.code
    );
    assert!(
        output.code.contains("if (item.kind === 'page') {")
            && output.code.contains("} else {")
            && !output.code.contains("(isOpen) && !(item.kind === 'page')"),
        "the inner chain should reuse the active guard and preserve a plain else branch:\n{}",
        output.code
    );
}

#[test]
fn test_v_for_enclosing_guard_does_not_reference_own_alias() {
    let script = r#"type GuidanceTest =
  | { testResult: 'Pass'; retestFlag: boolean; passOnly: string }
  | { testResult: 'Fail'; retestFlag?: false; reason: string }
type AimAtReport =
  | { kind: 'summary'; title: string }
  | { kind: 'history'; title: string; guidanceTests: GuidanceTest[] }
const aimAtReports: AimAtReport[] = []
"#;
    let template = r#"<template v-for="aimAtReport in aimAtReports">
  <template v-if="aimAtReport.kind === 'summary'">
    <span>{{ aimAtReport.title }}</span>
  </template>
  <template v-else>
    <template v-for="guidanceTest in aimAtReport.guidanceTests">
      <span v-if="guidanceTest.retestFlag">{{ aimAtReport.title }}</span>
      <span v-if="guidanceTest.testResult === 'Pass'">{{ guidanceTest.passOnly }}</span>
    </template>
  </template>
</template>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("__vForList(aimAtReports).forEach(([aimAtReport]) => {"),
        "expected parent v-for to declare aimAtReport before branch guards:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("if ((aimAtReport.kind === 'summary')) {\n\n    // v-for scope: aimAtReport in aimAtReports"),
        "v-for enclosing guard must not reference its own alias before declaration:\n{}",
        output.code
    );
}

#[test]
fn same_element_v_if_does_not_see_v_for_alias() {
    let script = r#"const menuList = [{ attributes: ["size"] }];"#;
    let template = r#"<section v-for="foods in menuList">
  <ul v-if="foods.attributes.length">
    <li v-if="attribute" v-for="(attribute, index) in foods.attributes" :key="index">
      {{ attribute }}
    </li>
  </ul>
</section>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("\n  // Undefined references from template:\n  void (attribute);\n"),
        "same-element v-if must be checked before its v-for aliases exist:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("__vForList(foods.attributes).forEach(([attribute, index]) => {"),
        "the v-for body should still bind aliases for descendants:\n{}",
        output.code
    );
}

#[test]
fn nested_else_v_for_does_not_recheck_narrowed_discriminant() {
    let script = r#"interface BarItem { type: 'hunk-bar'; lines: number }
interface LineItem { type: 'line'; text: string }
type ViewItem = BarItem | { type: 'section'; lines: LineItem[] }
const items: ViewItem[] = []
"#;
    let template = r#"<template v-for="(item, i) in items" :key="i">
  <button v-if="item.type === 'hunk-bar'">{{ item.lines }}</button>
  <div v-else>
    <div v-for="(row, j) in item.lines" :key="j">{{ row.text }}</div>
  </div>
</template>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert_eq!(
        output.code.matches("!(item.type === 'hunk-bar')").count(),
        1,
        "the v-else branch already narrows item; nested scopes must not compare it again:\n{}",
        output.code
    );
}
