//! Checker facts behind typescript-eslint parity (#12886, #12887, #12893,
//! #12894, #12896, #12918, #12919, #12921): type flags, structural shapes,
//! member-method facts and file-local declarations. Also the reported
//! regressions: empty tuples that panic the native runtime (#12949) and
//! unbound-method on destructured composable members (#12943), and
//! `type/no-unnecessary-type-parameters` (#12947): written uses decide most
//! parameters, the checker counts the uses an inferred type or a class
//! member adds.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, LintResult, Linter};

const SOURCE: &str = r#"<script lang="ts">
declare const maybe: (() => void) | undefined;
declare const text: string;
declare const map: Map<string, number>;
class Holder {
  method() {}
  free = function (this: void) {};
  arrow = () => {};
}
declare const holder: Holder;
const literal = { f: function (this: void) {}, g() {} };

async function confusing() {
  await maybe?.();
  console?.log(text);
  return (console.log(text), text);
}
function generic<T extends string>(value: T) {
  return `${value}` + value;
}
function plus(value: { a: number }) {
  return text + value;
}
const spreadMap = { ...map };
const stringified = `${literal}`;
void text;
setTimeout(text);
function local(setTimeout: (code: string) => void) {
  setTimeout(text);
}
const bound = holder.free;
const arrow = holder.arrow;
const unbound = holder.method;
const { f, g } = literal;
if (holder.method) {
}
export { confusing, generic, plus, local, spreadMap, stringified, bound, arrow, unbound, f, g };
</script>"#;

const RULES: &[&str] = &[
    "type/no-base-to-string",
    "type/no-confusing-void-expression",
    "type/no-implied-eval",
    "type/no-meaningless-void-operator",
    "type/no-misused-spread",
    "type/restrict-plus-operands",
    "type/restrict-template-expressions",
    "type/unbound-method",
];

const EXPECTED: &[(u32, u32, &str)] = &[
    // `text + value`: the object operand, not the whole expression. Its
    // `Object` stringification is a base-to-string finding as well.
    (22, 17, "type/no-base-to-string"),
    (22, 17, "type/restrict-plus-operands"),
    (24, 21, "type/no-misused-spread"),
    (25, 24, "type/no-base-to-string"),
    (25, 24, "type/restrict-template-expressions"),
    (26, 1, "type/no-meaningless-void-operator"),
    (27, 12, "type/no-implied-eval"),
    (33, 17, "type/unbound-method"),
    (34, 12, "type/unbound-method"),
];

#[test]
fn checker_facts_match_typescript_eslint() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "Parity.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| RULES.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            let (line, column) = line_column(SOURCE, diagnostic.start);
            (
                line,
                column,
                diagnostic.rule_name,
                diagnostic.message.to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort_by_key(|(line, column, rule, _)| (*line, *column, *rule));
    let keys = actual
        .iter()
        .map(|(line, column, rule, _)| (*line, *column, *rule))
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        EXPECTED,
        "findings:\n{}",
        actual
            .iter()
            .map(|(line, column, rule, message)| format!("{line}:{column} {rule} {message}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for (index, character) in source.char_indices() {
        if index >= offset as usize {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

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

const TYPE_PARAMETERS: &str = r#"<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
function identity<T>(value: T): T {
  return value;
}
function wrap<T>(value: T) {
  return [value];
}
function first<K, V>(map: Map<K, V>): K | undefined {
  return map.keys().next().value;
}
class Box<T> {
  value?: T;
}
class Pair<T> {
  constructor(readonly left: T, readonly right: T) {}
}
logValue(1);
const kept = identity("a");
export { kept, wrap, first, Box, Pair };
</script>"#;

/// `(line, column, message)`, with the `<script>` tag on line 1.
const TYPE_PARAMETER_FINDINGS: &[(u32, u32, &str)] = &[
    (
        2,
        19,
        "Type parameter T is used only once in the function signature.",
    ),
    (
        14,
        11,
        "Type parameter T is used only once in the class signature.",
    ),
];

#[test]
fn reports_type_parameters_used_once() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, TYPE_PARAMETERS, "TypeParameters.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "type/no-unnecessary-type-parameters")
        .map(|diagnostic| {
            let (line, column) = line_column(TYPE_PARAMETERS, diagnostic.start);
            (line, column, diagnostic.message.to_string())
        })
        .collect::<Vec<_>>();
    actual.sort();
    let expected = TYPE_PARAMETER_FINDINGS
        .iter()
        .map(|(line, column, message)| (*line, *column, (*message).to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{:#?}", result.diagnostics);
}
