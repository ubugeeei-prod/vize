//! Source geometry for the independently preserved pre-policy RAW native packet.
//!
//! Fixture custody, the original four-entry failure, and complete generated inputs
//! are retained in tests/fixtures/typechecker/outside-import-editor-raw-10695.

use std::path::Path;

use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_span::{SourceType, Span};
use serde_json::{Value, json};
use vize_carton::String;

const PACKET: &str = include_str!(
    "../../../../../tests/fixtures/typechecker/outside-import-editor-raw-10695/before-policy-diagnostics.json.txt"
);
const HELPERS: [&str; 5] = [
    "__Ref",
    "__VizeUnownedComponentAttrs",
    "__VizePrettify",
    "__VizeComponentFallthroughProps",
    "__VizeComponentMissingInputGuard",
];

pub(super) fn expected(code: &str, target: &Path) -> Value {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let target = target.to_str().unwrap().replace('\\', "/");
    let imports: Vec<_> = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::ImportDeclaration(import) = statement else {
                return None;
            };
            if import.source.value.as_str() != target {
                return None;
            }
            assert_eq!(import.import_kind, ImportOrExportKind::Value);
            let specifiers = import.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            let ImportDeclarationSpecifier::ImportSpecifier(greet) = specifiers.first().unwrap()
            else {
                panic!("the authored named greet import must remain named");
            };
            assert_eq!(greet.import_kind, ImportOrExportKind::Value);
            assert_eq!(greet.local.name.as_str(), "greet");
            assert_eq!(greet.imported.name().as_str(), "greet");
            Some(import.source.span)
        })
        .collect();
    assert_eq!(imports.len(), 1, "exactly one admitted import literal");
    let mut packet: Value = serde_json::from_str(PACKET).unwrap();
    let entries = packet.as_array_mut().unwrap();
    assert_eq!(entries.len(), 1 + HELPERS.len());
    *entries.first_mut().unwrap().get_mut("range").unwrap() =
        range(code, *imports.first().unwrap());
    for (entry, helper) in entries.iter_mut().skip(1).zip(HELPERS) {
        let aliases: Vec<_> = parsed
            .program
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::TSTypeAliasDeclaration(alias) if alias.id.name.as_str() == helper => {
                    Some(alias.id.span)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            aliases.len(),
            1,
            "exactly one top-level type alias {helper}"
        );
        *entry.get_mut("range").unwrap() = range(code, *aliases.first().unwrap());
    }
    packet
}

fn range(code: &str, span: Span) -> Value {
    let position = |byte: u32| {
        let prefix = code.get(..byte as usize).unwrap();
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
        let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count();
        json!({ "line": line, "character": column })
    };
    json!({ "start": position(span.start), "end": position(span.end) })
}

const BASELINE: &str = include_str!(
    "../../../../../tests/fixtures/typechecker/outside-import-editor-raw-10695/before-policy-generated.vue.ts.txt"
);
const AUTHORITY: &str = include_str!(
    "../../../../../tests/fixtures/typechecker/outside-import-editor-raw-10695/authority.json"
);

#[test]
fn unchanged_pre_policy_packet_matches_its_independent_generated_ast() {
    let authority: Value = serde_json::from_str(AUTHORITY).unwrap();
    assert_eq!(
        expected(
            BASELINE,
            Path::new(authority["generatedImport"].as_str().unwrap())
        ),
        serde_json::from_str::<Value>(PACKET).unwrap()
    );
}

fn synthetic_code(target: &str) -> String {
    let literal = serde_json::to_string(target).unwrap();
    let mut code = String::from(format!(
        "// 日本語🍣\r\nimport {{ greet }} from {literal};\r\n"
    ));
    for name in HELPERS {
        code.push_str(&format!("type {name} = 0;\r\n"));
    }
    code
}

#[test]
fn literal_geometry_tracks_utf16_escaped_quotes_crlf_and_different_root_lengths() {
    for target in [
        "/x/index.ts",
        "/long/日本語🍣/quoted\"root/packages/web/src/lib/greet/index.ts",
    ] {
        let packet = expected(&synthetic_code(target), Path::new(target));
        assert_eq!(
            packet
                .as_array()
                .unwrap()
                .first()
                .unwrap()
                .get("range")
                .unwrap(),
            &json!({"start":{"line":1,"character":22},"end":{"line":1,"character":22 + serde_json::to_string(target).unwrap().encode_utf16().count()}})
        );
    }
}

#[test]
#[should_panic(expected = "exactly one admitted import literal")]
fn an_unrelated_string_is_not_import_geometry_authority() {
    let code = synthetic_code("/x/index.ts").replace("import { greet } from", "const example =");
    expected(&code, Path::new("/x/index.ts"));
}

#[test]
#[should_panic(expected = "exactly one top-level type alias __Ref")]
fn duplicate_helper_declarations_are_not_position_authority() {
    let mut code = synthetic_code("/x/index.ts");
    code.push_str("type __Ref = 1;\n");
    expected(&code, Path::new("/x/index.ts"));
}
