#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions report original JSX subtree identity and transactional span laws"
)]

use super::ProgramReferenceSource;
use crate::expr::JsExpr;
use crate::resolution::{
    ResolutionErrorKind,
    sink::{ReferenceEvent, ReferenceSink},
    walk,
};
use alloc::vec::Vec;
use oxc_ast::ast::{CallExpression, Expression, JSXAttributeItem, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

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

#[derive(Default)]
struct Pending<'a> {
    rows: Vec<ReferenceEvent<'a>>,
    calls: Vec<(*const CallExpression<'a>, Span)>,
    invocations: Vec<(*const Expression<'a>, Span)>,
    reject: Option<&'static str>,
}
impl<'a> ReferenceSink<'a> for Pending<'a> {
    type Checkpoint = (usize, usize, usize);
    fn checkpoint(&self) -> Self::Checkpoint {
        (self.rows.len(), self.calls.len(), self.invocations.len())
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        if self.reject == Some(event.name()) {
            return Err(ResolutionErrorKind::MissingBinding);
        }
        self.rows.push(event);
        Ok(())
    }
    fn observe_call(
        &mut self,
        call: &CallExpression<'a>,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        self.calls.push((core::ptr::from_ref(call), span));
        Ok(())
    }
    fn observe_invocation(
        &mut self,
        expression: &Expression<'a>,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        self.invocations
            .push((core::ptr::from_ref(expression), span));
        Ok(())
    }
    fn rollback(&mut self, checkpoint: Self::Checkpoint) {
        self.rows.truncate(checkpoint.0);
        self.calls.truncate(checkpoint.1);
        self.invocations.truncate(checkpoint.2);
    }
}
fn expression<'p, 'a>(
    program: &'p oxc_ast::ast::Program<'a>,
    index: usize,
) -> LawResult<&'p Expression<'a>> {
    let Statement::ExpressionStatement(statement) = program.body.get(index).required()? else {
        return Err("expression fixture");
    };
    Ok(&statement.expression)
}

#[test]
fn existing_call_and_all_invocation_hooks_run_once_inside_actual_jsx_containers() -> LawResult {
    let arena = Allocator::default();
    let file = "<Comp a={run(value)}>{new Maker(value)}{tag`x${value}`}{import('dep')}</Comp>;";
    let profile = SourceType::jsx();
    let parsed = Parser::new(&arena, file, profile).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        profile,
    )
    .required()?;
    assert!(core::ptr::eq(
        source.program(),
        parsed.admitted().required()?.program()
    ));
    let original = expression(source.program(), 0)?;
    let Expression::JSXElement(element) = original else {
        return Err("JSX root");
    };
    let JSXAttributeItem::Attribute(attribute) =
        element.opening_element.attributes.first().required()?
    else {
        return Err("attribute");
    };
    let Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) = &attribute.value
    else {
        return Err("container");
    };
    let Expression::CallExpression(call) = container.expression.as_expression().required()? else {
        return Err("call");
    };
    let mut sink = Pending::default();
    source.expression(original, &mut sink).required()?;
    assert_eq!(
        sink.calls,
        [(
            core::ptr::from_ref(call.as_ref()),
            Span::new(call.span.start, call.span.end)
        )]
    );
    assert_eq!(
        sink.rows.iter().map(|row| row.name()).collect::<Vec<_>>(),
        ["Comp", "run", "value", "Maker", "value", "tag", "value"]
    );
    let invocations = sink
        .invocations
        .iter()
        .map(|(_, span)| file.get(span.start as usize..span.end as usize).required())
        .collect::<LawResult<Vec<_>>>()?;
    assert_eq!(
        invocations,
        ["new Maker(value)", "tag`x${value}`", "import('dep')"]
    );
    assert_eq!(
        sink.rows
            .iter()
            .map(|row| row.constructor())
            .collect::<Vec<_>>(),
        [false, false, false, true, false, false, false]
    );
    Ok(())
}

#[test]
fn late_unsupported_and_sink_refusal_restore_seed_rows_calls_and_invocations() -> LawResult {
    let arena = Allocator::default();
    let file = "seed(); <Comp>{new Maker(value)}{run(value)}<ns:tag/></Comp>; <Comp>{run(value)}<Other/></Comp>;";
    let profile = SourceType::jsx();
    let parsed = Parser::new(&arena, file, profile).parse_observed();
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        profile,
    )
    .required()?;
    let mut sink = Pending::default();
    source
        .expression(expression(source.program(), 0)?, &mut sink)
        .required()?;
    let rows = sink.rows.clone();
    let calls = sink.calls.clone();
    let invocations = sink.invocations.clone();
    assert_eq!(
        source
            .expression(expression(source.program(), 1)?, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::UnsupportedSyntax
    );
    assert_eq!(sink.rows, rows);
    assert_eq!(sink.calls, calls);
    assert_eq!(sink.invocations, invocations);
    sink.reject = Some("Other");
    assert_eq!(
        source
            .expression(expression(source.program(), 2)?, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::MissingBinding
    );
    assert_eq!(sink.rows, rows);
    assert_eq!(sink.calls, calls);
    assert_eq!(sink.invocations, invocations);
    Ok(())
}

#[test]
fn invalid_unicode_attribute_leaf_rejects_before_its_runtime_callback_and_rolls_back() -> LawResult
{
    let arena = Allocator::default();
    let file = "seed(); <Comp 作者={run(value)}/>;";
    let profile = SourceType::jsx();
    let parsed = Parser::new(&arena, file, profile).parse_observed();
    // Mutate a private resolver probe; it never establishes Program admission.
    let mut probe = Parser::new(&arena, file, profile).parse();
    let Statement::ExpressionStatement(statement) = probe.program.body.get_mut(1).required()?
    else {
        return Err("statement");
    };
    let Expression::JSXElement(element) = &mut statement.expression else {
        return Err("JSX");
    };
    let JSXAttributeItem::Attribute(attribute) =
        element.opening_element.attributes.first_mut().required()?
    else {
        return Err("attribute");
    };
    let oxc_ast::ast::JSXAttributeName::Identifier(name) = &mut attribute.name else {
        return Err("name");
    };
    name.span.start += 1;
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file,
        Span::new(0, file.len() as u32),
        profile,
    )
    .required()?;
    let mut sink = Pending::default();
    source
        .expression(expression(source.program(), 0)?, &mut sink)
        .required()?;
    let rows = sink.rows.clone();
    let calls = sink.calls.clone();
    assert_eq!(
        source
            .expression(expression(&probe.program, 1)?, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::InvalidSpan
    );
    assert_eq!(sink.rows, rows);
    assert_eq!(sink.calls, calls);
    assert_eq!(sink.invocations, []);
    Ok(())
}

#[test]
fn raw_expression_route_does_not_acquire_program_jsx_or_target_authority() -> LawResult {
    let arena = Allocator::default();
    let file = "<Comp/>;";
    let parsed = Parser::new(&arena, file, SourceType::jsx()).parse_observed();
    let original = expression(parsed.admitted().required()?.program(), 0)?;
    let source = file.get(..7).required()?;
    let expression = JsExpr {
        ast: original,
        source,
        span: Span::new(0, 7),
        coordinates: None,
    };
    let mut sink = Pending::default();
    assert_eq!(
        walk::expression(&expression, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::UnsupportedSyntax
    );
    assert_eq!(sink.rows, []);
    assert_eq!(sink.calls, []);
    assert_eq!(sink.invocations, []);
    Ok(())
}

#[test]
fn actual_fragment_container_work_is_bounded_by_the_existing_resolver_budget() -> LawResult {
    let arena = Allocator::default();
    let mut file = vize_l0::String::from("<>");
    for _ in 0..1100 {
        file.push_str("{value}");
    }
    file.push_str("</>;");
    let profile = SourceType::jsx();
    let parsed = Parser::new(&arena, file.as_str(), profile).parse_observed();
    assert!(!parsed.panicked() && !parsed.diagnostics().has_errors());
    let source = ProgramReferenceSource::checked(
        parsed.admitted().required()?,
        file.as_str(),
        Span::new(0, file.len() as u32),
        profile,
    )
    .required()?;
    let mut sink = Pending::default();
    assert_eq!(
        source
            .expression(expression(source.program(), 0)?, &mut sink)
            .err()
            .required()?
            .kind,
        ResolutionErrorKind::TraversalLimit
    );
    assert_eq!(sink.rows, []);
    assert_eq!(sink.calls, []);
    assert_eq!(sink.invocations, []);
    Ok(())
}

mod names;
