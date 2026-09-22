//! Every committed rule fixture and construct is replayed through lint_sfc.

use super::compare_template;
use crate::markup::differential::{TEMPLATES, scan_rule_fixtures};

#[test]
fn sfc_facade_matches_relief_on_the_committed_template_battery() {
    for (_, source) in TEMPLATES {
        compare_template(source);
    }
    for (_, source) in scan_rule_fixtures().templates {
        compare_template(&source);
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/davinci-matrix");
    let mut paths: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vue"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 90);
    for path in paths {
        let source = std::fs::read_to_string(path).unwrap();
        let descriptor = vize_atelier_sfc::parse_sfc(&source, Default::default()).unwrap();
        compare_template(&descriptor.template.unwrap().content);
    }
}
