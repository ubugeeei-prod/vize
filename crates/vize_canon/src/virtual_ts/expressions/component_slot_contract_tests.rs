use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts,
    generate_virtual_ts_with_offsets_and_checks,
};

#[test]
fn define_slots_exports_static_slot_marker_for_parents() {
    let script = r#"defineSlots<{
  default(props: { msg: string }): unknown;
}>();
"#;
    let template = r#"<slot :msg="'hello'" />"#;

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
            .contains("readonly __vizeSlots?: Partial<Slots>;"),
        "defineSlots must expose optional parent-provided slots without losing payload types:\n{}",
        output.code
    );
}

#[test]
fn required_slot_check_anchors_to_component_name() {
    let script = r#"import Child from "./Child.vue";
"#;
    let template = r#"<Child />"#;

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
            .contains("__VizeRequiredSlots<__VizeSlotContract_0, {}>"),
        "component usages must check required child slots even with no authored slots:\n{}",
        output.code
    );
    let source_start = template.find("Child").expect("component tag present");
    let source_range = source_start..source_start + "Child".len();
    let mapping = output
        .mappings
        .iter()
        .find(|mapping| {
            mapping.src_range == source_range
                && output.code[mapping.gen_range.clone()].contains("__vize_required_slots_0")
        })
        .expect("required slot diagnostic should anchor to the component name");
    assert!(
        output.code[mapping.gen_range.clone()].contains("{}"),
        "mapped range should include the failing required-slot assignment"
    );
}

#[test]
fn required_slot_check_records_static_slot_names() {
    let script = r#"import Child from "./Child.vue";
"#;
    let template = r#"<Child><template #header /></Child>"#;

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
            .contains("__VizeRequiredSlots<__VizeSlotContract_0, { readonly \"header\": true; }>"),
        "static slot names must satisfy required child slots:\n{}",
        output.code
    );
}

#[test]
fn required_slot_check_skips_open_index_signature_slot_contracts() {
    let script = r#"import OpenSlots from "./OpenSlots.vue";
"#;
    let template = r#"<OpenSlots><template #header="{ title }">{{ title }}</template></OpenSlots>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains("string extends keyof __S ? {}"),
        "open slot index signatures must not require every possible name:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("__VizeRequiredSlots<__VizeSlotContract_0, { readonly \"header\": true; }>"),
        "static parent slot names must still be recorded for finite contracts:\n{}",
        output.code
    );
}

#[test]
fn strict_slot_child_checks_are_opt_in() {
    let script = r#"import Child from "./Child.vue";
import TabItem from "./TabItem.vue";
"#;
    let template = r#"<Child><template #item><input /><TabItem /></template></Child>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        !output.code.contains("__VizeSlotChildren<"),
        "strict slot child helpers must stay out of the default output:\n{}",
        output.code
    );
}

#[test]
fn strict_slot_child_checks_emit_named_dom_and_component_children() {
    let script = r#"import Child from "./Child.vue";
import TabItem from "./TabItem.vue";
"#;
    let template = r#"<Child><template #item><input /><TabItem /></template></Child>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            experimental_strict_slot_children: true,
            ..Default::default()
        },
    );

    assert!(
        output.code.contains(
            r#"const __vize_slot_children_1_item: __VizeSlotChildren<__VizeSlotContract_1, "item"> = null as unknown as __VizeProvidedSlotChildren<[HTMLInputElement, typeof TabItem]>;"#,
        ),
        "strict slot child checks must describe named slot children:\n{}",
        output.code
    );
}

#[test]
fn strict_slot_child_checks_emit_default_slot_children() {
    let script = r#"import Child from "./Child.vue";
"#;
    let template = r#"<Child><button /></Child>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            experimental_strict_slot_children: true,
            ..Default::default()
        },
    );

    assert!(
        output.code.contains(
            r#"const __vize_slot_children_0_default: __VizeSlotChildren<__VizeSlotContract_0, "default"> = null as unknown as __VizeProvidedSlotChildren<[HTMLButtonElement]>;"#,
        ),
        "strict slot child checks must describe default slot children:\n{}",
        output.code
    );
}
