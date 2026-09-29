use super::{CorsaTypeAwareSession, RULE_NO_FLOATING_PROMISES, RULE_NO_REACTIVITY_LOSS};
use crate::{JsxLang, LintPreset, LintResult, Linter};

const FLOATING: &str = "\
export async function save(): Promise<void> {}\n\
\n\
export function submit(): void {\n\
  save();\n\
}\n\
";

const AWAITED: &str = "\
export async function save(): Promise<void> {}\n\
\n\
export async function submit(): Promise<void> {\n\
  await save();\n\
}\n\
";

const IGNORED: &str = "\
export async function save(): Promise<void> {}\n\
\n\
export function submit(): void {\n\
  void save();\n\
}\n\
";

const REACTIVITY: &str = "\
import { reactive } from \"vue\";\n\
\n\
export function useCount() {\n\
  const state = reactive({ count: 0 });\n\
  const count = state.count;\n\
  return { count };\n\
}\n\
";

fn corsa_available() -> bool {
    let mut session = match CorsaTypeAwareSession::new_with_corsa_path("save.ts", None) {
        Ok(session) => session,
        Err(_) => return false,
    };
    if session
        .open_virtual_project("export const value = 1;\n", "save.ts")
        .is_err()
    {
        session.close();
        return false;
    }
    session.close();
    true
}

fn type_aware() -> Linter {
    Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true)
}

fn count(result: &LintResult, rule: &str) -> usize {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == rule)
        .count()
}

fn starts(result: &LintResult, rule: &str) -> Vec<u32> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == rule)
        .map(|diagnostic| diagnostic.start)
        .collect()
}

#[test]
fn type_aware_stays_off_until_the_host_opts_in() {
    let result = Linter::with_preset(LintPreset::Opinionated).lint_script(FLOATING, "src/save.ts");
    assert_eq!(count(&result, RULE_NO_FLOATING_PROMISES), 0);
    assert_eq!(count(&result, "type/corsa-runtime"), 0);
}

#[test]
fn plain_javascript_and_declarations_skip_type_rules() {
    let linter = type_aware();
    assert_eq!(
        count(
            &linter.lint_script(FLOATING, "src/save.js"),
            RULE_NO_FLOATING_PROMISES
        ),
        0
    );
    assert_eq!(
        count(
            &linter.lint_script(FLOATING, "src/save.d.ts"),
            RULE_NO_FLOATING_PROMISES
        ),
        0
    );
    assert_eq!(
        count(
            &linter.lint_script(FLOATING, "src/save.js"),
            "type/corsa-runtime"
        ),
        0
    );
}

#[test]
fn plain_typescript_reports_a_floating_promise() {
    if !corsa_available() {
        return;
    }

    let linter = type_aware();
    let result = linter.lint_script(FLOATING, "src/save.ts");
    let Some(call) = FLOATING.find("save()") else {
        panic!("save() is in the fixture");
    };
    assert_eq!(
        starts(&result, RULE_NO_FLOATING_PROMISES),
        vec![call as u32]
    );
    assert_eq!(
        count(
            &linter.lint_script(FLOATING, "src/save.mts"),
            RULE_NO_FLOATING_PROMISES
        ),
        1
    );
    assert_eq!(
        count(
            &linter.lint_jsx(FLOATING, "src/save.tsx", JsxLang::Tsx),
            RULE_NO_FLOATING_PROMISES
        ),
        1
    );
    assert_eq!(
        count(
            &linter.lint_script(AWAITED, "src/save.ts"),
            RULE_NO_FLOATING_PROMISES
        ),
        0
    );
    assert_eq!(
        count(
            &linter.lint_script(IGNORED, "src/save.ts"),
            RULE_NO_FLOATING_PROMISES
        ),
        0
    );
}

#[test]
fn plain_typescript_reports_reactivity_loss() {
    if !corsa_available() {
        return;
    }

    let result = type_aware().lint_script(REACTIVITY, "src/use-count.ts");
    let Some(extract) = REACTIVITY.find("state.count") else {
        panic!("state.count is in the fixture");
    };
    assert_eq!(
        starts(&result, RULE_NO_REACTIVITY_LOSS),
        vec![extract as u32]
    );
    assert_eq!(count(&result, "type/require-typed-props"), 0);
    assert_eq!(
        messages(&result, RULE_NO_REACTIVITY_LOSS),
        vec!["Assigning 'state.count' to 'count' stores a plain snapshot"]
    );
}

fn messages<'a>(result: &'a LintResult, rule: &str) -> Vec<&'a str> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == rule)
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}
