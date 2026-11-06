//! Shared usages keep only the aliases their actual downstream checks read.
use super::{Analyzer, AnalyzerOptions, generate_virtual_ts};
use crate::virtual_ts::{
    generator::generate_virtual_ts_with_offsets_and_checks,
    types::{VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions},
};
use vize_carton::cstr;

#[test]
fn repeated_component_usages_share_prop_and_listener_contracts() {
    let script = r#"import MyChild from './MyChild.vue'
const first = 'first'
const second = 'second'
const onSelect = (value: number) => value
const onClose = () => undefined
"#;
    let template = r#"<div>
  <MyChild :label="first" @select="onSelect" @close="onClose" />
  <MyChild :label="second" @select="onSelect" @close="onClose" />
</div>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert_eq!(
        output.code.matches(" = typeof MyChild;\n").count(),
        1,
        "the component constructor contract must be extracted once: {}",
        output.code
    );
    for suffix in ["Component", "CheckProps", "CheckTail"] {
        let declaration = vize_carton::cstr!("type __MyChild_{suffix}_1");
        assert_eq!(
            output
                .code
                .lines()
                .filter(|line| line.split_once('=').map(|(left, _)| left.trim())
                    == Some(declaration.trim()))
                .count(),
            0,
            "unused shared declaration: {}",
            output.code
        );
    }
    output
        .code
        .find("type __MyChild_Check_1 = __MyChild_Check_0;")
        .expect("later whole-props check reuses the first resolved checker");
    assert_eq!(
        output.code.matches("const __vize_emit_props_").count(),
        2,
        "each usage should resolve its props once for all of its listeners: {}",
        output.code
    );
    // One component contract derives the child's props, emit props and
    // library shape once; each event then reads them for its own key.
    for alias in [
        "__library = typeof MyChild ",
        "__props = typeof MyChild ",
        "__emit_props = typeof MyChild ",
    ] {
        assert_eq!(
            output.code.matches(alias).count(),
            1,
            "{alias} should be declared once for both events: {}",
            output.code
        );
    }
    assert_eq!(
        output
            .code
            .matches("typeof MyChild extends { new (")
            .count(),
        1,
        "the constructor props are inferred once per component contract: {}",
        output.code
    );
    for event in ["select", "close"] {
        assert_eq!(
            output.code.matches(&*cstr!("_{event}_prop_args =")).count(),
            1,
            "{event} should resolve its stable listener contract once: {}",
            output.code
        );
    }
}

#[test]
fn shared_aliases_follow_actual_model_and_spread_reads() {
    let script = "import MyChild from './MyChild.vue'\nlet value = 'value'\nconst attrs = { label: 'label' }\n";
    // Attribute shape, strict model, typed spread, emit checks, exact read count.
    for (attributes, strict_model, typed_spread, emits, reads) in [
        (":label=\"value\"", false, false, true, 0),
        ("v-model=\"value\"", false, false, true, 0),
        ("v-model=\"value\"", true, false, true, 1),
        ("v-model=\"value\"", true, false, false, 0),
        ("v-bind=\"attrs\"", false, false, true, 0),
        ("v-bind=\"attrs\"", false, true, true, 1),
        ("v-model=\"value\" v-bind=\"attrs\"", true, true, true, 2),
    ] {
        for dynamic in [false, true] {
            for hoist in [false, true] {
                let tag = if dynamic {
                    "component :is=\"MyChild\""
                } else {
                    "MyChild"
                };
                let end_tag = if dynamic { "component" } else { "MyChild" };
                let template = cstr!(
                    "<div><{tag} {attributes}></{end_tag}><{tag} {attributes}></{end_tag}></div>"
                );
                let allocator = vize_carton::Allocator::new();
                let (root, errors) = vize_armature::parse(&allocator, template.as_str());
                assert!(errors.is_empty(), "{errors:?}");
                let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
                analyzer.analyze_script_setup(script);
                analyzer.analyze_template(&root);
                let summary = analyzer.finish();
                let usages = vize_croquis::facts::component_usage_list(&summary);
                assert_eq!(usages.len(), 2);
                assert_eq!(usages[0].name, usages[1].name);
                let name = crate::virtual_ts::helpers::to_safe_identifier_fragment(
                    usages[1].name.as_str(),
                );
                let output = generate_virtual_ts_with_offsets_and_checks(
                    &summary,
                    Some(script),
                    Some(&root),
                    0,
                    0,
                    &VirtualTsOptions::default(),
                    VirtualTsGenerationOptions {
                        check_options: VirtualTsCheckOptions {
                            strict_v_model: strict_model,
                            fallthrough_attributes: typed_spread,
                            check_unknown_props: false,
                            check_emits: emits,
                            ..Default::default()
                        },
                        hoist_shared_preamble: hoist,
                        local_component_ownership_helpers: hoist,
                        ..Default::default()
                    },
                );
                let alias = cstr!("__{name}_Component_1");
                let declaration = cstr!("  type {alias} = __{name}_Component_0;");
                let actual = output
                    .code
                    .lines()
                    .filter(|line| {
                        line.split_once('=').map(|(left, _)| left.trim())
                            == Some(cstr!("type {alias}").as_str())
                    })
                    .collect::<Vec<_>>();
                let expected = if reads == 0 {
                    vec![]
                } else {
                    vec![declaration.as_str()]
                };
                assert_eq!(
                    actual, expected,
                    "{attributes}; dynamic={dynamic}; hoist={hoist}: {}",
                    output.code
                );
                assert_eq!(
                    output.code.matches(alias.as_str()).count(),
                    if reads == 0 { 0 } else { reads + 1 },
                    "all actual readers must retain the exact per-usage owner: {}",
                    output.code
                );
                for suffix in ["CheckProps", "CheckTail"] {
                    let prefix = cstr!("type __{name}_{suffix}_1");
                    assert_eq!(
                        output
                            .code
                            .lines()
                            .filter(|line| line.split_once('=').map(|(left, _)| left.trim())
                                == Some(prefix.trim()))
                            .count(),
                        0,
                        "{}",
                        output.code
                    );
                }
                let checker = cstr!("  type __{name}_Check_1 = __{name}_Check_0;");
                let check_prefix = cstr!("type __{name}_Check_1");
                assert_eq!(
                    output
                        .code
                        .lines()
                        .filter(|line| line.split_once('=').map(|(left, _)| left.trim())
                            == Some(check_prefix.as_str()))
                        .collect::<Vec<_>>(),
                    vec![checker.as_str()],
                    "{}",
                    output.code
                );
            }
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta records complete prop projections and every mapping field"
)]
fn repeated_prop_value_type_is_emitted_once() {
    let script = "import MyChild from './MyChild.vue'\nconst first = 'first'\nconst second = 'second'\nconst extra = 1\n";
    let template = "<div><MyChild :label=\"first\" /><MyChild :label=\"second\" /><MyChild :label=\"() => extra\" /></div>";
    let name_ranges = label_name_ranges(template);
    let value_ranges = ["first", "second", "() => extra"]
        .into_iter()
        .map(|value| {
            let start = template.find(value).expect("authored prop value");
            start..start + value.len()
        })
        .collect::<Vec<_>>();
    let mut anchors = Vec::new();
    let mut outputs = Vec::new();
    for hoist in [false, true] {
        let output = generated(script, template, hoist);
        let prop_value = if hoist {
            "import(\"vize:virtual-ts\").$PropValue"
        } else {
            "$PropValue"
        };
        let shared_type = cstr!(
            "{prop_value}<__MyChild_ValueProps_0, 'label', __MyChild_FallthroughValue_0<'label'>>"
        );
        assert_eq!(
            output.code.matches(shared_type.as_str()).count(),
            1,
            "one component emits the prop value type once; hoist={hoist}: {}",
            output.code
        );
        assert_eq!(
            output
                .code
                .lines()
                .filter(
                    |line| line.trim() == "type __MyChild_1_prop_label = __MyChild_0_prop_label;"
                )
                .count(),
            1,
            "the second static use aliases the shared prop type; hoist={hoist}: {}",
            output.code
        );
        let callable = if hoist {
            "import(\"vize:virtual-ts\").$CallableProp<"
        } else {
            "$CallableProp<"
        };
        let callable_alias =
            cstr!("type __MyChild_2_prop_label = {callable}__MyChild_0_prop_label>;");
        assert_eq!(
            output
                .code
                .lines()
                .filter(|line| line.trim() == callable_alias.as_str())
                .count(),
            1,
            "an inline callback keeps its own wrapper; hoist={hoist}: {}",
            output.code
        );
        assert_eq!(
            output
                .code
                .lines()
                .filter(
                    |line| line.trim() == "type __MyChild_2_prop_label = __MyChild_0_prop_label;"
                )
                .count(),
            0,
            "a callback must not reuse the static prop type; hoist={hoist}: {}",
            output.code
        );
        let mut found = Vec::new();
        for (idx, (name_range, value_range)) in name_ranges.iter().zip(&value_ranges).enumerate() {
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
            assert!(
                output
                    .mapping
                    .spans()
                    .iter()
                    .flat_map(|mapping| &mapping.sub_spans)
                    .any(|span| span.src_range == *value_range),
                "the value diagnostic keeps its authored anchor; hoist={hoist}: {value_range:?}"
            );
            let prop_mapping = output
                .mapping
                .spans()
                .iter()
                .find(|mapping| {
                    mapping
                        .sub_spans
                        .iter()
                        .any(|span| span.src_range == *name_range)
                })
                .expect("prop mapping");
            found.push((
                prop_mapping.src_range.clone(),
                name_range.clone(),
                value_range.clone(),
            ));
        }
        if anchors.is_empty() {
            anchors = found;
        } else {
            assert_eq!(anchors, found, "hoisting must not move diagnostic anchors");
        }
        outputs.push((hoist, output));
    }
    insta::assert_debug_snapshot!("repeated_prop_value_projection", outputs);
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta records complete emit projections and every mapping field"
)]
fn repeated_emit_resolver_is_emitted_once() {
    let script = "import KeyPad from './KeyPad.vue'\nconst kind = 'k0'\nconst a0 = 'a0'\nconst b0 = 'b0'\nconst a1 = 'a1'\nconst b1 = 'b1'\nconst onPress = (key: string) => key\n";
    let template = "<div><template v-if=\"kind === 'k0'\"><KeyPad :label=\"a0\" @press=\"onPress\" /><KeyPad :label=\"b0\" @press=\"onPress\" /></template><template v-else-if=\"kind === 'k1'\"><KeyPad :label=\"a1\" @press=\"onPress\" /><KeyPad :label=\"b1\" @press=\"onPress\" /></template></div>";
    let values = ["a0", "b0", "a1", "b1"];
    let mut name_ranges = Vec::new();
    let mut from = 0;
    while let Some(at) = template[from..].find(":label") {
        let start = from + at + 1;
        name_ranges.push(start..start + "label".len());
        from = start + "label".len();
    }
    assert_eq!(name_ranges.len(), 4, "four label anchors");
    let value_ranges = values
        .into_iter()
        .map(|value| {
            let start = template.find(value).expect("authored prop value");
            start..start + value.len()
        })
        .collect::<Vec<_>>();
    let mut anchors = Vec::new();
    let mut outputs = Vec::new();
    for hoist in [false, true] {
        let output = generated(script, template, hoist);
        let factory = if hoist {
            "import(\"vize:virtual-ts\").$EmitPropsFactory<typeof KeyPad>"
        } else {
            "$EmitPropsFactory<typeof KeyPad>"
        };
        assert_eq!(
            output.code.matches(factory).count(),
            1,
            "one component emits the emit-resolver instance alias once; hoist={hoist}: {}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches("type __KeyPad_0_emit_resolver = ")
                .count(),
            1,
            "the first visible resolver is declared once; hoist={hoist}: {}",
            output.code
        );
        for idx in 1..4 {
            let alias = cstr!("type __KeyPad_{idx}_emit_resolver = __KeyPad_0_emit_resolver;");
            assert_eq!(
                output
                    .code
                    .lines()
                    .filter(|line| line.trim() == alias.as_str())
                    .count(),
                1,
                "later use {idx} aliases the first visible resolver; hoist={hoist}: {}",
                output.code
            );
        }
        assert_eq!(
            output.code.matches("const __vize_emit_props_").count(),
            4,
            "each use still resolves its own emit props; hoist={hoist}: {}",
            output.code
        );
        let mut found = Vec::new();
        for (idx, (name_range, value_range)) in name_ranges.iter().zip(&value_ranges).enumerate() {
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
            assert!(
                output
                    .mapping
                    .spans()
                    .iter()
                    .flat_map(|mapping| &mapping.sub_spans)
                    .any(|span| span.src_range == *value_range),
                "the value diagnostic keeps its authored anchor; hoist={hoist}: {value_range:?}"
            );
            let prop_mapping = output
                .mapping
                .spans()
                .iter()
                .find(|mapping| {
                    mapping
                        .sub_spans
                        .iter()
                        .any(|span| span.src_range == *name_range)
                })
                .expect("prop mapping");
            found.push((
                prop_mapping.src_range.clone(),
                name_range.clone(),
                value_range.clone(),
            ));
        }
        if anchors.is_empty() {
            anchors = found;
        } else {
            assert_eq!(anchors, found, "hoisting must not move diagnostic anchors");
        }
        outputs.push((hoist, output));
    }
    insta::assert_debug_snapshot!("repeated_emit_resolver_projection", outputs);
}

fn label_name_ranges(template: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(at) = template[from..].find(":label") {
        let start = from + at + 1;
        ranges.push(start..start + "label".len());
        from = start + "label".len();
    }
    assert_eq!(ranges.len(), 3, "three label anchors");
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
            local_component_ownership_helpers: hoist,
            ..Default::default()
        },
    )
}
