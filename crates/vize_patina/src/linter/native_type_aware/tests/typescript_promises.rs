//! Promise rules decide on checker facts (`__promiseFacts` and friends), so
//! they agree with typescript-eslint where rendered type names cannot tell.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::diagnostic::Severity;
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

/// `(line, rule)` pairs, with the exact message where it carries a name.
const EXPECTED: &[(u32, &str, Option<&str>)] = &[
    (9, "type/await-thenable", None),
    (12, "type/no-floating-promises", None),
    // `Promise.all(...)` and `new Promise(...)` statements float too.
    (26, "type/no-floating-promises", None),
    (31, "type/no-floating-promises", None),
    (
        19,
        "type/no-misused-promises",
        Some(
            "Promise-returning method provided where a void return was expected by extended/implemented type 'Base'.",
        ),
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
    for (line, rule, expected_message) in EXPECTED {
        let Some(expected_message) = expected_message else {
            continue;
        };
        assert!(
            actual.iter().any(|(at, name, message)| at == line
                && name == rule
                && message == expected_message),
            "{rule} on line {line} should read {expected_message}: {actual:#?}"
        );
    }
}

fn line_of(source: &str, offset: u32) -> u32 {
    let prefix = source.get(..offset as usize).unwrap_or(source);
    // 1-based, with the `<script>` tag on line 1.
    prefix.matches('\n').count() as u32 + 1
}

struct RequireAwaitCase {
    name: &'static str,
    script: &'static str,
    first_parameter: Option<&'static str>,
}

const REQUIRE_AWAIT_ANY_CASES: &[RequireAwaitCase] = &[
    RequireAwaitCase {
        name: "foo",
        script: r#"async function foo(): Promise<any> {
  return {} as any;
}"#,
        first_parameter: None,
    },
    RequireAwaitCase {
        name: "fn",
        script: r#"async function fn<T extends any>(x: T): Promise<unknown> {
  return x as any;
}"#,
        first_parameter: Some("x: T"),
    },
    RequireAwaitCase {
        name: "foo",
        script: r#"declare const value: any;
async function foo() {
  return value;
}"#,
        first_parameter: None,
    },
    RequireAwaitCase {
        name: "foo",
        script: r#"type Awaited<T> = T extends null | undefined
  ? T
  : T extends object & { then(onfulfilled: infer F, ...args: infer _): any }
    ? F extends (value: infer V, ...args: infer _) => any
      ? Awaited<V>
      : never
    : T;

declare const value: Awaited<Promise<Promise<any>>>;
async function foo(): Promise<number> {
  return value;
}"#,
        first_parameter: None,
    },
];

#[test]
fn require_await_reports_the_four_non_thenable_any_returns() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    for (index, case) in REQUIRE_AWAIT_ANY_CASES.iter().enumerate() {
        let source = format!(
            "<script lang=\"ts\">\n{}\nexport {{}};\n</script>",
            case.script
        );
        let result = lint_sfc_with_corsa(&linter, &source, &format!("Case{index}.vue"));
        let diagnostics = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "type/require-await")
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 1, "case {index}: {}", case.script);
        let diagnostic = diagnostics[0];
        let script_start = source.find(case.script).expect("script content") as u32;
        let function_start = case.script.find("async function").expect("function head") as u32;
        let function_end = case
            .first_parameter
            .map_or(case.script.len() as u32, |parameter| {
                case.script.find(parameter).expect("first parameter") as u32
            });
        assert_eq!(diagnostic.rule_name, "type/require-await");
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(
            diagnostic.message.as_str(),
            format!("Async function '{}' has no 'await' expression.", case.name)
        );
        assert_eq!(diagnostic.start, script_start + function_start);
        assert_eq!(diagnostic.end, script_start + function_end);
        assert!(diagnostic.help.is_none());
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
    }
}

#[test]
fn require_await_keeps_thenable_any_returns_valid() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script lang="ts">
async function promisedAny(): Promise<any> {
  return Promise.resolve<any>({});
}
export {};
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "ThenableControl.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "type/require-await")
    );
}

/// An unsupported template lang leaves the template out, never the native
/// type-aware script checks: the same SFC with an HTML and a `haml`
/// template reports the same floating promise through `Linter::lint_sfc`.
#[test]
fn an_unsupported_template_lang_keeps_native_script_checks() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
    let script = "<script setup lang=\"ts\">\nasync function f() {}\nf();\n</script>\n";
    let floating = |template: &str| -> Vec<(u32, u32)> {
        let source = format!("{script}{template}");
        let mut found: Vec<_> = linter
            .lint_sfc(&source, "Unsupported.vue")
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "type/no-floating-promises")
            .map(|diagnostic| (diagnostic.start, diagnostic.end))
            .collect();
        found.sort_unstable();
        found
    };
    let html = floating("<template lang=\"html\"><p/></template>\n");
    assert_eq!(html.len(), 1, "{html:?}");
    assert_eq!(floating("<template lang=\"haml\">%p  </template>\n"), html);
}
