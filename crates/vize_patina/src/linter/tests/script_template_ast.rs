//! Cross-block script rules consume only the template pass's valid evidence.

use super::Linter;

const RULE: &str = "script/no-use-computed-property-like-method";
const SCRIPT: &str = "<!-- 日本語 -->\n<script>export default { computed: { total() { return 1 } }, methods: { read() { return this.total() } } }</script>\n";

#[test]
fn shared_template_ast_preserves_script_findings_and_offsets() {
    for (template, template_finding, parse_error) in [
        ("<template><div>{{ total() }}</div></template>", true, false),
        (
            "<template><div id=\"a\"class=\"b\">{{ total() }}</div></template>",
            true,
            true,
        ),
        (
            "<template><div><span>{{ total() }}</div></template>",
            false,
            true,
        ),
        ("", false, false),
    ] {
        let source = format!("{SCRIPT}{template}");
        for type_aware in [false, true] {
            let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
            let result = if type_aware {
                // Exercise the native driver's shared-AST handoff without a
                // Corsa process: this fixture enables no type-querying rules.
                super::super::native_type_aware::lint_sfc_with_corsa(&linter, &source, "shared.vue")
            } else {
                linter.lint_sfc(&source, "shared.vue")
            };
            let diagnostics: Vec<_> = result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == RULE)
                .collect();
            assert_eq!(
                diagnostics.len(),
                1 + usize::from(template_finding),
                "{type_aware}: {template}: {:?}",
                result.diagnostics
            );
            assert_eq!(
                &source[diagnostics[0].start as usize..diagnostics[0].end as usize],
                "this.total()"
            );
            if template_finding {
                assert_eq!(
                    &source[diagnostics[1].start as usize..diagnostics[1].end as usize],
                    "total()"
                );
                assert!(diagnostics[1].start as usize >= SCRIPT.len());
            }
            assert_eq!(
                result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.rule_name == "parser/template"),
                parse_error,
                "{type_aware}: {template}"
            );
        }
    }
}
