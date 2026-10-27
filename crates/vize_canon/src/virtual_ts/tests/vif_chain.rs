use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn interrupted_sibling_chain_keeps_exclusions_outside_the_remaining_branches() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/content-mapper/escaton-nested-flow-9433.json"
    ))
    .unwrap();
    let source = fixture["originals"][0]["files"]["App.vue"]
        .as_str()
        .unwrap();
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let script = descriptor.script_setup.as_ref().unwrap().content.as_ref();
    let template = descriptor.template.as_ref().unwrap().content.as_ref();
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    // The original Vue SFC uses two self-closing non-void HTML elements.
    // Standard parsing retains their exact compatibility notices; no other
    // notice or error may pass this admission check.
    let notice = "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.";
    assert_eq!(
        errors
            .iter()
            .map(|error| (
                error.code,
                error.message.as_str(),
                error.loc.as_ref().map(|loc| (loc.span.start, loc.span.end)),
                error.is_recoverable(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                vize_armature::ErrorCode::ExtendPoint,
                notice,
                Some((33, 50)),
                true
            ),
            (
                vize_armature::ErrorCode::ExtendPoint,
                notice,
                Some((64, 96)),
                true
            ),
        ],
        "the original parser packet must remain complete: {errors:?}"
    );
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    let start = output.code.find("  if (!(view === 'a')) {").unwrap();
    let end = start
        + output
            .code
            .get(start..)
            .unwrap()
            .find("\n  // Reference setup bindings")
            .unwrap();
    assert_eq!(
        output.code.get(start..end),
        Some(concat!(
            "  if (!(view === 'a')) {\n",
            "    if (view === 'b') {\n",
            "      // @vize-map: expr -> 80:92\n",
            "    } else {\n",
            "      void (checked); // VModel\n",
            "      // @vize-map: expr -> 132:139\n",
            "    }\n",
            "  }\n",
        )),
        "the whole remaining chain must reuse the prior exclusions without rechecking them"
    );
    assert!(output.mapping.spans().iter().any(|mapping| {
        template.get(mapping.src_range.clone()) == Some("view === 'b'")
            && output.code.get(mapping.gen_range.clone()) == Some("view === 'b'")
    }));
    assert!(output.mapping.spans().iter().any(|mapping| {
        template.get(mapping.src_range.clone()) == Some("checked")
            && output.code.get(mapping.gen_range.clone()) == Some("checked")
    }));
}

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
            .contains("for (const [aimAtReport] of __vize_v_for_source_"),
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

    // `attribute` is read before the loop binds it, so it is an unresolved
    // template read: declared from the public instance, never from the alias.
    assert!(
        output.code.lines().any(|line| {
            line == "  var attribute!: __VizeTemplateInstanceMember<\"attribute\">; void (attribute);"
        }),
        "same-element v-if must be checked before its v-for aliases exist:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("for (const [attribute, index] of __vize_v_for_source_"),
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

    assert!(
        output.code.matches("else {").count() == 1
            && output.code.matches("!(item.type === 'hunk-bar')").count() <= 1,
        "v-else rechecked the discriminant:\n{}",
        output.code
    );
}

#[test]
fn nested_v_for_in_else_if_reuses_the_active_cumulative_guard() {
    let script = r#"import MyCell from './MyCell.vue'
const mode: 'a' | 'b' | 'c' = 'b'
const rows = [[1, 2]]
"#;
    let template = r#"<p v-if="mode === 'a'">a</p>
<table v-else-if="mode === 'b'">
  <tr v-for="(row, i) in rows" :key="i">
    <td v-for="cell in row" :key="cell"><MyCell :label="String(cell)" /></td>
  </tr>
</table>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let checks = output
        .code
        .split_once("// Component props value checks (template scope)")
        .expect("component props checks")
        .1;

    assert_eq!(
        checks.matches("if (!(mode === 'a'))").count()
            + checks.matches("if ((mode === 'b'))").count(),
        2,
        "the outer loop's branch guard must remain active for its nested loop:\n{checks}"
    );
    assert_eq!(
        checks.matches("// Component props in v-for scope:").count(),
        2,
        "both loop environments must still be replayed:\n{checks}"
    );
}

#[test]
fn nested_v_for_in_else_reuses_the_active_cumulative_guard() {
    let script = r#"import MyCell from './MyCell.vue'
const mode: 'a' | 'b' = 'b'
const rows = [[1, 2]]
"#;
    let template = r#"<p v-if="mode === 'a'">a</p>
<table v-else>
  <tr v-for="(row, i) in rows" :key="i">
    <td v-for="cell in row" :key="cell"><MyCell :label="String(cell)" /></td>
  </tr>
</table>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let checks = output
        .code
        .split_once("// Component props value checks (template scope)")
        .expect("component props checks")
        .1;
    let loop_scopes = checks
        .match_indices("// Component props in v-for scope:")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();

    assert_eq!(
        loop_scopes.len(),
        2,
        "both loop environments must still be replayed:\n{checks}"
    );
    assert_eq!(
        checks[..loop_scopes[0]]
            .matches("if (!(mode === 'a'))")
            .count(),
        1,
        "the outer loop must retain the v-else branch guard:\n{checks}"
    );
    assert_eq!(
        checks[loop_scopes[0]..loop_scopes[1]]
            .matches("if (!(mode === 'a'))")
            .count(),
        0,
        "the nested loop must not reopen the active v-else branch guard:\n{checks}"
    );
}

#[test]
fn component_listeners_share_linear_v_else_if_branch_guards() {
    let script = r#"import { reactive } from 'vue'
import KeyPad from './KeyPad.vue'
type Kind = 'k0' | 'k1' | 'k2' | 'k3'
const state = reactive<{ mode: 'edit' | 'view'; kind: Kind }>({ mode: 'edit', kind: 'k0' })
const onPress = (key: string) => key
const onClear = () => undefined
const onSubmit = (value: number) => value
"#;
    let mut template = String::from("<div>\n");
    for index in 0..4 {
        let directive = if index == 0 { "v-if" } else { "v-else-if" };
        template.push_str(
            format!(
                "  <template {directive}=\"state.mode === 'edit' && state.kind === 'k{index}'\">\n    <KeyPad label=\"a{index}\" @press=\"onPress\" @clear=\"onClear\" @submit=\"onSubmit\" />\n    <KeyPad label=\"b{index}\" @press=\"onPress\" @clear=\"onClear\" @submit=\"onSubmit\" />\n  </template>\n"
            )
            .as_str(),
        );
    }
    template.push_str("  <span v-else>none</span>\n</div>");

    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template.as_str());
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let listener_start = output.code.find("  // @press handler").unwrap_or(0);
    let listener_end = output
        .code
        .find("  // Component props type declarations")
        .unwrap_or(output.code.len());
    let navigation_start = output
        .code
        .find("// @ts-ignore Navigation-only guard")
        .unwrap();
    let navigation_end = output.code[navigation_start..]
        .find("  if (state.mode === 'edit' && state.kind === 'k0') {")
        .map(|offset| navigation_start + offset)
        .unwrap();
    let navigation = &output.code[navigation_start..navigation_end];
    let listeners = &output.code[listener_start..listener_end];

    assert_eq!(
        navigation
            .matches("// @ts-ignore Navigation-only guard")
            .count(),
        4,
        "one linear branch chain must own all event navigation references: {navigation}"
    );
    for index in 0..4 {
        let condition = format!("state.kind === 'k{index}'");
        assert_eq!(
            navigation.matches(condition.as_str()).count(),
            1,
            "navigation must emit each branch condition once: {condition}\n{navigation}"
        );
    }

    assert_eq!(
        listeners.matches("// @press handler").count(),
        8,
        "{listeners}"
    );
    assert_eq!(
        listeners.matches("// @clear handler").count(),
        8,
        "{listeners}"
    );
    assert_eq!(
        listeners.matches("// @submit handler").count(),
        8,
        "{listeners}"
    );
    for index in 0..4 {
        let condition = format!("state.kind === 'k{index}'");
        assert!(
            listeners.matches(condition.as_str()).count() <= 7,
            "each branch condition is emitted once in the shared chain and at most once per listener callback: {condition}\n{listeners}"
        );
    }
    assert_eq!(
        listeners
            .matches("// @ts-ignore Listener branch guard")
            .count(),
        3,
        "the three else-if continuations must share the listener branch chain: {listeners}"
    );
}

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
    output
        .code
        .find("type __MyChild_Component_1 = __MyChild_Component_0;")
        .expect("later usage reuses the first component contract");
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
    assert_eq!(
        output.code.matches("_prop_args_library =").count(),
        2,
        "select and close should each resolve their stable listener contract once: {}",
        output.code
    );
}

#[test]
fn guarded_sibling_listeners_own_their_lexical_contracts() {
    let script = r#"import { ref } from 'vue'
import MyButton from './MyButton.vue'
import MyMenu from './MyMenu.vue'
const shown = ref(true)
const open = () => undefined
const save = () => undefined
"#;
    let template = r#"<div v-if="shown">
  <MyButton @click="open">open</MyButton>
  <MyMenu>
    <template #activator><span>more</span></template>
    <MyButton @click="save">save</MyButton>
  </MyMenu>
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
        output.code.matches("_click_prop_args_library =").count(),
        2,
        "each sibling generated guard must declare the stable listener contract it references: {}",
        output.code
    );
    assert_eq!(
        output.code.matches("_click_prop_arrays =").count(),
        2,
        "array-listener aliases must stay inside their generated guard: {}",
        output.code
    );
}
