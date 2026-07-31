//! The props literal emits `v-bind="obj"` spreads in template order (#3444).
//!
//! Vue 3 resolves an overlapping key by source order, last binding wins, and so
//! does an object literal. Emitting every spread first would type-check the
//! named value for a key the runtime takes from the spread instead.

use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generated_props_literal(template: &str) -> vize_carton::String {
    let script = r#"import Child from "./Child.vue"
const bag = { count: 1 }
"#;
    let allocator = vize_carton::Bump::new();
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

    let named = code.find("\"count\": 2").expect("named prop emitted");
    let spread = code.find("...bag").expect("spread emitted");
    assert!(
        named < spread,
        "the spread is authored last, so it must win the key: {code}"
    );
}

#[test]
fn a_spread_before_a_named_prop_is_emitted_before_it() {
    let code = generated_props_literal(r#"<Child v-bind="bag" :count="2" />"#);

    let spread = code.find("...bag").expect("spread emitted");
    let named = code.find("\"count\": 2").expect("named prop emitted");
    assert!(
        spread < named,
        "the named prop is authored last, so it must win the key: {code}"
    );
}

#[test]
fn a_spread_rewrites_reserved_prop_references_with_exact_following_spans() {
    let script = r#"import Child from "./Child.vue"
defineProps<{ as?: string }>()
const bag = { count: 1 }
"#;
    let template = r#"<Child v-bind="{ as, count: bag.missing }" />"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    let generated = "{ as: props[\"as\"], count: bag.missing }";
    let generated_start = output.code.rfind(generated).unwrap_or_else(|| {
        panic!(
            "reserved spread reference was not rewritten:\n{}",
            output.code
        )
    });
    assert_eq!(
        output.code.matches(generated).count(),
        1,
        "a component spread must be checked only by the mapped props call",
    );
    let source_missing = template.find("missing").expect("authored member present");
    let generated_missing = generated_start + generated.find("missing").unwrap();
    let span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.gen_range.contains(&generated_missing))
        .unwrap_or_else(|| {
            panic!(
                "following member must have a precise mapping: spreads={:?} mappings={:?}",
                summary.component_usages[0].spread_props, output.mappings
            )
        });

    assert_eq!(
        generated_missing - span.gen_range.start,
        source_missing - span.src_range.start,
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
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;

    assert_eq!(
        code.matches(r#"...{ as: props["as"] },"#).count(),
        1,
        "only the root usage should fall back to the outer prop:\n{code}"
    );
    assert_eq!(
        code.matches("...{ as },").count(),
        2,
        "same-element and lexical-parent v-for locals must stay local:\n{code}"
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
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;

    assert!(
        code.contains("...{ as },"),
        "v-slot local must stay a local reference:\n{code}"
    );
    assert!(
        !code.contains(r#"...{ as: props["as"] },"#),
        "the outer prop must not replace a visible v-slot local:\n{code}"
    );
}
