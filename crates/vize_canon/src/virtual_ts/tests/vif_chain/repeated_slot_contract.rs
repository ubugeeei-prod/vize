//! A `v-if` / `v-else-if` chain keeps one structural slot contract.
use super::{Analyzer, AnalyzerOptions};
use crate::virtual_ts::{
    generator::generate_virtual_ts_with_offsets_and_checks,
    types::{VirtualTsGenerationOptions, VirtualTsOptions},
};
use vize_carton::cstr;

#[test]
fn repeated_slot_contract_is_emitted_once() {
    let script = "import KeyPad from './KeyPad.vue'\nconst kind = 'k0'\nconst a0 = 'a0'\nconst b0 = 'b0'\nconst a1 = 'a1'\nconst b1 = 'b1'\nconst onPress = (key: string) => key\n";
    let template = "<div><template v-if=\"kind === 'k0'\"><KeyPad :label=\"a0\" @press=\"onPress\" /><KeyPad :label=\"b0\" @press=\"onPress\" /></template><template v-else-if=\"kind === 'k1'\"><KeyPad :label=\"a1\" @press=\"onPress\" /><KeyPad :label=\"b1\" @press=\"onPress\" /></template></div>";
    let tag_ranges = keypad_ranges(template);
    let name_ranges = label_name_ranges(template);
    let mut tag_anchors = Vec::new();
    let mut prop_anchors = Vec::new();
    for hoist in [false, true] {
        let output = generated(script, template, hoist);
        assert_eq!(
            output
                .code
                .matches("$StructuralSlots<typeof KeyPad>")
                .count(),
            1,
            "one component emits the slot contract once; hoist={hoist}: {}",
            output.code
        );
        let declaration = if hoist {
            "type __KeyPad_slot_contract = import(\"vize:virtual-ts\").$StructuralSlots<typeof KeyPad>;"
        } else {
            "type __KeyPad_slot_contract = $StructuralSlots<typeof KeyPad>;"
        };
        assert_eq!(
            output.code.matches(declaration).count(),
            1,
            "the shared slot contract is declared once; hoist={hoist}: {}",
            output.code
        );
        for idx in 0..4 {
            let alias = cstr!("type $SlotContract_{idx} = __KeyPad_slot_contract;");
            assert!(
                output.code.contains(alias.as_str()),
                "use {idx} aliases the first slot contract; hoist={hoist}: {}",
                output.code
            );
            let required = if hoist {
                cstr!(
                    "const __vize_required_slots_{idx}: import(\"vize:virtual-ts\").$RequiredSlots<$SlotContract_{idx}, {{}}> = {{}};"
                )
            } else {
                cstr!(
                    "const __vize_required_slots_{idx}: $RequiredSlots<$SlotContract_{idx}, {{}}> = {{}};"
                )
            };
            assert!(
                output.code.contains(required.as_str()),
                "use {idx} keeps its required-slots const; hoist={hoist}: {}",
                output.code
            );
        }
        assert_eq!(
            output.code.matches("const __vize_required_slots_").count(),
            4,
            "each use still checks its own slots; hoist={hoist}: {}",
            output.code
        );
        assert!(
            output.code.contains("kind === 'k0'") && output.code.contains("kind === 'k1'"),
            "both branch conditions stay; hoist={hoist}"
        );
        let mut found_tags = Vec::new();
        for (idx, tag_range) in tag_ranges.iter().enumerate() {
            let check = cstr!("__vize_required_slots_{idx}");
            let mapping = output
                .mapping
                .spans()
                .iter()
                .find(|mapping| {
                    mapping.src_range == *tag_range
                        && output.code[mapping.gen_range.clone()].contains(check.as_str())
                })
                .expect("slot diagnostic anchors at the component tag");
            found_tags.push(mapping.src_range.clone());
        }
        let mut found_props = Vec::new();
        for (idx, name_range) in name_ranges.iter().enumerate() {
            let name_span = output
                .mapping
                .spans()
                .iter()
                .flat_map(|mapping| &mapping.sub_spans)
                .find(|span| span.src_range == *name_range)
                .expect("prop diagnostic anchors at the attribute name");
            assert_eq!(
                &output.code[name_span.gen_range.clone()],
                cstr!("__vize_prop_check_{idx}_label").as_str(),
                "TS2322 stays on the per-usage check; hoist={hoist}"
            );
            found_props.push(name_range.clone());
        }
        if tag_anchors.is_empty() {
            tag_anchors = found_tags;
            prop_anchors = found_props;
        } else {
            assert_eq!(
                tag_anchors, found_tags,
                "hoisting must not move slot diagnostic anchors"
            );
            assert_eq!(
                prop_anchors, found_props,
                "hoisting must not move prop diagnostic anchors"
            );
        }
    }
}

fn keypad_ranges(template: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(at) = template[from..].find("<KeyPad") {
        let start = from + at + 1;
        ranges.push(start..start + "KeyPad".len());
        from = start + "KeyPad".len();
    }
    assert_eq!(ranges.len(), 4, "four KeyPad tags");
    ranges
}

fn label_name_ranges(template: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(at) = template[from..].find(":label") {
        let start = from + at + 1;
        ranges.push(start..start + "label".len());
        from = start + "label".len();
    }
    assert_eq!(ranges.len(), 4, "four label anchors");
    ranges
}

fn generated(
    script: &str,
    template: &str,
    hoist: bool,
) -> crate::virtual_ts::types::VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
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
            hoist_shared_preamble: hoist,
            ..Default::default()
        },
    )
}
