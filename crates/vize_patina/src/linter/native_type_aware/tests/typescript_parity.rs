//! Checker facts behind typescript-eslint parity (#12886, #12887, #12893,
//! #12894, #12896, #12918, #12919, #12921, #12995): type flags, structural shapes,
//! member-method facts and file-local declarations.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintDiagnostic, LintPreset, Linter, Severity};

mod no_implied_eval_callable;
mod no_unnecessary_type_assertion;
mod restrict_plus_generic_intersection;
mod unbound_method_lib_members;
mod upstream_872_cases;
mod upstream_872_conformance_12983;
mod upstream_872_conformance_b;
mod upstream_872_enum_assignment_12994;
mod upstream_872_no_unsafe_return_12996;

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

const ENUM_COMPARISON_RULE: &str = "type/no-unsafe-enum-comparison";
const BRANDED_ENUM_CASE: &str = r#"enum Fruit {
  Apple = 'apple',
}
type __String =
  | (string & { __escapedIdentifier: void })
  | (void & { __escapedIdentifier: void })
  | Fruit;
declare const weirdString: __String;
weirdString === 'someArbitraryValue';"#;
const ENUM_COMPARISON_MESSAGE: &str =
    "The two values in this comparison do not have a shared enum type.";

fn enum_comparison_diagnostics(source: &str, filename: &str) -> Vec<LintDiagnostic> {
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![ENUM_COMPARISON_RULE.into()]));
    lint_sfc_with_corsa(&linter, source, filename)
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_name == ENUM_COMPARISON_RULE)
        .collect()
}

fn assert_enum_comparison_packet(source: &str, diagnostic: &LintDiagnostic) {
    let start = u32::try_from(source.find("weirdString ===").expect("comparison"))
        .expect("comparison offset");
    let end = start + u32::try_from("weirdString === 'someArbitraryValue'".len()).unwrap();
    assert_eq!(diagnostic.rule_name, ENUM_COMPARISON_RULE);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(diagnostic.message, ENUM_COMPARISON_MESSAGE);
    assert_eq!((diagnostic.start, diagnostic.end), (start, end));
    assert_eq!(diagnostic.help, None);
    assert!(diagnostic.labels.is_empty());
    assert!(diagnostic.fix.is_none());
}

#[test]
fn branded_string_enum_comparison_has_the_same_packet_in_both_script_modes() {
    if !corsa_available() {
        return;
    }
    for (filename, source) in [
        (
            "Classic.vue",
            format!("<script lang=\"ts\">\n{BRANDED_ENUM_CASE}\nexport {{}};\n</script>"),
        ),
        (
            "Setup.vue",
            format!("<script setup lang=\"ts\">\n{BRANDED_ENUM_CASE}\n</script>"),
        ),
    ] {
        let actual = enum_comparison_diagnostics(&source, filename);
        assert_eq!(actual.len(), 1, "{filename}: {actual:#?}");
        assert_enum_comparison_packet(&source, &actual[0]);
    }
}

#[test]
fn branded_string_controls_do_not_invent_enum_mismatches() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
enum Fruit {
  Apple = 'apple',
}
type BrandedString = string & { __escapedIdentifier: void };
declare const branded: BrandedString;
declare const fruitOrBrand: Fruit | BrandedString;
branded === 'someArbitraryValue';
fruitOrBrand === Fruit.Apple;
</script>"#;
    assert!(
        enum_comparison_diagnostics(source, "Controls.vue").is_empty(),
        "negative controls must remain silent"
    );
}

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

#[test]
fn setup_enum_computed_keys_do_not_become_unsafe_members() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
enum Keys {
  A = "A",
  B = "B",
}
type Foo = {
  [Keys.A]: number | null;
  [Keys.B]: number;
};
declare const foo: Foo;
const value = foo[Keys.A] ?? 0;
</script>

<template>
  <p>{{ value }}</p>
</template>"#;
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, source, "Keyed.vue");
    let unsafe_members = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "type/no-unsafe-member-access")
        .collect::<Vec<_>>();

    assert!(
        unsafe_members.is_empty(),
        "setup enum members must retain their authored type: {unsafe_members:#?}"
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
