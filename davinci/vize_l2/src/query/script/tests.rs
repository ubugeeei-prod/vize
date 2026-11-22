use super::*;
use alloc::{format, vec::Vec};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};

#[test]
fn ordered_direct_names_keep_forms_raw_spans_and_declaration_merging() {
    let script = r#"import Default, { type Spec as T, value as v } from 'm';
import type * as N from 'types';
export const { [v]: [first = Default, , ...tail], ...rest } = object;
const \u0061 = 1;
export function fn() { const local = 1; }
export class Cls { field = first }
enum Mode { A }
export enum Mode { B }
type A = string; interface I {}
namespace NS { export const inner = 1; }
import Alias = require('m');
export default function DefaultFn() {}
export * from 'other'; export { v as exported };
"#;
    let source = format!("<!--🧪-->\r\n<script lang='ts'>{script}</script>");
    let offset = source.find(script).unwrap();
    let arena = Allocator::new();
    let root = SourceRoot::new(&source).unwrap();
    let block = root
        .block(&source[offset..offset + script.len()], offset as u32)
        .unwrap();
    let original = AuthoredProgram::parse(&arena, block, SourceType::ts());
    let model = ScriptDeclarations::new(&original).unwrap();
    let mut rows = Vec::new();
    model.visit(|row| rows.push((row.name, row.span.slice(&source), row.kind, row.export)));
    use DeclarationExport::{Default, Local, Named};
    use ScriptDeclarationKind::*;
    let variable = Variable(VariableDeclarationKind::Const);
    assert_eq!(
        rows,
        [
            ("Default", "Default", Import { type_only: false }, Local),
            ("T", "T", Import { type_only: true }, Local),
            ("v", "v", Import { type_only: false }, Local),
            ("N", "N", Import { type_only: true }, Local),
            ("first", "first", variable, Named),
            ("tail", "tail", variable, Named),
            ("rest", "rest", variable, Named),
            ("a", r"\u0061", variable, Local),
            ("fn", "fn", Function, Named),
            ("Cls", "Cls", Class, Named),
            ("Mode", "Mode", Enum, Local),
            ("Mode", "Mode", Enum, Named),
            ("A", "A", TypeAlias, Local),
            ("I", "I", Interface, Local),
            ("NS", "NS", Namespace, Local),
            ("Alias", "Alias", ImportEquals { type_only: false }, Local),
            ("DefaultFn", "DefaultFn", Function, Default)
        ]
    );
}

#[test]
fn parser_recovery_cannot_publish_an_earlier_declaration() {
    let arena = Allocator::new();
    for source in [
        "const earlier=1;const broken=",
        "const x=1; const x=2; export { missing };",
    ] {
        let input = AuthoredProgram::parse(
            &arena,
            SourceRoot::new(source).unwrap().whole_block(),
            SourceType::mjs(),
        );
        let parsed = input.parsed();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            assert!(ScriptDeclarations::new(&input).is_err());
        } else {
            // Scope diagnostics are a separate semantic demand. The recovered
            // syntax policy uses exactly the original parser diagnostics.
            let mut count = 0;
            ScriptDeclarations::new(&input)
                .unwrap()
                .visit(|_| count += 1);
            assert_eq!(count, 2);
        }
    }
    let empty = AuthoredProgram::parse(
        &arena,
        SourceRoot::new("").unwrap().whole_block(),
        SourceType::mjs(),
    );
    ScriptDeclarations::new(&empty)
        .unwrap()
        .visit(|_| panic!("empty Program"));
}
