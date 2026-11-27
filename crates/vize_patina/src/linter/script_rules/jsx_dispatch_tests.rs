//! Standalone `.jsx` / `.tsx` modules reach the AST script rules.

use super::super::Linter;

const RULE: &str = "script/no-ref-as-operand";

fn operand_reports(source: &str, filename: &str, lang: vize_atelier_jsx::JsxLang) -> usize {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_jsx(source, filename, lang)
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .count()
}

#[test]
fn a_tsx_module_with_jsx_syntax_runs_ast_script_rules() {
    let source = "import { defineComponent, useTemplateRef } from 'vue'
export default defineComponent({
  setup() {
    const el = useTemplateRef<HTMLInputElement>('el')
    if (el) {
      console.log('mounted')
    }
    return () => <input ref=\"el\" />
  },
})
";
    assert_eq!(
        operand_reports(source, "A.tsx", vize_atelier_jsx::JsxLang::Tsx),
        1
    );
}

#[test]
fn a_jsx_module_with_jsx_syntax_runs_ast_script_rules() {
    let source = "import { ref } from 'vue'
const count = ref(0)
const next = count + 1
export const View = () => <p>{next}</p>
";
    assert_eq!(
        operand_reports(source, "A.jsx", vize_atelier_jsx::JsxLang::Jsx),
        1
    );
}
