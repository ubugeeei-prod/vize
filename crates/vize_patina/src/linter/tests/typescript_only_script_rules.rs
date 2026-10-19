//! Rules that ask for type syntax stay silent in JavaScript blocks.

use super::Linter;

const RULES: [&str; 3] = [
    "script/define-props-declaration",
    "script/define-emits-declaration",
    "script/require-typed-ref",
];

fn linter() -> Linter {
    Linter::new().with_enabled_rules(Some(RULES.iter().map(|rule| (*rule).into()).collect()))
}

fn findings(result: &crate::LintResult) -> Vec<&str> {
    let mut found: Vec<&str> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .filter(|rule| RULES.contains(rule))
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn javascript_sfc_blocks_skip_type_syntax_rules() {
    let script = "import { ref } from 'vue'\nconst props = defineProps({ a: String })\nconst emit = defineEmits(['change'])\nconst el = ref(null)\n";
    for lang in ["", " lang=\"js\""] {
        let sfc = format!("<script setup{lang}>\n{script}</script>\n");
        assert!(
            findings(&linter().lint_sfc(&sfc, "Js.vue")).is_empty(),
            "{lang}"
        );
    }
    let sfc = format!("<script setup lang=\"ts\">\n{script}</script>\n");
    let mut expected = RULES;
    expected.sort_unstable();
    assert_eq!(findings(&linter().lint_sfc(&sfc, "Ts.vue")), expected);
}

#[test]
fn standalone_files_follow_their_extension() {
    let source = "import { ref } from 'vue'\nexport const el = ref(null)\n";
    assert!(findings(&linter().lint_script(source, "use-el.js")).is_empty());
    assert_eq!(
        findings(&linter().lint_script(source, "use-el.ts")),
        ["script/require-typed-ref"]
    );
}
