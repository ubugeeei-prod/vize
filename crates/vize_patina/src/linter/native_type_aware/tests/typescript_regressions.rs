//! Reported regressions in the typescript-eslint checks on SFC scripts:
//! empty tuples that panic the native runtime (#12949) and unbound-method on
//! destructured composable members (#12943).
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, LintResult, Linter};

/// `(line, rule)` of every finding of `rules`, sorted, with the `<script>`
/// tag on line 1.
fn findings(result: &LintResult, source: &str, rules: &[&str]) -> Vec<(u32, String)> {
    let mut found = result
        .diagnostics
        .iter()
        .filter(|diagnostic| rules.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            let prefix = source.get(..diagnostic.start as usize).unwrap_or(source);
            (
                prefix.matches('\n').count() as u32 + 1,
                diagnostic.rule_name.to_string(),
            )
        })
        .collect::<Vec<_>>();
    found.sort();
    found
}

const EMPTY_TUPLES: &str = r#"<script setup lang="ts">
const results = [] as const;
const all = Promise.all([]);
const top = Math.max(...[]);
async function save(): Promise<void> {}
Promise.all([]);
save();
const count: number = results.length + top;
</script>

<template>
  <p>{{ all }} {{ count }}</p>
</template>"#;

/// TypeScript 7.0 and 7.1 panic describing the empty tuple of `[] as const`,
/// `Promise.all([])` and `...[]`, which failed the whole fact batch: the file
/// got `type/corsa-runtime` and no `type/*` rule ran. Only those expressions
/// go untyped now.
#[test]
fn empty_tuples_leave_the_other_checks_running() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, EMPTY_TUPLES, "EmptyTuples.vue");
    assert_eq!(
        findings(
            &result,
            EMPTY_TUPLES,
            &[
                "type/corsa-runtime",
                "type/no-floating-promises",
                "type/restrict-plus-operands"
            ]
        ),
        vec![
            (6, "type/no-floating-promises".into()),
            (7, "type/no-floating-promises".into()),
        ],
        "{:#?}",
        result.diagnostics
    );
}

const COMPOSABLES: &str = r#"<script setup lang="ts">
function useArrow() {
  const copy = (): void => {};
  return { copy };
}
function useAsyncArrow() {
  const save = async (): Promise<void> => {};
  return { save };
}
function useArrowProperty() {
  return { open: (): void => {} };
}
function useFunctionDeclaration() {
  function load(): void {}
  return { load };
}
function useFunctionExpression() {
  return { close: function (): void {} };
}
function useMethodShorthand() {
  return { reset(): void {} };
}
class Fields {
  arrow = () => {};
  expression = function () {};
}
const { copy } = useArrow();
const { save } = useAsyncArrow();
const { open } = useArrowProperty();
const { load } = useFunctionDeclaration();
const { close } = useFunctionExpression();
const { reset } = useMethodShorthand();
const { arrow, expression } = new Fields();
</script>

<template>
  <button @click="copy(); save(); open(); load(); close(); reset(); arrow(); expression()">x</button>
</template>"#;

/// typescript-eslint's `checkIfMethod`: a method, or a property initialized
/// with a `function` expression, is unbound once destructured; an arrow
/// function, a shorthand of one and a function declaration are not.
#[test]
fn unbound_method_reports_methods_and_function_expressions_only() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, COMPOSABLES, "Composables.vue");
    assert_eq!(
        findings(&result, COMPOSABLES, &["type/unbound-method"]),
        vec![
            (31, "type/unbound-method".into()),
            (32, "type/unbound-method".into()),
            (33, "type/unbound-method".into()),
        ],
        "{:#?}",
        result.diagnostics
    );
}
