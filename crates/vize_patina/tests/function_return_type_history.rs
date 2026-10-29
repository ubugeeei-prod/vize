//! Issues #7913 and #11103: preserve runtime ownership and contextual callbacks.

use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale};

const RULE: &str = "script/require-function-return-type";
const ARROW_HELP: &str = "Add a return type annotation: `const fn = (...): ReturnType => { ... }`";
const FUNCTION_HELP: &str = "Add a return type annotation: `function fn(...): ReturnType { ... }`";

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
            include_str!("fixtures/issue-7913/RunButton.vue.fixture"),
        ),
        (
            "callbacks.ts",
            include_str!("fixtures/issue-7913/callbacks.ts.fixture"),
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

#[test]
fn contextual_object_callbacks_typed_expressions_and_iifes_are_clean() {
    let source = r#"
declare function useOptions(options: {
    value: () => number;
    clear: () => void;
    nested: { run: () => void };
    list: Array<() => void>;
}): void;
declare class Holder {
    constructor(options: { value: () => number });
}
declare function register(callback: () => void): void;

useOptions({
    value: () => 1,
    clear: function () {},
    nested: { run: () => {} },
    list: [() => {}],
});
new Holder({ value: () => 2 });
register(function () {});

const typed: () => number = () => 1;
const typedObject: { value: () => number } = { value: () => 1 };
const asserted = { value: () => 1 } as { value: () => number };
const satisfied = { value: () => 1 } satisfies { value: () => number };
function withDefault(callback: () => void = () => {}): void { callback(); }

const arrowIife = (() => 1)();
const functionIife = (function () { return 1; })();
void typed;
void typedObject;
void asserted;
void satisfied;
void arrowIife;
void functionIife;
"#;
    assert_result(
        linter().lint_script(source, "contextual.ts"),
        "contextual.ts",
        vec![],
    );
}

fn arrow_diagnostic(source: &str, params: &str) -> LintDiagnostic {
    let start = source.find(params).unwrap();
    let end = start + source[start..].find("=>").unwrap() + 2;
    LintDiagnostic::warn(
        RULE,
        "Arrow function is missing a return type annotation",
        start as u32,
        end as u32,
    )
    .with_help(ARROW_HELP)
}

fn named_function_diagnostic(source: &str, name: &'static str) -> LintDiagnostic {
    let marker = format!("function {name}");
    let start = source.find(&marker).unwrap();
    let end = start + source[start..].find(')').unwrap() + 1;
    LintDiagnostic::warn(
        RULE,
        format!("Function '{name}' is missing a return type annotation"),
        start as u32,
        end as u32,
    )
    .with_help(FUNCTION_HELP)
}

#[test]
fn standalone_functions_and_declarations_inside_callbacks_still_report_exactly() {
    let source = r#"
const standalone = (standaloneValue: number) => standaloneValue;
const options = {
    value: (objectValue: number) => objectValue,
    clear: function objectClear() {},
};
declare function consume(callback: () => void): void;
consume(() => {
    function nestedDeclared() { return 1; }
    const nestedArrow = (nestedValue: number) => nestedValue;
    void nestedArrow;
});
void standalone;
void options;
"#;
    let diagnostics = vec![
        arrow_diagnostic(source, "(standaloneValue: number)"),
        arrow_diagnostic(source, "(objectValue: number)"),
        named_function_diagnostic(source, "objectClear"),
        named_function_diagnostic(source, "nestedDeclared"),
        arrow_diagnostic(source, "(nestedValue: number)"),
    ];
    assert_result(
        linter().lint_script(source, "controls.ts"),
        "controls.ts",
        diagnostics,
    );
}
