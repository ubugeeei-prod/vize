use super::ScriptLinter;

#[test]
fn test_has_vue_imports() {
    assert!(ScriptLinter::has_vue_imports("import { ref } from 'vue'"));
    assert!(ScriptLinter::has_vue_imports("import { ref } from \"vue\""));
    assert!(ScriptLinter::has_vue_imports(
        "import { h } from '@vue/runtime-core'"
    ));
    assert!(!ScriptLinter::has_vue_imports("import { foo } from 'bar'"));
}

#[test]
fn test_empty_linter() {
    let linter = ScriptLinter::new();
    let result = linter.lint("import { ref } from 'vue'", 0);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}
