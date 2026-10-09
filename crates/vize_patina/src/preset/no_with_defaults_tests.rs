//! Preset activation and unchanged macro ownership for withDefaults.

use super::LintPreset;
use crate::Linter;

#[test]
fn opinionated_reports_with_defaults_without_explicit_selection() {
    let source = r#"<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 });
</script>"#;
    let start = source.find("withDefaults").unwrap() as u32;
    for preset in LintPreset::ALL {
        let result = Linter::with_preset(preset).lint_sfc(source, "Counter.vue");
        let findings: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "script/no-with-defaults")
            .map(|diagnostic| (diagnostic.severity, diagnostic.start, diagnostic.end))
            .collect();
        let expected = if preset == LintPreset::Opinionated {
            vec![(crate::Severity::Warning, start, start + 12)]
        } else {
            vec![]
        };
        assert_eq!(findings, expected, "{}", preset.as_str());
    }
    assert!(
        Linter::new()
            .lint_sfc(source, "Counter.vue")
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "script/no-with-defaults")
    );
}

#[test]
fn opinionated_preserves_with_defaults_macro_ownership() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    for source in [
        r#"<script setup lang="ts">
const { count = 0 } = defineProps<{ count?: number }>();
</script>"#,
        r#"<script setup lang="ts">
// withDefaults(defineProps<{ count?: number }>(), { count: 0 });
const note = "withDefaults(defineProps(), {})";
function withDefaults(value: number) { return value; }
const count = withDefaults(0);
</script>"#,
        r#"<script lang="ts">
const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 });
</script>"#,
    ] {
        let result = linter.lint_sfc(source, "Counter.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| { diagnostic.rule_name != "script/no-with-defaults" }),
            "{:?}",
            result.diagnostics
        );
    }
    let result = linter.lint_script(
        "const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 });",
        "counter.ts",
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.rule_name != "script/no-with-defaults" }),
        "{:?}",
        result.diagnostics
    );
}
