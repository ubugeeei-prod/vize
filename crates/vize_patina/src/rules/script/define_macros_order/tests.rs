use super::DefineMacrosOrder;
use crate::rules::script::ScriptLinter;

fn lint(source: &str) -> Vec<String> {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(DefineMacrosOrder));
    linter
        .lint(source, 0)
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

fn lint_with_order(source: &str, order: &[&str]) -> Vec<String> {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(DefineMacrosOrder::configured(
        vize_l0::config::DefineMacrosOrderOptions {
            order: order.iter().map(|name| (*name).into()).collect(),
        },
    )));
    linter
        .lint(source, 0)
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

const NOT_AT_TOP: &str = "should be placed at the top of `<script setup>` (after any potential import statements or type definitions).";

// --- Valid: eslint-plugin-vue's default order ---

#[test]
fn props_then_emits_first() {
    let source = r#"
import { ref } from 'vue'
type Props = { count: number }
const props = defineProps<Props>()
const emit = defineEmits<{ change: [] }>()
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
defineSlots<{ default(props: {}): any }>()
const value = ref(0)
"#;
    assert!(lint(source).is_empty());
}

#[test]
fn either_macro_alone() {
    assert!(lint("const props = defineProps<{ count: number }>()\n").is_empty());
    assert!(
        lint("import { ref } from 'vue'\nconst emit = defineEmits(['change'])\nconst a = ref(0)\n")
            .is_empty()
    );
}

#[test]
fn unordered_macros_are_not_checked() {
    // defineModel, defineOptions and defineSlots are outside the default order.
    let source = r#"
const value = ref(0)
const model = defineModel<string>()
defineSlots<{ default(props: {}): any }>()
defineOptions({ name: 'MyComponent' })
"#;
    assert!(lint(source).is_empty());
}

#[test]
fn configured_order_accepts_define_options_first() {
    let source = r#"
defineOptions({ inheritAttrs: false })
const props = defineProps<{ label: string }>()
const emit = defineEmits<{ select: [] }>()
defineSlots<{ default(): unknown }>()
const value = ref(0)
"#;
    assert!(
        lint_with_order(
            source,
            &["defineOptions", "defineProps", "defineEmits", "defineSlots"]
        )
        .is_empty()
    );
}

#[test]
fn configured_order_reports_the_authored_order() {
    let source = r#"
const props = defineProps<{ label: string }>()
defineOptions({ inheritAttrs: false })
const emit = defineEmits<{ select: [] }>()
"#;
    assert_eq!(
        lint_with_order(source, &["defineOptions", "defineProps", "defineEmits"]),
        ["`defineOptions` should be above `defineProps`."]
    );
}

#[test]
fn configured_order_supports_project_macros() {
    let source = r#"
const props = defineProps<{ label: string }>()
definePage({ layout: 'default' })
"#;
    assert_eq!(
        lint_with_order(source, &["definePage", "defineProps"]),
        ["`definePage` should be above `defineProps`."]
    );
}

#[test]
fn skipped_statements_may_precede_the_macros() {
    let source = r#"
import type { Item } from "./item";
export type { Item };
export interface Props { label: string }
export const ready = true
enum Mode { A }
declare const injected: string
debugger
;
const props = withDefaults(defineProps<Props>(), { label: '' })
const { change } = defineEmits<{ change: [] }>()
"#;
    assert!(lint(source).is_empty());
}

#[test]
fn no_macros() {
    assert!(lint("import { ref } from 'vue'\nconst count = ref(0)\n").is_empty());
    assert!(lint("const label = 'defineProps then defineEmits'\n").is_empty());
}

// --- Invalid ---

#[test]
fn emits_before_props() {
    let source = r#"
const emit = defineEmits<{ change: [] }>()
const props = defineProps<{ count: number }>()
"#;
    assert_eq!(
        lint(source),
        ["`defineProps` should be above `defineEmits`."]
    );
}

#[test]
fn statement_before_props() {
    let source = r#"
const value = ref(0)
const props = defineProps<{ count: number }>()
"#;
    assert_eq!(lint(source), [format!("`defineProps` {NOT_AT_TOP}")]);
}

#[test]
fn other_macro_before_props() {
    // As upstream: an unordered macro is an ordinary statement.
    let source = r#"
const model = defineModel<string>()
const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 })
"#;
    assert_eq!(lint(source), [format!("`defineProps` {NOT_AT_TOP}")]);
}

#[test]
fn statement_between_props_and_emits() {
    let source = r#"
const props = defineProps<{ count: number }>()
type Emits = { change: [] }
const emit = defineEmits<Emits>()
"#;
    assert_eq!(lint(source), [format!("`defineEmits` {NOT_AT_TOP}")]);
}

#[test]
fn reports_one_statement_per_block() {
    let source = r#"
const value = ref(0)
const emit = defineEmits<{ change: [] }>()
const props = defineProps<{ count: number }>()
"#;
    assert_eq!(lint(source), [format!("`defineProps` {NOT_AT_TOP}")]);
}

#[test]
fn diagnostic_spans_the_statement() {
    let source = "const a = 1\nconst props = defineProps()\n";
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(DefineMacrosOrder));
    let result = linter.lint(source, 0);
    let [diagnostic] = result.diagnostics.as_slice() else {
        panic!("one diagnostic")
    };
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("const props = defineProps()")
    );
}
