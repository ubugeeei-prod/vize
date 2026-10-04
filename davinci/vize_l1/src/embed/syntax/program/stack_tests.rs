extern crate std;

use alloc::vec::Vec;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Comment, Expression, Statement, TSTupleElement, TSType,
};
use oxc_diagnostics::Diagnostics;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String};

use super::{EmbedSource, Lang, ProgramGoal, ProgramOptions, parse_program_once};
use crate::embed::syntax::EmbedHole;

const REPRODUCER: &str = include_str!(
    "../../../../../../tests/fuzz/regressions/l1_program/issue-7805-nested-array.tsx.input"
);
const SMALL_STACK: usize = 256 * 1024;
const REFERENCE_STACK: usize = 128 * 1024 * 1024;
const DEPTH: usize = 8192;

fn options(selector: u8) -> ProgramOptions {
    ProgramOptions {
        lang: if selector & 1 == 0 {
            Lang::Js
        } else {
            Lang::Ts
        },
        jsx: selector & 2 != 0,
        goal: if selector & 4 == 0 {
            ProgramGoal::Module
        } else {
            ProgramGoal::Script
        },
    }
}

fn source(text: &str) -> EmbedSource<'_> {
    EmbedSource::authored(text, Span::new(0, u32::try_from(text.len()).unwrap())).unwrap()
}

fn on_stack<T: Send + 'static>(size: usize, run: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(size)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap()
}

// Diagnostics equality includes every message, severity, code, help, note, URL
// and complete label record. Comment equality includes all parser-owned flags.
#[derive(Debug, PartialEq)]
struct Observation {
    source: String,
    profile: SourceType,
    panicked: bool,
    flow: bool,
    diagnostics: Diagnostics,
    comments: Vec<Comment>,
    admitted: bool,
    body_len: Option<usize>,
}

fn reference(text: &str, options: ProgramOptions) -> Observation {
    let allocator = Allocator::default();
    let parsed = Parser::new(allocator.as_oxc(), text, options.source_type()).parse();
    assert_eq!(parsed.program.source_text, text);
    assert_eq!(parsed.program.source_type, options.source_type());
    let admitted = !parsed.panicked && !parsed.is_flow_language && !parsed.diagnostics.has_errors();
    Observation {
        source: String::from(parsed.program.source_text),
        profile: parsed.program.source_type,
        panicked: parsed.panicked,
        flow: parsed.is_flow_language,
        diagnostics: parsed.diagnostics.clone(),
        comments: parsed.program.comments.to_vec(),
        admitted,
        body_len: admitted.then_some(parsed.program.body.len()),
    }
}

fn native(text: &str, options: ProgramOptions) -> Observation {
    let allocator = Allocator::default();
    let syntax = parse_program_once(&allocator, source(text), options);
    assert_eq!(syntax.source().text(), text);
    assert_eq!(syntax.source().text().as_ptr(), text.as_ptr());
    assert_eq!(syntax.source_type(), options.source_type());
    let observation = syntax.observation.as_ref().unwrap();
    let expected_hole = if observation.is_flow_language() {
        Some(EmbedHole::UnsupportedFlow)
    } else if observation.panicked() || observation.diagnostics().has_errors() {
        Some(EmbedHole::Syntax)
    } else {
        None
    };
    assert_eq!(syntax.hole(), expected_hole);
    if let Some(admitted) = syntax.admitted_program() {
        assert_eq!(admitted.source(), text);
        assert_eq!(admitted.source_type(), options.source_type());
        assert_eq!(admitted.options(), ParseOptions::default());
    }
    for comment in syntax.comments() {
        let span = comment.authored_span().unwrap();
        assert_eq!(comment.decoded_span().unwrap(), span);
        assert_eq!(
            comment.text().unwrap(),
            text.get(span.start as usize..span.end as usize).unwrap()
        );
    }
    for diagnostic in syntax.diagnostics() {
        for label in diagnostic.labels() {
            let span = label.authored_span().unwrap();
            assert_eq!(label.decoded_span().unwrap(), span);
            assert!(text.get(span.start as usize..span.end as usize).is_some());
        }
    }
    let result = Observation {
        source: String::from(syntax.source().text()),
        profile: syntax.source_type(),
        panicked: observation.panicked(),
        flow: observation.is_flow_language(),
        diagnostics: observation.diagnostics().clone(),
        comments: observation.comments().to_vec(),
        admitted: syntax.admitted_program().is_some(),
        body_len: syntax.program().map(|program| program.body.len()),
    };
    // Real owned observations are destroyed before the same parser arena.
    drop(syntax);
    result
}

#[test]
fn original_array_crash_all_eight_profiles_match_complete_large_stack_reference() {
    assert_eq!(REPRODUCER.len(), 6875);
    assert_eq!(
        REPRODUCER.bytes().filter(|byte| *byte == b'[').count(),
        5596
    );
    assert_eq!(REPRODUCER.bytes().filter(|byte| *byte == b']').count(), 0);
    for selector in 0..8 {
        let expected = on_stack(REFERENCE_STACK, move || {
            reference(REPRODUCER, options(selector))
        });
        assert!(expected.panicked);
        assert!(!expected.admitted);
        assert!(expected.diagnostics.has_errors());
        let actual = on_stack(SMALL_STACK, move || native(REPRODUCER, options(selector)));
        assert_eq!(actual, expected, "original selector {selector}");
        assert_eq!(native(REPRODUCER, options(selector)), expected);
    }
}

#[test]
fn array_recovery_and_comments_keep_full_observations_across_stack_sizes() {
    for text in [
        "/* α */ [1,,...[2],]; // β",
        "/* α */ [1, /* β */ ; // γ",
        "/* α */ [...[1],, /* β */ 2,]; // γ",
    ] {
        for selector in 0..8 {
            let expected = on_stack(REFERENCE_STACK, move || reference(text, options(selector)));
            let actual = on_stack(SMALL_STACK, move || native(text, options(selector)));
            assert_eq!(actual, expected, "selector {selector}: {text}");
        }
    }
}

fn assert_deep_array(root: &ArrayExpression<'_>, depth: usize) {
    let mut array = root;
    for level in 0..depth {
        assert_eq!(
            array.span,
            oxc_span::Span::new(level as u32, (2 * depth + 1 - level) as u32)
        );
        assert_eq!(array.elements.len(), 1);
        let element = array.elements.first().unwrap();
        if level + 1 == depth {
            let ArrayExpressionElement::NumericLiteral(leaf) = element else {
                panic!("innermost array retains its actual numeric child")
            };
            assert_eq!(leaf.value, 0.0);
            assert_eq!(
                leaf.span,
                oxc_span::Span::new(depth as u32, depth as u32 + 1)
            );
        } else {
            let ArrayExpressionElement::ArrayExpression(child) = element else {
                panic!("each authored opening bracket retains its actual array node")
            };
            array = child;
        }
    }
}

fn deep_source() -> String {
    let mut text = String::from("[".repeat(DEPTH));
    text.push('0');
    text.push_str(&"]".repeat(DEPTH));
    text.push(';');
    text
}

#[test]
fn valid_deep_arrays_keep_all_actual_nodes_in_eight_small_thread_profiles() {
    on_stack(SMALL_STACK, || {
        let text = deep_source();
        for selector in 0..8 {
            let allocator = Allocator::default();
            let syntax = parse_program_once(&allocator, source(&text), options(selector));
            assert_eq!(syntax.hole(), None);
            assert_eq!(syntax.diagnostics().count(), 0);
            assert_eq!(syntax.comments().count(), 0);
            let program = syntax.program().unwrap();
            assert_eq!(program.source_text, text);
            assert_eq!(program.source_text.as_ptr(), text.as_ptr());
            assert_eq!(program.source_type, options(selector).source_type());
            assert_eq!(program.body.len(), 1);
            let Statement::ExpressionStatement(statement) = program.body.first().unwrap() else {
                panic!("original array expression statement")
            };
            let Expression::ArrayExpression(array) = &statement.expression else {
                panic!("original array root")
            };
            assert_deep_array(array, DEPTH);
            drop(syntax);
        }
    });
}

#[test]
fn original_deep_expression_entry_retains_ast_and_small_thread_unwinds_normally() {
    let unwind = std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            let text = deep_source();
            let expression_text = text.strip_suffix(';').unwrap();
            let allocator = Allocator::default();
            let parsed = Parser::new(allocator.as_oxc(), expression_text, SourceType::mjs())
                .parse_expression()
                .unwrap();
            let Expression::ArrayExpression(array) = &parsed else {
                panic!("original expression root")
            };
            assert_deep_array(array, DEPTH);
            // A real retained AST and arena drop during ordinary caller unwind.
            panic!("expected deep-expression caller unwind");
        })
        .unwrap()
        .join()
        .unwrap_err();
    assert_eq!(
        unwind.downcast_ref::<&str>(),
        Some(&"expected deep-expression caller unwind")
    );
}

#[test]
fn valid_deep_tuple_types_keep_all_original_nodes_on_small_threads() {
    on_stack(SMALL_STACK, || {
        let mut text = String::from("type T = ");
        let start = text.len() as u32;
        text.push_str(&"[".repeat(DEPTH));
        text.push_str(&"]".repeat(DEPTH));
        text.push(';');
        for selector in [1, 3, 5, 7] {
            let allocator = Allocator::default();
            let syntax = parse_program_once(&allocator, source(&text), options(selector));
            assert_eq!(syntax.hole(), None);
            assert_eq!(syntax.diagnostics().count(), 0);
            let program = syntax.program().unwrap();
            assert_eq!(program.source_text.as_ptr(), text.as_ptr());
            assert_eq!(program.body.len(), 1);
            let Statement::TSTypeAliasDeclaration(alias) = program.body.first().unwrap() else {
                panic!("original type alias statement")
            };
            let TSType::TSTupleType(root) = &alias.type_annotation else {
                panic!("original tuple type root")
            };
            let mut tuple = &**root;
            for level in 0..DEPTH {
                assert_eq!(
                    tuple.span,
                    oxc_span::Span::new(start + level as u32, start + (2 * DEPTH - level) as u32)
                );
                if level + 1 == DEPTH {
                    assert_eq!(tuple.element_types.len(), 0);
                } else {
                    assert_eq!(tuple.element_types.len(), 1);
                    let TSTupleElement::TSTupleType(child) = tuple.element_types.first().unwrap()
                    else {
                        panic!("each original tuple retains its actual nested type")
                    };
                    tuple = child;
                }
            }
        }
    });
}
