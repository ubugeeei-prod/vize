//! #12995: enum comparison facts are invariant between classic and setup scripts.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintDiagnostic, LintPreset, Linter, Severity};

const RULE: &str = "type/no-unsafe-enum-comparison";
const CASE: &str = r#"enum Fruit {
  Apple = 'apple',
}
type __String =
  | (string & { __escapedIdentifier: void })
  | (void & { __escapedIdentifier: void })
  | Fruit;
declare const weirdString: __String;
weirdString === 'someArbitraryValue';"#;
const MESSAGE: &str = "The two values in this comparison do not have a shared enum type.";

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]))
}

fn diagnostics(source: &str, filename: &str) -> Vec<LintDiagnostic> {
    lint_sfc_with_corsa(&linter(), source, filename)
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .collect()
}

fn asserted_packet(source: &str, diagnostic: &LintDiagnostic) {
    let start = u32::try_from(source.find("weirdString ===").expect("comparison"))
        .expect("comparison offset");
    let end = start + u32::try_from("weirdString === 'someArbitraryValue'".len()).unwrap();
    assert_eq!(diagnostic.rule_name, RULE);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(diagnostic.message, MESSAGE);
    assert_eq!((diagnostic.start, diagnostic.end), (start, end));
    assert_eq!(diagnostic.help, None);
    assert!(diagnostic.labels.is_empty());
    assert!(diagnostic.fix.is_none());
}

#[test]
fn branded_string_enum_comparison_has_the_same_packet_in_both_script_modes() {
    if !corsa_available() {
        return;
    }
    for (filename, source) in [
        (
            "Classic.vue",
            format!("<script lang=\"ts\">\n{CASE}\nexport {{}};\n</script>"),
        ),
        (
            "Setup.vue",
            format!("<script setup lang=\"ts\">\n{CASE}\n</script>"),
        ),
    ] {
        let actual = diagnostics(&source, filename);
        assert_eq!(actual.len(), 1, "{filename}: {actual:#?}");
        asserted_packet(&source, &actual[0]);
    }
}

#[test]
fn branded_string_controls_do_not_invent_enum_mismatches() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
enum Fruit {
  Apple = 'apple',
}
type BrandedString = string & { __escapedIdentifier: void };
declare const branded: BrandedString;
declare const fruitOrBrand: Fruit | BrandedString;
branded === 'someArbitraryValue';
fruitOrBrand === Fruit.Apple;
</script>"#;
    assert!(
        diagnostics(source, "Controls.vue").is_empty(),
        "negative controls must remain silent"
    );
}
