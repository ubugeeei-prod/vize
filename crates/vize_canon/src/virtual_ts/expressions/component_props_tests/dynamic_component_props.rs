use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

#[test]
fn dynamic_component_props_use_resolved_setup_binding() {
    let script = r#"import Child from "./Child.vue"
const comp = Child
const count = "nope"
"#;
    let template = r#"<component :is="comp" :count="count" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        output.code.contains("type __comp_Props_0 = typeof comp"),
        "dynamic component props should resolve against the :is binding:\n{}",
        output.code
    );
    assert!(
        output.code.contains(r#""count": count"#),
        "dynamic component props should include authored child props:\n{}",
        output.code
    );
    assert!(
        !output.code.contains(r#""is": comp"#),
        "the runtime-only :is binding must not be checked as a child prop:\n{}",
        output.code
    );
}
