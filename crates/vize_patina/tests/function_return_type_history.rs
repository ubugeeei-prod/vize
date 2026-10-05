//! Issue #7913: distinguish runtime functions from TypeScript type arrows.

use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale};

const RULE: &str = "script/require-function-return-type";
const ARROW_HELP: &str = "Add a return type annotation: `const fn = (...): ReturnType => { ... }`";

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
}

fn assert_result(actual: LintResult, filename: &str, diagnostics: Vec<LintDiagnostic>) {
    let expected = LintResult {
        filename: filename.into(),
        warning_count: diagnostics.len(),
        error_count: 0,
        diagnostics,
    };
    // Compare the entire public result, including help, labels, fixes and ranges.
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"));
}

#[test]
fn original_reporter_sfc_and_plain_typescript_are_clean() {
    let linter = linter();
    for (filename, source) in [
        (
            "RunButton.vue",
            include_str!("fixtures/issue-7913/RunButton.vue"),
        ),
        (
            "callbacks.ts",
            include_str!("fixtures/issue-7913/callbacks.ts"),
        ),
    ] {
        let result = if filename.ends_with(".vue") {
            linter.lint_sfc(source, filename)
        } else {
            linter.lint_script(source, filename)
        };
        assert_result(result, filename, vec![]);
    }
}

#[test]
fn nested_function_constructor_and_generic_types_are_clean() {
    let source = r#"
type Handler = (value: string) => void;
type Build = new (value: string) => { handler: Handler };
type Factory<T> = <U extends T>(value: U) => () => U;
interface Queue { callbacks: Array<() => void>; run(value: Handler): void; }
const handlers: Map<string, (value: string) => Handler> = new Map();
const callback = (value: string): (() => void) => (): void => {};
declare function ambient(value: Handler): void;
function overloaded(value: string): void;
function overloaded(value: number): void;
function overloaded(value: string | number): void {}
const text = "function ghost() {} and () => void";
// function comment() {} and () => void
"#;
    assert_result(linter().lint_script(source, "types.ts"), "types.ts", vec![]);
}

#[test]
fn real_arrows_keep_whole_findings_at_authored_utf8_crlf_ranges() {
    let source = "// 雪😀\r\nconst run = (value: string) => { return value };\r\n";
    let start = source.find("(value").unwrap() as u32;
    let end = source.find("=>").unwrap() as u32 + 2;
    let diagnostic = LintDiagnostic::warn(
        RULE,
        "Arrow function is missing a return type annotation",
        start,
        end,
    )
    .with_help(ARROW_HELP);
    assert_result(
        linter().lint_script(source, "runtime.ts"),
        "runtime.ts",
        vec![diagnostic],
    );
}

#[test]
fn annotated_functions_and_existing_callback_policy_remain_clean() {
    let source = r#"
function run(value: string): string { return value }
const arrow = (value: string): string => value;
consume(() => { return 1 }, (value: string) => { return value });
"#;
    assert_result(
        linter().lint_script(source, "callbacks.ts"),
        "callbacks.ts",
        vec![],
    );
}

#[test]
fn same_function_name_in_a_type_does_not_hide_the_runtime_function() {
    let source = "type run = () => void;\nfunction run(value: string) { return value }";
    let start = source.find("function run").unwrap() as u32;
    let end = source.rfind(')').unwrap() as u32 + 1;
    let diagnostic = LintDiagnostic::warn(
        RULE,
        "Function 'run' is missing a return type annotation",
        start,
        end,
    )
    .with_help("Add a return type annotation: `function fn(...): ReturnType { ... }`");
    assert_result(
        linter().lint_script(source, "shadow.ts"),
        "shadow.ts",
        vec![diagnostic],
    );
}
