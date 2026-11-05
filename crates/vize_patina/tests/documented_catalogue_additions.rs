//! Independent whole reports captured from the immutable v0.691.0 public CLI.

use super::{configuration, configure, example, reference_root};
use serde_json::Value;
use vize_patina::{OutputFormat, format_results};

#[test]
fn newly_documented_rule_examples_preserve_all_twenty_four_public_report_packets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "fixtures/documented-catalogue-additions/reports.json"
    ))
    .unwrap();
    assert_eq!(corpus["baselineRelease"], "v0.691.0");
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let page =
            std::fs::read_to_string(reference_root().join(format!("{}.md", id.replace('/', "-"))))
                .unwrap();
        let linter = configure(id, &configuration(&page));
        for (heading, key) in [("## Bad", "bad"), ("## Good", "good")] {
            let (source, language, filename) = example(&page, heading, id);
            assert_eq!(language, "vue", "audited SFC examples: {id}");
            let result = linter.lint_sfc(&source, &filename);
            let output = format_results(&[result], &[(filename, source)], OutputFormat::Json);
            assert_eq!(
                serde_json::from_str::<Value>(&output).unwrap(),
                case[key],
                "{id} {heading}: complete public report; no fields dropped or projected"
            );
        }
    }
}
