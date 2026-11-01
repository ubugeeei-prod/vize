use super::*;

fn facts(specifiers: &[&str], declarations: &[&str]) -> ClosureSyntaxFacts {
    ClosureSyntaxFacts {
        module_specifiers: specifiers
            .iter()
            .map(|value| String::from(*value))
            .collect(),
        module_declarations: declarations
            .iter()
            .map(|value| String::from(*value))
            .collect(),
        ..Default::default()
    }
}

#[test]
fn every_static_module_form_keeps_the_complete_ordered_vector() {
    let source = r#"
import Value from "static";
import type { Type } from "type";
export { Value } from "export";
export type { Type } from "export-type";
export * from "star";
export * as Namespace from "namespace-export";
type A = import("import-type").Thing;
type B = typeof import("typeof-import");
import Legacy = require("import-equals");
const dynamic = import("dynamic");
const common = require("common");
declare module "declared" {
  export type Inside = import("inside").Thing;
  import Other = require("inside-equals");
}
declare module "second-declared" {}
"#;
    assert_eq!(
        inspect(source, SourceType::ts()),
        Some(facts(
            &[
                "static",
                "type",
                "export",
                "export-type",
                "star",
                "namespace-export",
                "import-type",
                "typeof-import",
                "import-equals",
                "dynamic",
                "common",
                "inside",
                "inside-equals",
            ],
            &["declared", "second-declared"],
        ))
    );
}

#[test]
fn decoded_literals_templates_duplicates_and_nested_types_preserve_source_order() {
    let source = r#"
import "\u0070kg";
const first = import(`tem\u0070late`);
const second = require(`req\u0075ire`);
const third = require<import("inner").Type>("outer");
const fourth = (require)(("wrapped"));
const fifth = import(("wrapped-import"));
export * from "pkg";
declare module "\u0064eclared" {}
"#;
    assert_eq!(
        inspect(source, SourceType::ts()),
        Some(facts(
            &[
                "pkg",
                "template",
                "require",
                "inner",
                "outer",
                "wrapped",
                "wrapped-import",
                "pkg",
            ],
            &["declared"],
        ))
    );
}

#[test]
fn all_moved_bindings_including_escaped_and_nested_names_are_conservative_facts() {
    for name in MOVED_NAMES {
        for declaration in [
            vize_carton::cstr!("type {name}<T> = T;"),
            vize_carton::cstr!("function outer<{name}>() {{}}"),
            vize_carton::cstr!("namespace Scope {{ interface {name} {{}} }}"),
        ] {
            let mut expected = facts(&[], &[]);
            expected.moved_name_binding = true;
            assert_eq!(inspect(&declaration, SourceType::ts()), Some(expected));
        }
        let mut expected = facts(&["bindings"], &[]);
        expected.moved_name_binding = true;
        assert_eq!(
            inspect(
                &vize_carton::cstr!("import type {{ Other as {name} }} from 'bindings';"),
                SourceType::ts()
            ),
            Some(expected)
        );
    }
    let mut expected = facts(&[], &[]);
    expected.moved_name_binding = true;
    assert_eq!(
        inspect("type \\u0024UsePublicInstance<T> = T;", SourceType::ts()),
        Some(expected)
    );
    assert_eq!(
        inspect(
            "type $ComponentInput<T,D,A> = { authored: true }; type Ref = $UsePublicInstance<unknown>;",
            SourceType::ts()
        ),
        Some(facts(&[], &[])),
        "references are not bindings, and ComponentInput is not a moved callee"
    );
}

#[test]
fn opaque_references_do_not_hide_nested_or_partial_literal_observations() {
    let source = r#"
declare const value: string;
declare const args: string[];
void import(value);
void import(`prefix-${value}`);
void require(value);
void require();
void require(...args);
void require("kept", value);
void import(require("nested"));
"#;
    let mut expected = facts(&["kept", "nested"], &[]);
    expected.opaque_module_reference = true;
    assert_eq!(inspect(source, SourceType::ts()), Some(expected));
}

#[test]
fn reference_markers_include_all_line_endings_and_do_not_resolve_inputs() {
    for source in [
        "/// <reference path='./owned.d.ts' />\nexport {};",
        "\u{feff}/// <reference types='package' />\r\nexport {};",
        "/// \u{feff}<reference path='bom-gap' />\nexport {};",
        "// first\r/// <reference lib='es2022' />\rexport {};",
        "// first\u{2028}/// <reference path='x' />\u{2029}export {};",
        "const note = `\n/// <reference path='lookalike' />\n`;",
    ] {
        let mut expected = facts(&[], &[]);
        expected.reference_directives = true;
        assert_eq!(inspect(source, SourceType::ts()), Some(expected));
    }
    assert_eq!(
        inspect("const note = '/// <reference';", SourceType::ts()),
        Some(facts(&[], &[]))
    );
}

#[test]
fn strict_parse_refuses_partial_recovery_and_respects_the_supplied_source_type() {
    for source in [
        "import 'before'; type Broken = ;",
        "import 'before'; const text = '",
    ] {
        assert!(inspect(source, SourceType::ts()).is_none());
    }
    assert!(inspect("type OnlyTs = string;", SourceType::mjs()).is_none());
    assert_eq!(
        inspect("export const view = <div />;", SourceType::tsx()),
        Some(facts(&[], &[]))
    );
}

#[test]
fn javascript_jsdoc_references_are_unsupported_not_complete_empty_facts() {
    for (source, source_type) in [
        (
            "/** @import {Thing} from 'jsdoc-import' */\nexport {};",
            SourceType::mjs(),
        ),
        (
            "/** @type {import('jsdoc-type').Thing} */\nlet value;",
            SourceType::mjs(),
        ),
        (
            "import('visible');\n/** @type {import('hidden').Thing} */\nlet value;",
            SourceType::mjs(),
        ),
        (
            "/** @import {Thing} from 'jsx-jsdoc' */\nexport const view = <div />;",
            SourceType::jsx(),
        ),
    ] {
        assert_eq!(inspect(source, source_type), None);
    }
}
