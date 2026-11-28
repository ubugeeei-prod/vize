//! Generated reads that TypeScript options and authored conditions decide.

use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_carton::String;
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate(script: &str, template: &str, check_options: VirtualTsCheckOptions) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            check_options,
            ..Default::default()
        },
    )
    .code
}

fn lines_with<'a>(code: &'a str, needle: &str) -> Vec<&'a str> {
    code.lines()
        .map(str::trim)
        .filter(|line| line.contains(needle))
        .collect()
}

fn listener_reads(code: &str) -> Vec<&str> {
    lines_with(code, "_events_nav_0")
        .into_iter()
        .filter(|line| line.starts_with("void "))
        .collect()
}

#[test]
fn listener_navigation_uses_element_access_only_under_no_property_access_from_index_signature() {
    let script = "import Child from './Child.vue'\nfunction onClick() {}\n";
    let template = r#"<Child @click="onClick" @focus-in="onClick" />"#;
    let dot = generate(script, template, VirtualTsCheckOptions::default());
    assert_eq!(
        listener_reads(&dot),
        vec![
            "void __vize_events_nav_0.click;",
            "void __vize_kebab_events_nav_0.focusIn;",
        ]
    );
    let element = generate(
        script,
        template,
        VirtualTsCheckOptions {
            no_property_access_from_index_signature: true,
            ..Default::default()
        },
    );
    assert_eq!(
        listener_reads(&element),
        vec![
            "void __vize_events_nav_0['click'];",
            "void __vize_kebab_events_nav_0['focusIn'];",
        ]
    );
}

#[test]
fn slot_conditions_read_the_declared_slot_their_branch_renders() {
    let script = "const slots = defineSlots<{ tip: () => unknown; note: () => unknown; other: () => unknown }>()\n";
    let template = r#"<div>
  <span v-if="slots.tip"><slot name="tip" /></span>
  <p v-if="slots.other">none</p>
  <p v-else-if="slots.note"><slot name="note" /></p>
</div>"#;
    let code = generate(script, template, VirtualTsCheckOptions::default());
    assert_eq!(
        lines_with(&code, "slots."),
        vec![
            "if ((slots.tip)) {",
            "void slots.tip;",
            "if (slots.other) {",
            "} else if (slots.note) {",
            "void slots.note;",
        ]
    );
}
