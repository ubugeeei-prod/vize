#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions bind original JSX names to checked spans and transactional events"
)]

use super::{LawResult, Pending, Required, expression};
use crate::resolution::{ResolutionErrorKind, source::ProgramReferenceSource};
use alloc::vec::Vec;
use oxc_ast::ast::{Expression, JSXElementName, JSXMemberExpressionObject, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String};

#[test]
fn original_member_root_is_one_borrowed_reference_and_properties_are_static_source_leaves()
-> LawResult {
    let arena = Allocator::default();
    let file = "<ui.組.Button>{value}</ui.組.Button>;";
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
    let root = expression(source.program(), 0)?;
    let Expression::JSXElement(element) = root else {
        return Err("JSX element");
    };
    let JSXElementName::MemberExpression(member) = &element.opening_element.name else {
        return Err("member");
    };
    let JSXMemberExpressionObject::MemberExpression(object) = &member.object else {
        return Err("nested member");
    };
    let JSXMemberExpressionObject::IdentifierReference(identifier) = &object.object else {
        return Err("actual root reference");
    };
    let mut sink = Pending::default();
    source.expression(root, &mut sink).required()?;
    let row = sink.rows.first().required()?;
    assert!(core::ptr::eq(row.name(), identifier.name.as_str()));
    assert_eq!(row.span(), Span::new(1, 3));
    assert_eq!(
        sink.rows.iter().map(|row| row.name()).collect::<Vec<_>>(),
        ["ui", "value"]
    );
    assert_eq!(sink.calls, []);
    assert_eq!(sink.invocations, []);
    Ok(())
}

#[test]
fn invalid_original_tag_leaves_restore_opening_and_late_closing_events() -> LawResult {
    for (intrinsic, closing) in [(true, false), (false, false), (true, true), (false, true)] {
        let arena = Allocator::default();
        let file = if intrinsic {
            "seed(); <my-作者>{run(value)}</my-作者>;"
        } else {
            "seed(); <UI.作者>{run(value)}</UI.作者>;"
        };
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
        let name = if closing {
            &mut element.closing_element.as_mut().required()?.name
        } else {
            &mut element.opening_element.name
        };
        let span = match name {
            JSXElementName::Identifier(name) => &mut name.span,
            JSXElementName::MemberExpression(member) => &mut member.property.span,
            _ => return Err("actual static JSX leaf"),
        };
        span.start += if intrinsic { 4 } else { 1 };
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
    }
    Ok(())
}

#[test]
fn late_namespace_and_real_member_root_sink_refusals_restore_prior_invocation_rows() -> LawResult {
    let arena = Allocator::default();
    let file = "seed(); <div><UI.Button>{new Maker(value)}</UI.Button><ns:tag/></div>; <div>{run(value)}<Other.Button/></div>;";
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
fn original_static_member_chain_uses_the_existing_depth_budget_without_partial_facts() -> LawResult
{
    let arena = Allocator::default();
    let mut file = String::from("<UI");
    for _ in 0..80 {
        file.push_str(".Part");
    }
    file.push_str("/>;");
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
