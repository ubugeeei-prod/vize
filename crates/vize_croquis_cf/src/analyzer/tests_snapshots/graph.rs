use super::*;

#[test]
fn test_snapshot_dependency_graph() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Create a complex dependency graph
    let mut comp_a = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    comp_a.analyze_script_setup(
        r#"import { provide } from 'vue'
provide('a', 1)"#,
    );
    comp_a
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("CompB"));
    comp_a
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("CompC"));

    let mut comp_b = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    comp_b.analyze_script_setup(
        r#"import { inject, provide } from 'vue'
const a = inject('a')
provide('b', 2)"#,
    );
    comp_b
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("CompD"));

    let mut comp_c = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    comp_c.analyze_script_setup(
        r#"import { inject } from 'vue'
const a = inject('a')"#,
    );
    comp_c
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("CompD"));

    let mut comp_d = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    comp_d.analyze_script_setup(
        r#"import { inject } from 'vue'
const b = inject('b')"#,
    );

    analyzer.add_file_with_analysis(Path::new("CompA.vue"), "", comp_a.finish());
    analyzer.add_file_with_analysis(Path::new("CompB.vue"), "", comp_b.finish());
    analyzer.add_file_with_analysis(Path::new("CompC.vue"), "", comp_c.finish());
    analyzer.add_file_with_analysis(Path::new("CompD.vue"), "", comp_d.finish());

    analyzer.rebuild_component_edges();

    // Build graph output
    let mut output = String::new();
    output.push_str("=== Dependency Graph ===\n\n");

    let mut nodes: Vec<_> = analyzer.graph().nodes().collect();
    nodes.sort_by(|a, b| a.path.cmp(&b.path));

    for node in nodes {
        append!(output, "Node: {}\n", node.path);
        append!(output, "  component_name: {:?}\n", node.component_name);
        append!(output, "  is_entry: {}\n", node.is_entry);
        append!(output, "  imports: {:?}\n", node.imports);
        output.push('\n');
    }

    assert_snapshot!(output);
}
