use super::super::plain_exports::{PlainScriptExport, PlainScriptExportKind};
use super::NamespaceHoistPlan;
use vize_s0::{CompactString, String as VizeString};

fn plan(script: &str) -> NamespaceHoistPlan {
    NamespaceHoistPlan::collect(Some(script), false, true)
}

fn hoisted_text(script: &str) -> Vec<&str> {
    plan(script)
        .spans()
        .iter()
        .map(|&(start, end)| &script[start as usize..end as usize])
        .collect()
}

fn export(name: &str, kind: PlainScriptExportKind) -> PlainScriptExport {
    PlainScriptExport {
        name: CompactString::new(name),
        kind,
        source_range: 0..name.len(),
        bridged_value: true,
    }
}

#[test]
fn every_namespace_form_is_hoisted_with_its_export_keyword() {
    let script = "namespace Bare {\n  export const a = 1;\n}\nexport namespace Named {\n  export const b = 2;\n}\nexport module Legacy {\n  export const c = 3;\n}\nexport namespace A.B.C {\n  export const d = 4;\n}\ndeclare namespace Ambient {\n  const e: number;\n}\n";

    assert_eq!(
        hoisted_text(script),
        vec![
            "namespace Bare {\n  export const a = 1;\n}",
            "export namespace Named {\n  export const b = 2;\n}",
            "export module Legacy {\n  export const c = 3;\n}",
            "export namespace A.B.C {\n  export const d = 4;\n}",
            "declare namespace Ambient {\n  const e: number;\n}",
        ]
    );
}

#[test]
fn nominal_declarations_keep_module_identity_without_a_namespace() {
    let script = "export const a = 1;\nexport class C {}\nexport enum E { A }\n";

    assert_eq!(
        hoisted_text(script),
        vec!["export class C {}", "export enum E { A }"]
    );
}

#[test]
fn callable_declarations_and_overloads_keep_identity_without_a_namespace() {
    let script = "const seed = 1;\nfunction helper(value: number): number { return value + seed }\nexport function build(value: number): number;\nexport function build(value: string): string;\nexport function build(value: number | string) { return typeof value === 'number' ? helper(value) : value }\n";
    assert_eq!(
        hoisted_text(script),
        vec![
            "function helper(value: number): number { return value + seed }",
            "export function build(value: number): number;",
            "export function build(value: string): string;",
            "export function build(value: number | string) { return typeof value === 'number' ? helper(value) : value }",
        ]
    );
    let plan = plan(script);
    let mut exports = vec![export("build", PlainScriptExportKind::Value)];
    plan.reconcile_exports(&mut exports);
    assert!(exports.is_empty());
    let mut fields = Vec::new();
    plan.push_captured_return_fields(&exports, &mut fields);
    assert_eq!(fields, vec![CompactString::new("seed")]);
}

#[test]
fn callable_declarations_keep_their_leading_diagnostic_directives() {
    let script = "const seed = 1;\n// @ts-expect-error authored return mismatch\nexport function bad(): number { return 'bad' }\n";
    assert_eq!(
        hoisted_text(script),
        vec![
            "// @ts-expect-error authored return mismatch\nexport function bad(): number { return 'bad' }"
        ]
    );
}

#[test]
fn relocated_declarations_keep_leading_documentation_but_not_trailing_comments() {
    let script = "const seed = 1; /** Not documentation. */\nfunction helper() { return seed }\n/** First overload. */\nexport function build(value: number): number;\n/**\n * Implementation.\n */\nexport function build(value: number) { return helper() + value }\n/** Class documentation. */ export class C {}\n";
    assert_eq!(
        hoisted_text(script),
        vec![
            "function helper() { return seed }",
            "/** First overload. */\nexport function build(value: number): number;",
            "/**\n * Implementation.\n */\nexport function build(value: number) { return helper() + value }",
            "/** Class documentation. */ export class C {}",
        ]
    );
}

#[test]
fn documentation_and_diagnostic_directives_move_together() {
    let script = "const seed = 1;\r\n// @ts-expect-error authored return mismatch\r\n/** Return the greeting. */\r\nexport function bad(): number { return 'bad' }\r\n";
    assert_eq!(
        hoisted_text(script),
        vec![
            "// @ts-expect-error authored return mismatch\r\n/** Return the greeting. */\r\nexport function bad(): number { return 'bad' }"
        ]
    );
}

#[test]
fn a_same_line_trailing_jsdoc_is_not_promoted_to_leading_documentation() {
    let script = "const seed = 1; /** Not documentation. */ function helper() { return seed }";
    assert_eq!(
        hoisted_text(script),
        vec!["function helper() { return seed }"]
    );
}

#[test]
fn a_merge_partner_is_hoisted_and_dropped_from_the_export_bridge() {
    let script = "export function f() {}\nexport namespace f {\n  export const v = 1;\n}\nexport const other = 2;\n";

    assert_eq!(
        hoisted_text(script),
        vec![
            "export namespace f {\n  export const v = 1;\n}",
            "export function f() {}",
        ]
    );

    let mut exports = vec![
        export("f", PlainScriptExportKind::Value),
        export("other", PlainScriptExportKind::Value),
    ];
    plan(script).reconcile_exports(&mut exports);
    assert_eq!(exports, vec![export("other", PlainScriptExportKind::Value)]);
}

#[test]
fn a_same_named_variable_is_not_treated_as_a_merge_partner() {
    // TypeScript does not merge a namespace into a `const`, so hoisting the
    // variable would relocate user code without repairing anything.
    let script = "const N = 1;\nnamespace N {\n  export type T = number;\n}\n";

    assert_eq!(
        hoisted_text(script),
        vec!["namespace N {\n  export type T = number;\n}"]
    );
}

#[test]
fn captured_setup_bindings_become_ambient_aliases_of_the_setup_return() {
    let script = "const localBase = 10;\nclass Local {}\nexport const shared = 2;\nconst unused = 3;\nexport namespace Uses {\n  export const derived = localBase + shared;\n  export const made: Local = new Local();\n}\n";

    let plan = plan(script);
    let mut exports = vec![export("shared", PlainScriptExportKind::Value)];
    plan.reconcile_exports(&mut exports);
    assert_eq!(
        exports,
        vec![PlainScriptExport {
            name: CompactString::new("shared"),
            kind: PlainScriptExportKind::Value,
            source_range: 0.."shared".len(),
            bridged_value: false,
        }]
    );

    let mut ts = VizeString::default();
    plan.emit_ambient_captures(&mut ts, &exports);
    assert_eq!(
        ts.as_str(),
        "// Setup-scope bindings a hoisted namespace body reads\n\
         declare const localBase: ReturnType<typeof __setup>[\"localBase\"];\n\
         export declare const shared: ReturnType<typeof __setup>[\"shared\"];\n\
         \n"
    );

    let mut fields = Vec::new();
    plan.push_captured_return_fields(&exports, &mut fields);
    assert_eq!(fields, vec![CompactString::new("localBase")]);
}

#[test]
fn a_member_property_matching_a_setup_binding_is_not_captured() {
    let script = "const total = 1;\nconst counts = { total: 2 };\nnamespace Sums {\n  export const value = counts.total;\n}\n";

    let plan = plan(script);
    let mut ts = VizeString::default();
    plan.emit_ambient_captures(&mut ts, &[]);
    assert_eq!(
        ts.as_str(),
        "// Setup-scope bindings a hoisted namespace body reads\n\
         declare const counts: ReturnType<typeof __setup>[\"counts\"];\n\
         \n"
    );
}

#[test]
fn a_merge_partner_body_capture_is_aliased_too() {
    let script = "const seed = 1;\nexport function build() {\n  return seed;\n}\nexport namespace build {\n  export const v = 2;\n}\n";

    let plan = plan(script);
    let mut ts = VizeString::default();
    plan.emit_ambient_captures(&mut ts, &[]);
    assert_eq!(
        ts.as_str(),
        "// Setup-scope bindings a hoisted namespace body reads\n\
         declare const seed: ReturnType<typeof __setup>[\"seed\"];\n\
         \n"
    );
}

#[test]
fn a_script_setup_block_keeps_the_plan_empty() {
    let script = "export namespace N {\n  export const a = 1;\n}\n";

    let plan = NamespaceHoistPlan::collect(Some(script), true, true);
    assert_eq!(plan.spans(), &[] as &[(u32, u32)]);

    let mut ts = VizeString::default();
    plan.emit_ambient_captures(&mut ts, &[]);
    assert_eq!(ts.as_str(), "");
}
