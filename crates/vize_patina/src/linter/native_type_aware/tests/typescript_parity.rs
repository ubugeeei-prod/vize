//! Checker facts behind typescript-eslint parity (#12886, #12887, #12893,
//! #12894, #12896, #12918, #12919, #12921): type flags, structural shapes,
//! member-method facts and file-local declarations.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

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
    // `text + value`: the object operand, not the whole expression.
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
