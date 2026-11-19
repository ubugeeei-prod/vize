#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions report exact retained-call and transactional laws"
)]

use alloc::vec::Vec;
use oxc_ast::ast::{CallExpression, ChainElement, Expression, Program, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::ProgramReferenceSource;
use crate::resolution::{
    ResolutionErrorKind,
    sink::{ReferenceEvent, ReferenceSink},
};

type LawResult<T = ()> = Result<T, &'static str>;
trait Required<T> {
    fn required(self) -> LawResult<T>;
}
impl<T> Required<T> for Option<T> {
    fn required(self) -> LawResult<T> {
        self.ok_or("missing actual fixture node")
    }
}
impl<T, E> Required<T> for Result<T, E> {
    fn required(self) -> LawResult<T> {
        self.map_err(|_| "fixture operation rejected")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Observed<'a> {
    pointer: *const CallExpression<'a>,
    span: Span,
}
#[derive(Default)]
struct Pending<'a> {
    references: Vec<ReferenceEvent<'a>>,
    calls: Vec<Observed<'a>>,
    reject: Option<Span>,
}
impl<'a> ReferenceSink<'a> for Pending<'a> {
    type Checkpoint = (usize, usize);
    fn checkpoint(&self) -> Self::Checkpoint {
        (self.references.len(), self.calls.len())
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.references.push(event);
        Ok(())
    }
    fn observe_call(
        &mut self,
        call: &CallExpression<'a>,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        self.calls.push(Observed {
            pointer: core::ptr::from_ref(call),
            span,
        });
        if self.reject == Some(span) {
            return Err(ResolutionErrorKind::UnsupportedSyntax);
        }
        Ok(())
    }
    fn rollback(&mut self, checkpoint: Self::Checkpoint) {
        self.references.truncate(checkpoint.0);
        self.calls.truncate(checkpoint.1);
    }
}

fn expression<'p, 'a>(program: &'p Program<'a>, index: usize) -> LawResult<&'p Expression<'a>> {
    let Statement::ExpressionStatement(statement) = program.body.get(index).required()? else {
        return Err("expression fixture");
    };
    Ok(&statement.expression)
}
fn call<'p, 'a>(expression: &'p Expression<'a>) -> LawResult<&'p CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Ok(call),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => Ok(call),
            _ => Err("call fixture"),
        },
        _ => Err("call fixture"),
    }
}

#[test]
fn real_nested_and_chain_calls_observe_original_nodes_once_with_unicode_authored_spans() -> LawResult
{
    let allocator = Allocator::default();
    let file = "🌸<script>/*kept*/ 工場(inner(作者), tail()); optional?.(作者);</script>";
    let content = Span::new("🌸<script>".len() as u32, (file.len() - 9) as u32);
    let raw = file
        .get(content.start as usize..content.end as usize)
        .required()?;
    let parsed = Parser::new(allocator.as_oxc(), raw, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        content,
        SourceType::mjs(),
    )
    .required()?;
    let root = call(expression(source.program(), 0)?)?;
    let inner = call(root.arguments.first().required()?.to_expression())?;
    let tail = call(root.arguments.get(1).required()?.to_expression())?;
    let optional = call(expression(source.program(), 1)?)?;
    let expected = [root, inner, tail, optional].map(|call| Observed {
        pointer: core::ptr::from_ref(call),
        span: Span::new(call.span.start, call.span.end),
    });
    let mut sink = Pending::default();
    source
        .expression(expression(source.program(), 0)?, &mut sink)
        .required()?;
    source
        .expression(expression(source.program(), 1)?, &mut sink)
        .required()?;
    assert_eq!(sink.calls, expected);
    assert_eq!(
        sink.references
            .iter()
            .map(|row| row.name())
            .collect::<Vec<_>>(),
        ["工場", "inner", "作者", "tail", "optional", "作者"]
    );
    let authored = sink
        .calls
        .iter()
        .map(|row| {
            let span = source.authored_span(row.span).required()?;
            file.get(span.start as usize..span.end as usize).required()
        })
        .collect::<LawResult<Vec<_>>>()?;
    assert_eq!(
        authored,
        [
            "工場(inner(作者), tail())",
            "inner(作者)",
            "tail()",
            "optional?.(作者)"
        ]
    );
    Ok(())
}

#[test]
fn later_unsupported_argument_and_observer_refusal_restore_all_prior_rows() -> LawResult {
    let allocator = Allocator::default();
    let file = "seed(); first(keep, () => missing); outer(inner(value));";
    let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        SourceType::mjs(),
    )
    .required()?;
    let mut sink = Pending::default();
    source
        .expression(expression(source.program(), 0)?, &mut sink)
        .required()?;
    let references = sink.references.clone();
    let calls = sink.calls.clone();
    let unsupported = source.expression(expression(source.program(), 1)?, &mut sink);
    assert_eq!(
        unsupported.err().required()?.kind,
        ResolutionErrorKind::UnsupportedSyntax
    );
    assert_eq!(sink.references, references);
    assert_eq!(sink.calls, calls);
    let outer = call(expression(source.program(), 2)?)?;
    let inner = call(outer.arguments.first().required()?.to_expression())?;
    sink.reject = Some(Span::new(inner.span.start, inner.span.end));
    let refused = source.expression(expression(source.program(), 2)?, &mut sink);
    assert_eq!(
        refused.err().required()?.kind,
        ResolutionErrorKind::UnsupportedSyntax
    );
    assert_eq!(sink.references, references);
    assert_eq!(sink.calls, calls);
    Ok(())
}

#[test]
fn invalid_utf8_call_projection_rejects_before_observer_and_preserves_prior_rows() -> LawResult {
    let allocator = Allocator::default();
    let file = "seed(); 作者();";
    let parsed = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    // Mutate a private resolver probe; it never establishes Program admission.
    let mut probe = Parser::new(allocator.as_oxc(), file, SourceType::mjs()).parse();
    let Statement::ExpressionStatement(statement) = probe.program.body.get_mut(1).required()?
    else {
        return Err("actual expression fixture");
    };
    let Expression::CallExpression(call) = &mut statement.expression else {
        return Err("actual call fixture");
    };
    call.span.start += 1; // adversarial retained subtree with an interior UTF-8 endpoint
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        SourceType::mjs(),
    )
    .required()?;
    let mut sink = Pending::default();
    source
        .expression(expression(source.program(), 0)?, &mut sink)
        .required()?;
    let references = sink.references.clone();
    let calls = sink.calls.clone();
    let invalid = source.expression(expression(&probe.program, 1)?, &mut sink);
    assert_eq!(
        invalid.err().required()?.kind,
        ResolutionErrorKind::InvalidSpan
    );
    assert_eq!(sink.references, references);
    assert_eq!(sink.calls, calls);
    Ok(())
}

#[test]
fn direct_eval_and_type_arguments_remain_refused_before_observation() -> LawResult {
    let allocator = Allocator::default();
    let file = "eval(value); generic<Type>(value);";
    let source_type = SourceType::ts().with_module(true);
    let parsed = Parser::new(allocator.as_oxc(), file, source_type).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    assert_eq!(
        parsed.admitted().required()?.program().source_type,
        source_type
    );
    assert_eq!(
        parsed.admitted().required()?.program().span,
        oxc_span::Span::new(0, file.len() as u32)
    );
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        source_type,
    )
    .required()?;
    let mut sink = Pending::default();
    for index in 0..2 {
        let refused = source.expression(expression(source.program(), index)?, &mut sink);
        assert_eq!(
            refused.err().required()?.kind,
            ResolutionErrorKind::UnsupportedSyntax
        );
        assert_eq!(sink.calls, []);
        assert_eq!(sink.references, []);
    }
    Ok(())
}
