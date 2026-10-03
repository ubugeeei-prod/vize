#![expect(
    clippy::panic_in_result_fn,
    reason = "assertions verify projection before structural callbacks and transactional rollback"
)]

use super::*;
use alloc::vec::Vec;
use oxc_ast::ast::{JSXElementName, JSXMemberExpressionObject, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::Allocator;

#[derive(Default)]
struct Events {
    rows: Vec<(SyntaxEdge, Span)>,
    references: usize,
    invalid_callback: bool,
    invalid_start: Option<u32>,
    empty_component_callback: bool,
}
impl<'a> ReferenceSink<'a> for Events {
    type Checkpoint = (usize, usize);
    fn checkpoint(&self) -> Self::Checkpoint {
        (self.rows.len(), self.references)
    }
    fn reference(&mut self, _: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.references += 1;
        Ok(())
    }
    fn observe_syntax(
        &mut self,
        kind: SyntaxKind<'a>,
        edge: SyntaxEdge,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        if Some(span.start) == self.invalid_start {
            self.invalid_callback = true;
        }
        if matches!(kind, SyntaxKind::Component(_)) && span.start == span.end {
            self.empty_component_callback = true;
        }
        self.rows.push((edge, span));
        Ok(())
    }
    fn rollback(&mut self, (rows, references): Self::Checkpoint) {
        self.rows.truncate(rows);
        self.references = references;
    }
}

#[test]
fn empty_opening_component_leaf_is_refused_before_structural_callback_for_simple_and_member_roots()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in ["<Comp/>;", "<UI.Button/>;"] {
        let mut parsed = Parser::new(&arena, source, SourceType::jsx()).parse();
        let Statement::ExpressionStatement(statement) =
            parsed.program.body.first_mut().ok_or("statement")?
        else {
            return Err("statement kind");
        };
        let Expression::JSXElement(element) = &mut statement.expression else {
            return Err("element");
        };
        let name = match &mut element.opening_element.name {
            JSXElementName::IdentifierReference(name) => name,
            JSXElementName::MemberExpression(member) => match &mut member.object {
                JSXMemberExpressionObject::IdentifierReference(name) => name,
                _ => return Err("member root"),
            },
            _ => return Err("component root"),
        };
        name.span.end = name.span.start;
        let mut sink = Events::default();
        assert_eq!(
            retained(
                ReferenceSource::ProgramJsx(source),
                &statement.expression,
                &mut sink
            )
            .err()
            .ok_or("error")?
            .kind,
            ResolutionErrorKind::InvalidSpan
        );
        assert_eq!(sink.rows, []);
        assert_eq!(sink.references, 0);
        assert!(!sink.empty_component_callback);
    }
    Ok(())
}

#[test]
fn late_utf8_projection_failure_never_reaches_structural_observer_or_keeps_partial_events()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "seed; <my-作者>text</my-作者>;";
    let mut parsed = Parser::new(&arena, source, SourceType::jsx()).parse();
    let Statement::ExpressionStatement(statement) =
        parsed.program.body.get_mut(1).ok_or("statement")?
    else {
        return Err("statement kind");
    };
    let Expression::JSXElement(element) = &mut statement.expression else {
        return Err("element");
    };
    let JSXElementName::Identifier(name) =
        &mut element.closing_element.as_mut().ok_or("closing")?.name
    else {
        return Err("intrinsic");
    };
    name.span.start += 4; // first byte after the first byte of 作者 in the closing leaf
    let invalid = name.span.start;
    let mut sink = Events {
        invalid_start: Some(invalid),
        ..Events::default()
    };
    let first = parsed.program.body.first().ok_or("first")?;
    let Statement::ExpressionStatement(first) = first else {
        return Err("first expression");
    };
    retained(
        ReferenceSource::ProgramJsx(source),
        &first.expression,
        &mut sink,
    )
    .map_err(|_| "seed")?;
    let prior = sink.rows.clone();
    let Statement::ExpressionStatement(last) = parsed.program.body.get(1).ok_or("last")? else {
        return Err("last expression");
    };
    assert_eq!(
        retained(
            ReferenceSource::ProgramJsx(source),
            &last.expression,
            &mut sink
        )
        .err()
        .ok_or("error")?
        .kind,
        ResolutionErrorKind::InvalidSpan
    );
    assert_eq!(sink.rows, prior);
    assert_eq!(sink.references, 1);
    assert!(!source.is_char_boundary(invalid as usize));
    assert!(!sink.invalid_callback);
    Ok(())
}
