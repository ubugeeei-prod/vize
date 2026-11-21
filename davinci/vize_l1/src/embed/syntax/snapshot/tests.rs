use super::*;
use crate::embed::{
    Embed, EmbedSource, Grammar, Lang,
    syntax::{ProgramGoal, ProgramOptions, parse_once, parse_program_once},
};
use core::ptr;
use oxc_ast::ast::{Expression, Statement};
use vize_l0::Span;

fn source(text: &str) -> EmbedSource<'_> {
    EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap()
}

#[test]
fn requesting_each_profile_once_retains_the_exact_original_results() {
    let arena = Allocator::default();
    let input = "void ((a+b))";
    let original = parse_program_once(&arena, source(input), ProgramOptions::module(Lang::Js));
    let snapshot = ProgramSnapshot::observe_printing(&arena, &original).unwrap();
    let checked = snapshot.semantic();
    assert!(ptr::eq(checked, &original));
    let semantic = checked.program().unwrap();
    let printed = snapshot.for_print().unwrap();
    assert!(ptr::eq(printed, snapshot.for_print().unwrap()));
    assert!(!ptr::eq(semantic, printed));
    assert!(ptr::eq(semantic.source_text, input) && ptr::eq(printed.source_text, input));
    let argument = |program: &Program<'_>| {
        let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
            panic!("statement")
        };
        let Expression::UnaryExpression(unary) = &statement.expression else {
            panic!("void")
        };
        matches!(unary.argument, Expression::ParenthesizedExpression(_))
    };
    assert!(argument(semantic));
    assert!(!argument(printed));
}

#[test]
fn returning_outside_a_function_and_invalid_programs_keep_their_profiles_diagnostics() {
    let arena = Allocator::default();
    let make = |text| parse_program_once(&arena, source(text), ProgramOptions::module(Lang::Js));
    let returned = make("return a;");
    let returned = ProgramSnapshot::observe_printing(&arena, &returned).unwrap();
    assert!(returned.for_print().is_ok());
    assert!(returned.semantic().admitted_program().is_none());
    let invalid = make("const = ;");
    let invalid = ProgramSnapshot::observe_printing(&arena, &invalid).unwrap();
    let error = invalid.for_print().unwrap_err();
    assert!(!error.is_empty());
    assert!(ptr::eq(error, invalid.for_print().unwrap_err()));
    assert!(invalid.semantic().admitted_program().is_none());
    assert!(invalid.semantic().diagnostics().count() > 0);
}

#[test]
fn typescript_jsx_script_goal_and_source_offsets_belong_to_one_snapshot() {
    let arena = Allocator::default();
    let document = "前<script>const x = <T>value;</script>";
    let start = document.find("const").unwrap() as u32;
    let end = document.find("</script>").unwrap() as u32;
    let source = EmbedSource::authored(document, Span::new(start, end)).unwrap();
    let normal = ProgramOptions::module(Lang::Ts);
    let ts = parse_program_once(&arena, source, normal);
    let ts = ProgramSnapshot::observe_printing(&arena, &ts).unwrap();
    let tsx = parse_program_once(
        &arena,
        source,
        ProgramOptions {
            jsx: true,
            ..normal
        },
    );
    let tsx = ProgramSnapshot::observe_printing(&arena, &tsx).unwrap();
    assert!(ts.for_print().is_ok() && ts.semantic().admitted_program().is_some());
    assert!(tsx.for_print().is_err() && tsx.semantic().admitted_program().is_none());
    assert_eq!(
        ts.semantic()
            .authored_span(oxc_span::Span::new(0, 5))
            .unwrap(),
        Span::new(start, start + 5)
    );
    let script = parse_program_once(
        &arena,
        source,
        ProgramOptions {
            goal: ProgramGoal::Script,
            ..normal
        },
    );
    let script = ProgramSnapshot::observe_printing(&arena, &script).unwrap();
    assert!(script.semantic().program().unwrap().source_type.is_script());
    assert!(ts.semantic().program().unwrap().source_type.is_module());
}

#[test]
fn a_new_document_observes_new_bytes_without_inheriting_admission_or_diagnostics() {
    let arena = Allocator::default();
    let before = parse_program_once(
        &arena,
        source("const a=;"),
        ProgramOptions::module(Lang::Js),
    );
    let after = parse_program_once(
        &arena,
        source("const a=0"),
        ProgramOptions::module(Lang::Js),
    );
    let before = ProgramSnapshot::observe_printing(&arena, &before).unwrap();
    let after = ProgramSnapshot::observe_printing(&arena, &after).unwrap();
    assert!(before.for_print().is_err());
    assert!(after.for_print().is_ok());
    assert!(after.semantic().admitted_program().is_some());
    assert!(before.semantic().admitted_program().is_none());
}

#[test]
fn wrapped_inputs_cannot_supply_a_whole_program_to_the_printer() {
    let arena = Allocator::default();
    for shape in [
        Shape::Expr,
        Shape::HandlerBody,
        Shape::SlotParams,
        Shape::ForHead,
        Shape::FilterChain,
    ] {
        let syntax = parse_once(
            &arena,
            Embed {
                grammar: Grammar {
                    shape,
                    lang: Lang::Js,
                },
                source: source("value"),
            },
        );
        assert_eq!(
            ProgramSnapshot::observe_printing(&arena, &syntax).err(),
            Some(shape)
        );
    }
}
