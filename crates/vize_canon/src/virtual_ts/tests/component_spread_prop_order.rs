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
