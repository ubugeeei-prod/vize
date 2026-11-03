use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::ReferenceTarget;
use vize_l2::lang::js::{JsxFile, JsxFileProducer};
use vize_l3::jsx::{JsxDecisionKind as Decision, JsxIssueKind as Issue, build_jsx_decisions};

fn lower<'a>(
    arena: &'a Allocator,
    source: &'a str,
    profile: SourceType,
) -> Result<JsxFile<'a>, &'static str> {
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let observation = Parser::new(arena, source, profile).parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, observation, block, 0).map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    producer.finish().map_err(|_| "finish")
}

#[test]
fn jsx_and_tsx_scalar_body_reads_are_bound_to_the_genuine_original_function_scope()
-> Result<(), &'static str> {
    let source = "function render(message) { return <div>{message}{0x10}{'\\u0041'}{true}{false}{null}</div>; }";
    for profile in [SourceType::jsx(), SourceType::tsx().with_module(true)] {
        let arena = Allocator::default();
        let owner = lower(&arena, source, profile)?;
        let original = owner
            .observation()
            .admitted()
            .ok_or("admission")?
            .program()
            .body
            .as_ptr();
        let mut moved = vec![build_jsx_decisions(owner).map_err(|_| "scalar projection")?];
        moved.reserve(256);
        let analysis = moved.pop().ok_or("analysis")?;
        if analysis
            .owner()
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr()
            != original
        {
            return Err("original moved Program");
        }
        let read = analysis
            .decisions()
            .find(|row| matches!(row.kind(), Decision::Read { .. }))
            .ok_or("actual read")?;
        let Decision::Read { name, binding } = read.kind() else {
            return Err("read kind");
        };
        let [reference] = read.node().references().ok_or("references")? else {
            return Err("one original read");
        };
        if name != "message"
            || reference.target != ReferenceTarget::Resolved(binding)
            || reference.scope == analysis.owner().file().units().first().ok_or("unit")?.scope
            || !core::ptr::eq(read.node().owner(), analysis.owner())
        {
            return Err("same owning function binding");
        }
        let values = analysis
            .decisions()
            .filter_map(|row| match row.kind() {
                value @ (Decision::Number(_)
                | Decision::String(_)
                | Decision::Boolean(_)
                | Decision::Null) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        if values
            != [
                Decision::Number(16.0_f64.to_bits()),
                Decision::String("A"),
                Decision::Boolean(true),
                Decision::Boolean(false),
                Decision::Null,
            ]
            || analysis
                .decisions()
                .filter(|row| row.kind() == Decision::ExpressionContainer)
                .count()
                != 6
        {
            return Err("original scalar decision values");
        }
    }
    Ok(())
}

#[test]
fn scalar_support_keeps_compound_body_and_attribute_expressions_refused() -> Result<(), &'static str>
{
    for source in [
        "function render(x) { return <div>{x + 1}</div>; }",
        "function render(x) { return <div>{-1}</div>; }",
        "function render(x) { return <div id={x + 1}/>; }",
    ] {
        let arena = Allocator::default();
        let owner = lower(&arena, source, SourceType::jsx())?;
        let original = owner
            .observation()
            .admitted()
            .ok_or("admission")?
            .program()
            .body
            .as_ptr();
        let rejected = build_jsx_decisions(owner)
            .err()
            .ok_or("whole-view refusal")?;
        if rejected
            .owner()
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr()
            != original
            || !rejected.owner().file().is_complete()
            || !rejected
                .issues()
                .iter()
                .any(|issue| matches!(issue.kind, Issue::Expression | Issue::Attribute))
        {
            return Err("typed refusal retains genuine lower custody");
        }
    }
    Ok(())
}
