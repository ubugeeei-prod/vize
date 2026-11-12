//! Promise rules decide on checker facts (`__promiseFacts` and friends), so
//! they agree with typescript-eslint where rendered type names cannot tell.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const SOURCE: &str = r#"<script setup lang="ts">
declare const anyValue: any;
declare const createLike: () => PromiseLike<number>;
declare const promiseValue: Promise<number>;
declare function takesVoid(...callbacks: Array<() => void>): void;
async function awaits<T>(value: T, count: number) {
  await anyValue;
  await value;
  await count;
}
async function floats() {
  promiseValue;
  createLike();
}
class Base {
  run(): void {}
}
class Derived extends Base {
  async run() {}
}
type HasVoid = { f(): void };
const object: HasVoid = {
  async f() {},
};
takesVoid(() => {}, async () => {});
Promise.all([1, promiseValue]);
class CustomError {}
function throws(): never {
  throw new CustomError();
}
new Promise((resolve, reject) => reject(5));
</script>"#;

/// `(line, rule)` pairs, with the message where it carries a name.
const EXPECTED: &[(u32, &str, Option<&str>)] = &[
    (9, "type/await-thenable", None),
    (12, "type/no-floating-promises", None),
    (
        19,
        "type/no-misused-promises",
        Some("extended/implemented type 'Base'"),
    ),
    (23, "type/no-misused-promises", None),
    (25, "type/no-misused-promises", None),
    (26, "type/await-thenable", None),
    (29, "type/only-throw-error", None),
    (31, "type/prefer-promise-reject-errors", None),
];

#[test]
fn promise_rules_follow_checker_facts() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "Promises.vue");
    let wanted: Vec<&str> = EXPECTED.iter().map(|(_, rule, _)| *rule).collect();
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| wanted.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            (
                line_of(SOURCE, diagnostic.start),
                diagnostic.rule_name,
                diagnostic.message.to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    let lines = actual
        .iter()
        .map(|(line, rule, _)| (*line, *rule))
        .collect::<Vec<_>>();
    let mut expected = EXPECTED
        .iter()
        .map(|(line, rule, _)| (*line, *rule))
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(lines, expected, "promise findings: {actual:#?}");
    for (line, rule, fragment) in EXPECTED {
        let Some(fragment) = fragment else {
            continue;
        };
        assert!(
            actual.iter().any(|(at, name, message)| at == line
                && name == rule
                && message.contains(fragment)),
            "{rule} on line {line} should mention {fragment}: {actual:#?}"
        );
    }
}

fn line_of(source: &str, offset: u32) -> u32 {
    let prefix = source.get(..offset as usize).unwrap_or(source);
    // The SFC's first line is the `<script>` tag, so script lines start at 1.
    prefix.matches('\n').count() as u32
}
