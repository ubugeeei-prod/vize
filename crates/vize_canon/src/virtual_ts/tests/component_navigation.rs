use super::generate_virtual_ts;

fn generate(script: &str, template: &str, script_setup: bool) -> super::super::VirtualTsOutput {
    use vize_croquis::{Analyzer, AnalyzerOptions};

    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(
        errors.is_empty(),
        "template should parse without errors: {errors:?}"
    );

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if script_setup {
        analyzer.analyze_script_setup(script);
    } else {
        analyzer.analyze_script_plain(script);
    }
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    generate_virtual_ts(&summary, Some(script), Some(&root), 0)
}

#[test]
fn component_template_navigation_mappings() {
    use vize_croquis::{Analyzer, AnalyzerOptions};

    let script = r#"import Child from './Child.vue'
const count = 1
"#;
    let template = r#"<Child label="ready" :count="count" />"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    let tag_start = template.find("Child").unwrap();
    let tag_mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.src_range == (tag_start..tag_start + "Child".len()))
        .expect("component tag should map to a generated component reference");
    assert_eq!(&output.code[tag_mapping.gen_range.clone()], "Child");

    let static_prop_start = template.find("label").unwrap();
    let static_prop_mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.src_range == (static_prop_start..static_prop_start + "label".len()))
        .expect("static prop name should map to a generated prop type reference");
    assert_eq!(&output.code[static_prop_mapping.gen_range.clone()], "label");

    let dynamic_prop_start = template.find(":count").unwrap() + 1;
    let dynamic_prop_mapping = output
        .mappings
        .iter()
        .find(|mapping| {
            mapping.src_range == (dynamic_prop_start..dynamic_prop_start + "count".len())
        })
        .expect("dynamic prop name should map to a generated prop type reference");
    assert_eq!(
        &output.code[dynamic_prop_mapping.gen_range.clone()],
        "count"
    );
}

#[test]
fn non_ascii_template_string_literals_are_preserved() {
    let script = r#"import { defineComponent } from '@nuxtjs/composition-api';

const NAVIGATION_ITEMS = [
  { header: 'アカウント' },
  { header: 'コンテンツ管理' },
] as const;

export default defineComponent({
  setup() {
    return { NAVIGATION_ITEMS };
  },
});
"#;
    let template = r#"<template v-for="menu in NAVIGATION_ITEMS">
  <v-list-item :disabled="menu.header === 'アカウント'">
    {{ menu.header }}
  </v-list-item>
</template>"#;

    let output = generate(script, template, false);

    assert!(output.code.contains("menu.header === 'アカウント'"));
    assert!(output.code.contains("header: 'コンテンツ管理'"));
    assert!(
        !output.code.contains("ã"),
        "virtual TS must not contain mojibake:\n{}",
        output.code
    );
}

#[test]
fn non_ascii_template_string_literals_survive_comment_stripping() {
    let script = r#"const menu = { header: 'アカウント' }"#;
    let template =
        r#"<div :title="/* leading */ menu.header === 'アカウント' ? '選択中' : '通常'"></div>"#;

    let output = generate(script, template, true);

    assert!(output.code.contains("menu.header === 'アカウント'"));
    assert!(output.code.contains("'選択中'"));
    assert!(output.code.contains("'通常'"));
    assert!(
        !output.code.contains("ã"),
        "comment stripping must keep UTF-8 literals intact:\n{}",
        output.code
    );
}
