//! Analysis-dependent rules must keep running on recovered template parses.
//!
//! Found while hydrating the pinned `vue2-elm` fixture (#3294): a recovered
//! attribute boundary (`version="1.1"class="x"`) used to classify the whole
//! parse as fatal, so croquis analysis was skipped and every
//! `has_analysis`-guarded rule silently reported nothing. A recovered parse
//! yields a complete tree, and it is exactly the shape where a silent rule
//! skip is least acceptable.

use super::Linter;
use vize_s0::ToCompactString;

fn rule_names(result: &crate::linter::config::LintResult) -> Vec<&'static str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect()
}

#[test]
fn recovered_attribute_boundary_still_runs_analysis_rules() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    // The missing whitespace between `version="1.1"` and `class="x"` is
    // reported (MissingWhitespaceBetweenAttributes) and recovered with an
    // inferred attribute boundary; the unused v-for `index` must still be
    // seen by the analysis-guarded rule on that recovered tree.
    let source = "<template><div version=\"1.1\"class=\"x\">\
<span v-for=\"(item, index) in items\" :key=\"item\">{{ item }}</span>\
</div></template>";
    let result = linter.lint_sfc(source, "test.vue");
    let rules = rule_names(&result);

    assert!(rules.contains(&"parser/template"), "{rules:?}");
    assert!(rules.contains(&"vue/no-unused-vars"), "{rules:?}");
}

#[test]
fn recovered_parse_and_clean_parse_report_the_same_analysis_rules() {
    // The formatted twin of the recovered source (whitespace restored) must
    // agree with it on every non-parser rule, so pristine and formatted
    // trees lint identically (#3294's lint-agreement property).
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let recovered = "<template><div version=\"1.1\"class=\"x\">\
<span v-for=\"(item, index) in items\" :key=\"item\">{{ item }}</span>\
</div></template>";
    let clean = "<template><div version=\"1.1\" class=\"x\">\
<span v-for=\"(item, index) in items\" :key=\"item\">{{ item }}</span>\
</div></template>";

    let mut recovered_rules = rule_names(&linter.lint_sfc(recovered, "test.vue"));
    recovered_rules.retain(|rule| *rule != "parser/template");
    let clean_rules = rule_names(&linter.lint_sfc(clean, "test.vue"));

    assert_eq!(recovered_rules, clean_rules);
    assert_eq!(recovered_rules, ["vue/no-unused-vars"]);
}

#[test]
fn unrecovered_parse_errors_still_disable_analysis_rules() {
    // A truly broken template (no documented recovery) keeps the previous
    // behavior: parse diagnostics only, no analysis-dependent output.
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source =
        "<template><div><span v-for=\"(item, index) in items\">{{ item }}</span></template>";
    let result = linter.lint_sfc(source, "test.vue");
    let rules = rule_names(&result);

    assert!(!rules.contains(&"vue/no-unused-vars"), "{rules:?}");
}
