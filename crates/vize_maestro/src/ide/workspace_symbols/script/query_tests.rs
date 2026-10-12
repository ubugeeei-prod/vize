//! Preserve complete authored symbol packets when an absent query skips parsing.

use super::*;

fn packet(source: &str, query: &str) -> serde_json::Value {
    let uri = Url::parse("file:///workspace/module.ts").unwrap();
    let mut symbols = Vec::new();
    collect(&uri, source, query, &mut symbols);
    serde_json::to_value(symbols).unwrap()
}

#[test]
fn ascii_case_matches_preserve_every_top_level_declaration_and_range() {
    let source = "export const Exact = 1;\nexport function ExactFunction() {}\nexport class ExactClass {}\nexport interface ExactInterface {}\nexport type ExactType = number;\nexport enum ExactEnum {}\n";
    assert_eq!(
        packet(source, "exact"),
        serde_json::json!([
            { "name": "Exact", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 18 }
                }
            } },
            { "name": "ExactFunction", "kind": 12, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 1, "character": 16 }, "end": { "line": 1, "character": 29 }
                }
            } },
            { "name": "ExactClass", "kind": 5, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 2, "character": 13 }, "end": { "line": 2, "character": 23 }
                }
            } },
            { "name": "ExactInterface", "kind": 11, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 3, "character": 17 }, "end": { "line": 3, "character": 31 }
                }
            } },
            { "name": "ExactType", "kind": 26, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 4, "character": 12 }, "end": { "line": 4, "character": 21 }
                }
            } },
            { "name": "ExactEnum", "kind": 10, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 5, "character": 12 }, "end": { "line": 5, "character": 21 }
                }
            } }
        ])
    );
}

#[test]
fn decoded_and_unicode_names_keep_original_utf16_ranges() {
    assert_eq!(
        packet("export const \\u0066oo = 1;\n", "foo"),
        serde_json::json!([
            { "name": "foo", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 21 }
                }
            } }
        ])
    );
    assert_eq!(
        packet("export const Klass = 1;\n", "klass"),
        serde_json::json!([
            { "name": "Klass", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 18 }
                }
            } }
        ])
    );
    assert_eq!(
        packet("export const İ = 1;\n", "i"),
        serde_json::json!([
            { "name": "İ", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 14 }
                }
            } }
        ])
    );
}

#[test]
fn empty_unicode_and_long_queries_keep_original_results() {
    assert_eq!(
        packet("export const x = 1;", ""),
        serde_json::json!([
            { "name": "x", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 14 }
                }
            } }
        ])
    );
    assert_eq!(
        packet("export const café = 1;", "café"),
        serde_json::json!([
            { "name": "café", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 17 }
                }
            } }
        ])
    );
    assert_eq!(
        packet(
            "export const aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa = 1;",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ),
        serde_json::json!([
            { "name": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 46 }
                }
            } }
        ])
    );
}

#[test]
fn a_query_at_the_scan_boundary_keeps_case_and_range() {
    assert_eq!(
        packet(
            "export const abcdefghijklmnopqrstuvwxyzABCDEF = 1;",
            "abcdefghijklmnopqrstuvwxyzabcdef"
        ),
        serde_json::json!([
            { "name": "abcdefghijklmnopqrstuvwxyzABCDEF", "kind": 13, "location": {
                "uri": "file:///workspace/module.ts", "range": {
                    "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 45 }
                }
            } }
        ])
    );
}

#[test]
fn raw_query_hits_do_not_make_ignored_nodes_into_symbols() {
    for source in [
        "// exact\nexport const unrelated = 1;",
        "export const unrelated = 'exact';",
        "import { exact } from './other';",
        "export default function exact() {}",
        "const { exact } = object;",
        "function outer() { const exact = 1; }",
    ] {
        assert_eq!(packet(source, "exact"), serde_json::json!([]), "{source}");
    }
}

#[test]
fn absent_queries_preserve_prior_packets_and_invalid_source_stays_empty() {
    let uri = Url::parse("file:///workspace/module.ts").unwrap();
    let mut symbols = Vec::new();
    collect(&uri, "export const retained = 1;", "retained", &mut symbols);
    let before = serde_json::to_value(&symbols).unwrap();
    collect(&uri, "export const unrelated = 2;", "absent", &mut symbols);
    assert_eq!(serde_json::to_value(symbols).unwrap(), before);

    for source in [
        "export const exact = ;",
        "export const = 1;",
        "export function () {}",
        "export interface {}",
        "export type = number;",
        "export enum {}",
    ] {
        assert_eq!(packet(source, "exact"), serde_json::json!([]), "{source}");
        assert_eq!(packet(source, ""), serde_json::json!([]), "{source}");
    }
}
