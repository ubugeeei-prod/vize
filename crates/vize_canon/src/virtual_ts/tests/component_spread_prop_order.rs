//! The props literal emits `v-bind="obj"` spreads in template order (#3444).
//!
//! Vue 3 resolves an overlapping key by source order, last binding wins, and so
//! does an object literal. Emitting every spread first would type-check the
//! named value for a key the runtime takes from the spread instead.

use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

const FALLTHROUGH_PROBE_MARKER: &str = "const __VizeFallthroughProps_";

fn partition_spread_probes(code: &str) -> (vize_carton::String, vize_carton::String) {
    let mut primary = vize_carton::String::default();
    let mut secondary = vize_carton::String::default();
    let mut cursor = 0;
    while let Some(relative_marker) = code[cursor..].find(FALLTHROUGH_PROBE_MARKER) {
        let marker = cursor + relative_marker;
        primary.push_str(&code[cursor..marker]);
        let literal_start = marker
            + code[marker..]
                .find(" = ")
                .expect("fallthrough probe assignment")
            + 3;
        let literal_end = literal_start
            + code[literal_start..]
                .find(";\n")
                .expect("fallthrough probe terminator");
        secondary.push_str(&code[literal_start..literal_end]);
        secondary.push('\n');
        cursor = literal_end + 2;
    }
    primary.push_str(&code[cursor..]);
    (primary, secondary)
}

fn generated_props_literal(template: &str) -> vize_carton::String {
    let script = r#"import Child from "./Child.vue"
const bag = { count: 1 }
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    generate_virtual_ts(&summary, Some(script), Some(&root), 0).code
}

#[test]
fn a_spread_after_a_named_prop_is_emitted_after_it() {
    let code = generated_props_literal(r#"<Child :count="2" v-bind="bag" />"#);
    let (primary, secondary) = partition_spread_probes(&code);

    let named = primary
        .find("\"count\": 2")
        .expect("named prefix emitted as a direct property");
    let ignore = primary
        .find("// @ts-ignore Named prop diagnostics are owned by the per-prop check.")
        .expect("the mapped per-prop check owns the named prop diagnostic");
    let spread = primary.find("...bag").expect("spread emitted");
    assert!(
        named < spread,
        "the spread is authored last, so it must win the key: {code}"
    );
    assert_eq!(
        primary.matches("...{ \"count\": 2 }").count(),
        0,
        "a named prop must stay direct so its diagnostic remains property-local: {code}"
    );
    assert!(
        ignore < named,
        "the suppression must precede its entry: {code}"
    );
    assert_eq!(secondary.matches("\"count\": 2").count(), 1, "{code}");
    assert_eq!(secondary.matches("...bag").count(), 1, "{code}");
}

#[test]
fn a_bare_static_prop_before_a_spread_suppresses_the_overwrite_diagnostic() {
    let code = generated_props_literal(r#"<Child disabled v-bind="bag" />"#);
    let (primary, secondary) = partition_spread_probes(&code);

    let ignore = primary
        .find("// @ts-ignore Named prop diagnostics are owned by the per-prop check.")
        .expect("the mapped per-prop check owns the bare prop diagnostic");
    let named = primary
        .find("\"disabled\": true")
        .expect("bare static prop emitted as true");
    let spread = primary.find("...bag").expect("spread emitted");
    assert!(ignore < named && named < spread, "{code}");
    assert_eq!(
        primary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "the mapped child literal must suppress TypeScript's synthetic overwrite warning: {code}"
    );
    assert_eq!(secondary.matches("\"disabled\": true").count(), 1, "{code}");
    assert_eq!(
        secondary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "the unmapped fallthrough literal must preserve the authored shape: {code}"
    );
}

#[test]
fn a_spread_before_a_named_prop_is_emitted_before_it() {
    let code = generated_props_literal(r#"<Child v-bind="bag" :count="2" />"#);
    let (primary, secondary) = partition_spread_probes(&code);

    let spread = primary.find("...bag").expect("spread emitted");
    let named = primary.find("\"count\": 2").expect("named prop emitted");
    assert!(
        spread < named,
        "the named prop is authored last, so it must win the key: {code}"
    );
    assert_eq!(
        primary.matches("...{ \"count\": 2 }").count(),
        0,
        "a trailing named prop must remain direct: {code}"
    );
    assert_eq!(secondary.matches("...bag").count(), 1, "{code}");
    assert_eq!(secondary.matches("\"count\": 2").count(), 1, "{code}");
}

#[test]
fn each_named_segment_before_a_later_spread_stays_direct() {
    let script = r#"import Child from "./Child.vue"
const first = { count: 1 }
const second = { label: 'ok' }
"#;
    let template =
        r#"<Child :count="2" v-bind="first" label="middle" v-bind="second" tone="info" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;
    let (primary, secondary) = partition_spread_probes(&code);

    assert_eq!(primary.matches("...{ \"count\": 2 }").count(), 0, "{code}");
    assert_eq!(
        primary.matches("...{ \"label\": \"middle\" }").count(),
        0,
        "{code}"
    );
    assert_eq!(primary.matches("\"count\": 2").count(), 1, "{code}");
    assert_eq!(
        primary.matches("\"label\": \"middle\"").count(),
        1,
        "{code}"
    );
    assert_eq!(primary.matches("\"tone\": \"info\"").count(), 1, "{code}");
    assert_eq!(
        primary.matches("...{ \"tone\": \"info\" }").count(),
        0,
        "{code}"
    );
    let count = primary.find("\"count\": 2").unwrap();
    let first = primary.find("...first").unwrap();
    let label = primary.find("\"label\": \"middle\"").unwrap();
    let second = primary.find("...second").unwrap();
    let tone = primary.find("\"tone\": \"info\"").unwrap();
    assert!(
        count < first && first < label && label < second && second < tone,
        "{code}"
    );
    assert_eq!(secondary.matches("\"count\": 2").count(), 1, "{code}");
    assert_eq!(
        secondary.matches("\"label\": \"middle\"").count(),
        1,
        "{code}"
    );
    assert_eq!(secondary.matches("\"tone\": \"info\"").count(), 1, "{code}");
}

#[test]
fn dynamic_arguments_and_true_named_duplicates_are_not_disguised_as_spread_entries() {
    let script = r#"import Child from "./Child.vue"
const key = 'data-id'
"#;
    let template = r#"<Child :[key]="1" :count="1" :count="2" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;

    assert_eq!(code.matches("...{ \"count\"").count(), 0, "{code}");
    assert_eq!(code.matches("\"count\":").count(), 2, "{code}");
}

#[test]
fn duplicate_named_props_before_a_spread_stay_visible_to_typescript() {
    let code = generated_props_literal(r#"<Child :count="1" :count="2" v-bind="bag" />"#);
    let (primary, secondary) = partition_spread_probes(&code);

    // Both direct entries stay in the one outer object literal so TypeScript
    // still reports the authored duplicate (TS1117).
    assert_eq!(
        primary.matches("\"count\":").count(),
        2,
        "both duplicate entries must remain visible in the mapped literal: {code}"
    );
    assert_eq!(primary.matches("...{ \"count\"").count(), 0, "{code}");
    assert_eq!(
        primary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "only the first mapped entry can be ignored; the duplicate must stay visible to TypeScript: {code}"
    );
    assert_eq!(secondary.matches("\"count\":").count(), 2, "{code}");
    let named = primary.find("\"count\": 1").expect("named run emitted");
    let spread = primary.find("...bag").expect("spread emitted");
    assert!(named < spread, "the spread is authored last: {code}");
}

#[test]
fn duplicate_named_props_across_spreads_stay_visible_to_typescript() {
    let code =
        generated_props_literal(r#"<Child :count="1" v-bind="bag" :count="2" v-bind="bag" />"#);

    let (primary, secondary) = partition_spread_probes(&code);
    assert_eq!(primary.matches("\"count\":").count(), 2, "{code}");
    assert_eq!(secondary.matches("\"count\":").count(), 2, "{code}");
    assert_eq!(
        primary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "only the first mapped key may hide a synthetic overwrite warning; the later duplicate across a spread must stay visible to TypeScript: {code}"
    );
    assert_eq!(
        secondary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "the unmapped probe must preserve the same authored order: {code}"
    );
}

#[test]
fn duplicate_bare_props_before_a_spread_keep_the_duplicate_diagnostic() {
    let code = generated_props_literal(r#"<Child disabled disabled v-bind="bag" />"#);
    let (primary, secondary) = partition_spread_probes(&code);

    assert_eq!(primary.matches("\"disabled\": true").count(), 2, "{code}");
    assert_eq!(secondary.matches("\"disabled\": true").count(), 2, "{code}");
    assert_eq!(
        primary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "only the first mapped bare prop may hide the synthetic overwrite warning; the duplicate must remain visible to TypeScript: {code}"
    );
    assert_eq!(
        secondary
            .matches("// @ts-ignore Named prop diagnostics")
            .count(),
        1,
        "the unmapped probe must preserve the duplicate shape: {code}"
    );
}

#[test]
fn component_event_directives_do_not_enter_the_props_literal() {
    let code =
        generated_props_literal(r#"<Child @keydown="first" @keydown.esc.prevent="second" />"#);

    assert_eq!(
        code.matches("\"onKeydown\": undefined as any").count(),
        0,
        "event directives are checked by the component-event path; duplicating them in the props literal reports legal modifier pairs as TS1117: {code}"
    );
}

#[test]
fn component_events_do_not_mask_spread_listener_props() {
    let code =
        generated_props_literal(r#"<Child @keydown="first" v-bind="bag" @click="second" />"#);

    let (inference, canonical) = code
        .split_once("// Component props value checks (template scope)")
        .expect("component props section emitted");
    let (primary, secondary) = partition_spread_probes(canonical);
    assert_eq!(
        inference.matches("...bag").count(),
        1,
        "the inference path must retain the spread: {code}"
    );
    assert_eq!(
        primary.matches("...bag").count(),
        1,
        "the mapped child check must retain the spread: {code}"
    );
    assert_eq!(
        secondary.matches("...bag").count(),
        1,
        "the unmapped fallthrough probe must retain the spread: {code}"
    );
    assert_eq!(
        (
            code.matches("\"onKeydown\": undefined as any").count(),
            code.matches("\"onClick\": undefined as any").count(),
        ),
        (0, 0),
        "event directive placeholders would mask same-named listener props from spreads: {code}"
    );
}

#[test]
fn singleton_spread_keeps_exact_named_value_source_mapping() {
    let script = r#"import Child from "./Child.vue"
const bag = { count: 1, label: 'ok' }
const value = { missing: 'bad' }
"#;
    let template = r#"<Child :count="value.missing" v-bind="bag" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let (primary, secondary) = partition_spread_probes(&output.code);

    let generated_value = "\"count\": value.missing";
    let generated_missing = primary
        .find(generated_value)
        .map(|start| start + generated_value.find("missing").unwrap())
        .expect("named value emitted inside singleton spread");
    assert_eq!(
        secondary.matches(generated_value).count(),
        1,
        "{code}",
        code = output.code
    );
    let authored_missing = template.find("missing").unwrap();
    let span = output
        .mapping
        .spans()
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.gen_range.contains(&generated_missing))
        .unwrap_or_else(|| {
            panic!(
                "singleton value lost its sub-span: {:?}",
                output.mapping.spans()
            )
        });
    assert_eq!(
        generated_missing - span.gen_range.start,
        authored_missing - span.src_range.start,
    );
    assert_eq!(
        span.src_range,
        template.find("value.missing").unwrap()
            ..template.find("value.missing").unwrap() + "value.missing".len()
    );
}

#[test]
fn a_spread_rewrites_reserved_prop_references_with_exact_following_spans() {
    let script = r#"import Child from "./Child.vue"
defineProps<{ as?: string }>()
const bag = { count: 1 }
"#;
    let template = r#"<Child v-bind="{ as, count: bag.missing }" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let (primary, secondary) = partition_spread_probes(&output.code);

    let generated = "{ as: props[\"as\"], count: bag.missing }";
    let generated_start = primary.find(generated).unwrap_or_else(|| {
        panic!(
            "reserved spread reference was not rewritten:\n{}",
            output.code
        )
    });
    assert_eq!(
        primary.matches(generated).count(),
        1,
        "the mapped child check must retain the authored literal",
    );
    assert_eq!(
        secondary.matches(generated).count(),
        1,
        "the unmapped fallthrough probe must retain the authored literal",
    );
    let source_missing = template.find("missing").expect("authored member present");
    let generated_missing = generated_start + generated.find("missing").unwrap();
    let span = output
        .mapping
        .spans()
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.gen_range.contains(&generated_missing))
        .unwrap_or_else(|| {
            panic!(
                "following member must have a precise mapping: spreads={:?} mappings={:?}",
                vize_croquis::facts::component_usage_list(&summary)[0].spread_props,
                output.mapping.spans()
            )
        });

    assert_eq!(
        generated_missing - span.gen_range.start,
        source_missing - span.src_range.start,
    );
    let marker = output
        .code
        .find(FALLTHROUGH_PROBE_MARKER)
        .expect("fallthrough probe marker");
    let unmapped_missing = marker
        + output.code[marker..].find(generated).unwrap()
        + generated.find("missing").unwrap();
    let mapped_secondary_sub_spans = output
        .mapping
        .spans()
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .filter(|span| span.gen_range.contains(&unmapped_missing))
        .count();
    assert_eq!(
        mapped_secondary_sub_spans, 0,
        "the secondary fallthrough literal must stay unmapped to avoid duplicate diagnostics",
    );
}

#[test]
fn v_for_reserved_bindings_shadow_props_through_lexical_parents() {
    let script = r#"import Child from "./Child.vue"
defineProps<{ as: number }>()
const items = ["local"]
"#;
    let template = r#"<Child v-bind="{ as }" />
<Child v-for="as in items" v-bind="{ as }" />
<div v-for="as in items">
  <div v-for="item in items">
    <Child v-bind="{ as }" />
  </div>
</div>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;
    let (primary, secondary) = partition_spread_probes(&code);

    assert_eq!(
        primary.matches(r#"...{ as: props["as"] },"#).count(),
        1,
        "only the mapped root usage should fall back to the outer prop:\n{code}"
    );
    assert_eq!(
        primary.matches("...{ as },").count(),
        2,
        "mapped same-element and lexical-parent v-for locals must stay local:\n{code}"
    );
    assert_eq!(
        secondary.matches(r#"...{ as: props["as"] },"#).count(),
        1,
        "the secondary root usage must use the same binding:\n{code}"
    );
    assert_eq!(
        secondary.matches("...{ as },").count(),
        2,
        "secondary v-for locals must use the same lexical bindings:\n{code}"
    );
}

#[test]
fn v_slot_reserved_binding_shadows_the_outer_prop() {
    let script = r#"import Child from "./Child.vue"
import Provider from "./Provider.vue"
defineProps<{ as: number }>()
"#;
    let template = r#"<Provider>
  <template #default="{ as }">
    <Child v-bind="{ as }" />
  </template>
</Provider>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;
    let (primary, secondary) = partition_spread_probes(&code);

    assert_eq!(
        primary.matches("...{ as },").count(),
        1,
        "the mapped v-slot binding must stay local:\n{code}"
    );
    assert_eq!(
        secondary.matches("...{ as },").count(),
        1,
        "the unmapped v-slot binding must stay local:\n{code}"
    );
    assert_eq!(
        primary.matches(r#"...{ as: props["as"] },"#).count(),
        0,
        "the outer prop must not replace a mapped v-slot local:\n{code}"
    );
    assert_eq!(
        secondary.matches(r#"...{ as: props["as"] },"#).count(),
        0,
        "the outer prop must not replace an unmapped v-slot local:\n{code}"
    );
}
