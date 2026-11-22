use super::*;
use alloc::{format, vec};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};

#[test]
fn exact_nine_primitive_judgments_and_original_ranges() {
    use Primitive::{Boolean, Number, String};
    let arena = Allocator::new();
    for (annotation, expected) in [("boolean", Boolean), ("number", Number), ("string", String)] {
        for (literal, actual) in [("true", Boolean), ("0x2a", Number), ("'🦀'", String)] {
            let script = format!("export const 雪: {annotation} = {literal};");
            let source = format!("<!--葉🦀-->\r\n<script lang=ts>{script}</script>");
            let offset = source.find(&script).unwrap();
            let block = SourceRoot::new(&source)
                .unwrap()
                .block(&source[offset..offset + script.len()], offset as u32)
                .unwrap();
            let program = AuthoredProgram::parse(&arena, block, SourceType::tsx());
            let query = LiteralAssignments::new(&program).unwrap();
            let rows = query.prove();
            let checked = query.check(&rows).unwrap();
            assert!(checked.is_complete());
            let [Row::Assignment(row)] = rows.as_slice() else {
                panic!("one assignment")
            };
            assert_eq!(
                (
                    row.binding.slice(&source),
                    row.annotation.slice(&source),
                    row.initializer.slice(&source)
                ),
                ("雪", annotation, literal)
            );
            assert_eq!(
                (row.expected, row.actual, row.exported, row.kind),
                (expected, actual, true, VariableDeclarationKind::Const)
            );
            assert_eq!(
                row.verdict,
                if expected == actual {
                    Verdict::Assignable
                } else {
                    Verdict::Mismatch
                }
            );
        }
    }
}

#[test]
fn a_claim_cannot_hide_reorder_or_change_a_supported_judgment() {
    let text = "let first: string = true, 雪: number = 9; f(); const last = 8;";
    let arena = Allocator::new();
    let program = AuthoredProgram::parse(
        &arena,
        SourceRoot::new(text).unwrap().whole_block(),
        SourceType::ts(),
    );
    let query = LiteralAssignments::new(&program).unwrap();
    let rows = query.prove();
    let [
        Row::Assignment(first),
        Row::Assignment(second),
        Row::Outside(call),
        Row::Outside(last),
    ] = rows.as_slice()
    else {
        panic!("all four slots")
    };
    assert_eq!((call.slice(text), last.slice(text)), ("f();", "last = 8"));
    assert_eq!(query.check(&rows).unwrap().assignments().count(), 2);
    assert!(!query.check(&rows).unwrap().is_complete());
    for (replacement, error) in [
        (Row::Outside(first.binding), AssignmentError::Outside),
        (Row::Assignment(*second), AssignmentError::Source),
        (
            Row::Assignment(Assignment {
                initializer: first.binding,
                ..*first
            }),
            AssignmentError::Source,
        ),
        (
            Row::Assignment(Assignment {
                kind: VariableDeclarationKind::Const,
                ..*first
            }),
            AssignmentError::Source,
        ),
        (
            Row::Assignment(Assignment {
                exported: true,
                ..*first
            }),
            AssignmentError::Source,
        ),
        (
            Row::Assignment(Assignment {
                actual: Primitive::Number,
                ..*first
            }),
            AssignmentError::Type,
        ),
        (
            Row::Assignment(Assignment {
                expected: Primitive::Number,
                ..*first
            }),
            AssignmentError::Type,
        ),
        (
            Row::Assignment(Assignment {
                verdict: Verdict::Assignable,
                ..*first
            }),
            AssignmentError::Verdict,
        ),
    ] {
        let mut forged = rows.clone();
        forged[0] = replacement;
        assert_eq!(query.check(&forged).err(), Some(error));
    }
    assert_eq!(
        query.check(&rows[..3]).err(),
        Some(AssignmentError::Missing)
    );
    let mut extra = rows.clone();
    extra.push(rows[0]);
    assert_eq!(query.check(&extra).err(), Some(AssignmentError::Extra));
    let another = AuthoredProgram::parse(
        &arena,
        SourceRoot::new("let a: number = 4;").unwrap().whole_block(),
        SourceType::ts(),
    );
    assert!(
        LiteralAssignments::new(&another)
            .unwrap()
            .check(&rows)
            .is_err()
    );
}

#[test]
fn unsupported_typed_syntax_does_not_turn_into_a_local_diagnostic() {
    let arena = Allocator::new();
    let text = "; declare const ambient: number; const {x} = y; if (true) { let x: number = false; } let n!: number; const s = '@ts-ignore'; let v: string = value; export type A = number;";
    let program = AuthoredProgram::parse(
        &arena,
        SourceRoot::new(text).unwrap().whole_block(),
        SourceType::ts(),
    );
    let query = LiteralAssignments::new(&program).unwrap();
    let rows = query.prove();
    let checked = query.check(&rows).unwrap();
    assert_eq!(checked.assignments().count(), 0);
    assert_eq!(
        rows.iter()
            .map(|r| match r {
                Row::Outside(s) => s.slice(text),
                _ => panic!("outside"),
            })
            .collect::<Vec<_>>(),
        vec![
            "declare const ambient: number;",
            "{x} = y",
            "if (true) { let x: number = false; }",
            "n!: number",
            "s = '@ts-ignore'",
            "v: string = value",
            "export type A = number;"
        ]
    );
    for (text, ty, error) in [
        (
            "let x: string = 1",
            SourceType::mjs(),
            ScriptRefusal::Grammar,
        ),
        (
            "declare const x: string",
            SourceType::d_ts(),
            ScriptRefusal::Grammar,
        ),
        (
            "// @ts-ignore\nlet x: string = 1",
            SourceType::ts(),
            ScriptRefusal::DiagnosticDirective,
        ),
        (
            "/* @TS-NOCHECK */ let x: string = 1",
            SourceType::ts(),
            ScriptRefusal::DiagnosticDirective,
        ),
    ] {
        let p = AuthoredProgram::parse(&arena, SourceRoot::new(text).unwrap().whole_block(), ty);
        assert_eq!(LiteralAssignments::new(&p).err(), Some(error));
    }
    let p = AuthoredProgram::parse(
        &arena,
        SourceRoot::new("let ok: number = 1; let broken: = ;")
            .unwrap()
            .whole_block(),
        SourceType::ts(),
    );
    assert!(matches!(
        LiteralAssignments::new(&p),
        Err(ScriptRefusal::Parse(_))
    ));
}
