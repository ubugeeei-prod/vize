use super::*;
use crate::{DrawerOptions, binding_occurrences::BindingOccurrence};
use vize_relief::Allocator;

const SCRIPT: &str =
    "const label = 'Name';\nconst hint = 'Shown below the field';\nconst id = 'field';";
const TEMPLATE: &str = "<label :for=\"id\">{{ label }}</label>\n<input :id=\"id\" />\n<p :id=\"`${id}-hint`\">{{ hint }}</p>";

fn capture(script: &str, template: &str) -> BindingOccurrences {
    let allocator = Allocator::default();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty());
    let mut drawer = Drawer::with_options(DrawerOptions::for_lint()).with_binding_occurrences();
    drawer.draw_script_setup(script);
    drawer.draw_template(&root);
    drawer.finish_with_binding_occurrences().1.unwrap()
}

fn references<'a>(packet: &'a BindingOccurrences, name: &str) -> Vec<&'a BindingOccurrence> {
    let binding = packet
        .bindings()
        .find(|binding| binding.name == name && binding.identity.block == OccurrenceBlock::Script)
        .unwrap();
    packet
        .occurrences()
        .iter()
        .filter(|occurrence| occurrence.binding == binding.identity)
        .collect()
}

#[test]
fn entire_reported_template_has_only_owned_binding_reads() {
    let packet = capture(SCRIPT, TEMPLATE);
    assert_eq!(references(&packet, "label").len(), 1);
    assert_eq!(references(&packet, "hint").len(), 1);
    assert_eq!(references(&packet, "id").len(), 3);
    for name in ["label", "hint", "id"] {
        let binding = packet
            .bindings()
            .find(|binding| {
                binding.name == name && binding.identity.block == OccurrenceBlock::Script
            })
            .unwrap();
        assert_eq!(
            SCRIPT
                .get(binding.identity.start as usize..binding.identity.end as usize)
                .expect("binding range stays within authored UTF-8 boundaries"),
            name
        );
        for reference in references(&packet, name) {
            assert_eq!(reference.block, OccurrenceBlock::Template);
            assert_eq!(
                TEMPLATE
                    .get(reference.start as usize..reference.end as usize)
                    .expect("reference range stays within authored UTF-8 boundaries"),
                name
            );
        }
    }
    assert_eq!(
        references(&packet, "label")[0].start as usize,
        TEMPLATE.find("{{ label").unwrap() + 3
    );
    assert_eq!(
        references(&packet, "hint")[0].start as usize,
        TEMPLATE.find("{{ hint").unwrap() + 3
    );
    assert!(references(&packet, "id").iter().all(|reference| {
        !TEMPLATE
            .get(..reference.start as usize)
            .expect("reference prefix stays within authored UTF-8 boundaries")
            .ends_with(':')
    }));
}

#[test]
fn repeat_cache_keeps_each_location_and_lexical_owner() {
    let template = "<p>{{ id }}</p><p v-for=\"id in [1]\">{{ id }}</p><p>{{ id }}</p>";
    let packet = capture(SCRIPT, template);
    let reads = references(&packet, "id");
    assert_eq!(reads.len(), 2);
    assert_eq!(reads[0].start as usize, template.find("{{ id").unwrap() + 3);
    assert_eq!(
        reads[1].start as usize,
        template.rfind("{{ id").unwrap() + 3
    );
    assert_ne!(reads[0].start, reads[1].start);
}

#[test]
fn property_keys_strings_comments_and_expression_locals_are_not_global_reads() {
    let template = "<p :title=\"({ id: label, text: 'hint', [id]: hint })\">{{ /* id */ label }}{{ ((id) => id)(1) }}</p>";
    let packet = capture(SCRIPT, template);
    assert_eq!(references(&packet, "label").len(), 2);
    assert_eq!(references(&packet, "hint").len(), 1);
    assert_eq!(references(&packet, "id").len(), 1);
    let id = references(&packet, "id")[0];
    assert_eq!(id.start as usize, template.find("[id]").unwrap() + 1);
    let label = references(&packet, "label")[1];
    assert_eq!(
        label.start as usize,
        template.find("/* id */ label").unwrap() + 9
    );
}

#[test]
fn unicode_and_crlf_keep_authored_byte_offsets_before_utf16_projection() {
    let script = "// 日本語😀\r\nconst 名称 = 'name';\r\n";
    let template = "<p title=\"名称😀\">\r\n{{ 名称 }}\r\n</p>";
    let packet = capture(script, template);
    let reads = references(&packet, "名称");
    assert_eq!(reads.len(), 1);
    assert_eq!(
        reads[0].start as usize,
        template.find("{{ 名称").unwrap() + 3
    );
    assert_eq!(reads[0].end - reads[0].start, "名称".len() as u32);
    let declaration = packet
        .bindings()
        .find(|binding| binding.name == "名称")
        .unwrap();
    assert_eq!(
        declaration.identity.start as usize,
        script.find("名称 =").unwrap()
    );
}

#[test]
fn capture_leaves_ordinary_semantic_snapshot_unchanged() {
    let allocator = Allocator::default();
    let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
    assert!(errors.is_empty());
    let mut ordinary = Drawer::with_options(DrawerOptions::for_lint());
    ordinary.draw_script_setup(SCRIPT).draw_template(&root);
    let expected = serde_json::to_value(ordinary.finish().semantic_snapshot()).unwrap();
    let mut captured = Drawer::with_options(DrawerOptions::for_lint()).with_binding_occurrences();
    captured.draw_script_setup(SCRIPT);
    captured.draw_template(&root);
    let (croquis, packet) = captured.finish_with_binding_occurrences();
    assert_eq!(
        serde_json::to_value(croquis.semantic_snapshot()).unwrap(),
        expected
    );
    assert_eq!(packet.unwrap().occurrences().len(), 5);
}

#[test]
fn ordinary_capture_storage_is_empty_and_pointer_sized() {
    use crate::script_parser::ScriptParseResult;
    use std::mem::{size_of, size_of_val};

    assert_eq!(
        size_of::<Option<Box<super::OccurrenceCapture>>>(),
        size_of::<usize>()
    );
    let parsed = ScriptParseResult::default();
    assert_eq!(size_of_val(&parsed.occurrence_capture), size_of::<usize>());
    assert!(parsed.occurrence_capture.is_none());
    let mut ordinary = Drawer::with_options(DrawerOptions::for_lint());
    assert!(ordinary.occurrence_capture.is_none());
    ordinary.draw_script_setup(SCRIPT);
    assert!(ordinary.occurrence_capture.is_none());
    assert!(
        ordinary
            .with_binding_occurrences()
            .occurrence_capture
            .is_some()
    );
}

#[test]
fn full_analysis_keeps_occurrences_explicit_and_complete_public_outputs_equal() {
    let allocator = Allocator::default();
    let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
    assert!(errors.is_empty());
    let mut ordinary = Drawer::with_options(DrawerOptions::full());
    assert!(ordinary.occurrence_capture.is_none());
    ordinary.draw_script_setup(SCRIPT).draw_template(&root);
    assert!(ordinary.occurrence_capture.is_none());
    let mut expected = ordinary.finish();
    assert!(expected.setup_context.take_ssr_occurrences().is_none());
    let mut captured = Drawer::with_options(DrawerOptions::full()).with_binding_occurrences();
    captured.draw_script_setup(SCRIPT).draw_template(&root);
    let (actual, packet) = captured.finish_with_binding_occurrences();
    assert_eq!(actual.to_vir(), expected.to_vir());
    assert_eq!(
        serde_json::to_value(actual.semantic_snapshot()).unwrap(),
        serde_json::to_value(expected.semantic_snapshot()).unwrap()
    );
    assert_eq!(packet.unwrap().occurrences().len(), 5);
}
