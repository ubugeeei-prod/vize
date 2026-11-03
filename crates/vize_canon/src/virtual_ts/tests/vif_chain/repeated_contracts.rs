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
    for alias in ["__library =", "__props =", "__emit_props ="] {
        assert_eq!(
            output
                .code
                .lines()
                .filter(
                    |line| line.trim_start().starts_with("type __MyChild_") && line.contains(alias)
                )
                .count(),
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
