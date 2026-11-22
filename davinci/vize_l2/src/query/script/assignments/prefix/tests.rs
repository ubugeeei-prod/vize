use super::*;
use crate::query::script::assignments::{LiteralAssignments, Primitive, Verdict};
use alloc::{format, vec};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::authored_program::AuthoredProgram;

fn block<'a>(source: &'a str, start: usize, end: usize) -> SourceBlock<'a> {
    SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap()
}
fn index<'a>(original: SourceBlock<'a>) -> LiteralPrefixIndex<'a> {
    let arena = Allocator::new();
    let syntax = AuthoredProgram::parse(&arena, original, SourceType::ts());
    let q = LiteralAssignments::new(&syntax).unwrap();
    let rows = q.prove();
    LiteralPrefixIndex::collect(original, &q.check(&rows).unwrap()).unwrap()
}

#[test]
fn every_truncation_agrees_with_a_fresh_parser_on_its_precise_original_extent() {
    let program = "// 🦀\r\nexport const 最初: number=4;\r\nlet second: string = true, third: string = '葉;🦀';\r\ncall();\nvar last: boolean=false;\r\nconst asi: number=9";
    let first = program.find(";\r").unwrap() + 1;
    let second = program.find("';\r").unwrap() + 2;
    let last = program.find("false;").unwrap() + "false;".len();
    let old = format!("<!--💠--><script lang=ts>{program}</script>");
    let at = old.find("//").unwrap();
    let retained = index(block(&old, at, at + program.len()));
    for (end, _) in program
        .char_indices()
        .chain(core::iter::once((program.len(), ' ')))
    {
        let prefix = &program[..end];
        let newer = format!("<script setup lang='ts'>{prefix}§ const unfinished: = ;</script>");
        let start = newer.find("//").unwrap_or("<script setup lang='ts'>".len());
        let stop = newer.rfind("</script>").unwrap();
        let view = retained
            .revalidate(block(&newer, start, stop), SourceType::ts())
            .unwrap();
        let bound = if end >= last {
            last
        } else if end >= second {
            second
        } else if end >= first {
            first
        } else {
            0
        };
        assert_eq!(
            view.covered(),
            Span::new(start as u32, (start + bound) as u32),
            "edit at {end}"
        );
        let arena = Allocator::new();
        let syntax = AuthoredProgram::parse(
            &arena,
            block(&newer, start, start + bound),
            SourceType::ts(),
        );
        let q = LiteralAssignments::new(&syntax).unwrap();
        let proof = q.prove();
        let oracle = q.check(&proof).unwrap();
        assert_eq!(
            view.assignments().collect::<Vec<_>>(),
            oracle.assignments().copied().collect::<Vec<_>>(),
            "{end}"
        );
        for row in view.assignments() {
            assert_eq!(view.binding_at(row.binding.start), Some(row));
            assert_eq!(view.binding_at(row.binding.end), None);
        }
    }
}

#[test]
fn each_terminated_statement_is_a_unit_and_an_asi_or_partial_declarator_is_withheld() {
    let old =
        "let first: string = 1, next: boolean = false;\nlet asi: number=2\nconst last: number=4;";
    let idx = index(SourceRoot::new(old).unwrap().whole_block());
    let inputs = [
        ("let first: string = 1, next: boolean = ", 0),
        (
            "let first: string = 1, next: boolean = false;\nlet asi: number=2\n(",
            2,
        ),
        (
            "let first: string = 1, next: boolean = false;\nlet asi: number=2\nconst last: number=4;/*",
            4,
        ),
        (
            "let FIRST: string = 1, next: boolean = false;\nlet asi: number=2\nconst last: number=4;",
            0,
        ),
    ];
    for (now, count) in inputs {
        let found = idx
            .revalidate(
                SourceRoot::new(now).unwrap().whole_block(),
                SourceType::ts(),
            )
            .unwrap();
        assert_eq!(found.assignments().len(), count);
    }
    let complete = idx
        .revalidate(
            SourceRoot::new(old).unwrap().whole_block(),
            SourceType::ts(),
        )
        .unwrap();
    assert_eq!(
        complete
            .assignments()
            .map(|a| (a.binding.slice(old), a.expected, a.verdict))
            .collect::<Vec<_>>(),
        vec![
            ("first", Primitive::String, Verdict::Mismatch),
            ("next", Primitive::Boolean, Verdict::Assignable),
            ("asi", Primitive::Number, Verdict::Assignable),
            ("last", Primitive::Number, Verdict::Assignable)
        ]
    );
}

#[test]
fn diagnostic_directives_grammar_and_foreign_owners_cannot_reuse_a_prefix() {
    let old = "const x: number=true;";
    let arena = Allocator::new();
    let original = SourceRoot::new(old).unwrap().whole_block();
    let parsed = AuthoredProgram::parse(&arena, original, SourceType::ts());
    let q = LiteralAssignments::new(&parsed).unwrap();
    let rows = q.prove();
    let checked = q.check(&rows).unwrap();
    let copy = format!("{old}");
    assert_eq!(
        LiteralPrefixIndex::collect(SourceRoot::new(&copy).unwrap().whole_block(), &checked).err(),
        Some(PrefixRefusal::ForeignSource)
    );
    let idx = LiteralPrefixIndex::collect(original, &checked).unwrap();
    assert_eq!(
        idx.revalidate(original, SourceType::tsx()).err(),
        Some(PrefixRefusal::Grammar)
    );
    for tail in ["// @TS-NOCHECK", "/* @ts-ignore */", "'@ts-expect-error'"] {
        let text = format!("{old}{tail}");
        assert_eq!(
            idx.revalidate(
                SourceRoot::new(&text).unwrap().whole_block(),
                SourceType::ts()
            )
            .err(),
            Some(PrefixRefusal::DiagnosticDirective)
        );
    }
    let far = "x".repeat(130);
    let src = format!("const {far}: string='a'; const second: number=7;");
    let view = index(SourceRoot::new(&src).unwrap().whole_block());
    for end in [0, 1, 63, 64, 65, 127, 128, 129, src.len()] {
        let now = format!("{}!", &src[..end]);
        let data = view
            .revalidate(
                SourceRoot::new(&now).unwrap().whole_block(),
                SourceType::ts(),
            )
            .unwrap();
        assert_eq!(
            data.assignments().len(),
            if end == src.len() { 2 } else { 0 }
        );
    }
}
