//! Many sibling v-if / v-for branches must not reprint each cumulative guard.

use vize_carton::{String, append};
use vize_croquis::{Analyzer, AnalyzerOptions};

const BRANCHES: usize = 40;

fn project(script: &str, template: &str) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    crate::virtual_ts::generate_virtual_ts(&summary, Some(script), Some(&root), 0).code
}

#[test]
fn many_vif_vfor_branches_do_not_repeat_cumulative_guards() {
    let script = "\
import Cell from './Cell.vue'
type Row = { kind: string; cells: string[] }
const rows: Row[] = []
";
    let mut template = String::from("<section v-for=\"row in rows\" :key=\"row.kind\">\n");
    for index in 0..BRANCHES {
        let directive = if index == 0 { "v-if" } else { "v-else-if" };
        append!(
            template,
            "  <template {directive}=\"row.kind === 'k{index}'\">\n"
        );
        template.push_str("    <span v-for=\"cell in row.cells\" :key=\"cell\">");
        template.push_str("{{ cell }}<Cell :label=\"cell\" /></span>\n");
        template.push_str("  </template>\n");
    }
    template.push_str("</section>\n");

    let code = project(script, template.as_str());
    let k0 = code.matches("row.kind === 'k0'").count();
    let k0_in_budget = (1..=8).contains(&k0);
    assert!(
        k0_in_budget,
        "branch 0 is repeated {k0} times in {} bytes; a cumulative guard would repeat it once per later branch",
        code.len()
    );
    let repeated_negations = code
        .split("!(row.kind === 'k0') && !(row.kind === 'k1') && !(row.kind === 'k2')")
        .nth(1)
        .is_some();
    assert!(
        !repeated_negations,
        "sibling v-for wrappers still repeat negated branches ({} bytes)",
        code.len()
    );
    for index in 0..BRANCHES {
        let mut needle = String::from("row.kind === 'k");
        append!(needle, "{index}'");
        let present = code.split(needle.as_str()).nth(1).is_some();
        assert!(present, "dropped the authored check {needle}");
    }
    assert_eq!(
        code.matches("// v-for scope: cell in row.cells").count(),
        BRANCHES,
        "every inner loop must still be projected"
    );
    assert_eq!(
        code.matches("// Component props in v-for scope: cell in row.cells")
            .count(),
        BRANCHES,
        "prop replay must still enter every inner loop"
    );
    assert!(
        code.matches("__vize_prop_check_").count() >= BRANCHES,
        "component prop checks must stay inside each branch"
    );
    let alias_tested_early = code.split("if ((cell").nth(1).is_some();
    assert!(
        !alias_tested_early,
        "a v-for alias must not be tested before it is bound"
    );
}
