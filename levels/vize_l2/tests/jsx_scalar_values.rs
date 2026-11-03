use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::lang::js::{JsxFile, JsxFileProducer};
use vize_l2::resolution::{SyntaxKind as Syntax, Usage};

fn lower<'a>(arena: &'a Allocator, source: &'a str) -> Result<JsxFile<'a>, &'static str> {
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let observation = Parser::new(arena, source, SourceType::jsx()).parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, observation, block, 0).map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    producer.finish().map_err(|_| "finish")
}

#[test]
fn original_scalar_values_keep_decoded_values_authored_source_and_exact_reference_rows()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "function render(message) { return <div>{message}{0x10}{'\\u0041'}{false}{null}</div>; }";
    let owner = lower(&arena, source)?;
    let values = owner
        .nodes()
        .filter(|node| node.kind().is_some_and(|kind| kind.is_expression()))
        .filter(|node| node.parent().and_then(|parent| parent.kind()) == Some(Syntax::Container))
        .collect::<Vec<_>>();
    let expected = [
        (Syntax::IdentifierExpression("message"), "message"),
        (Syntax::NumberExpression(16.0_f64.to_bits()), "0x10"),
        (Syntax::StringExpression("A"), "'\\u0041'"),
        (Syntax::BooleanExpression(false), "false"),
        (Syntax::NullExpression, "null"),
    ];
    if values.len() != expected.len() {
        return Err("scalar count");
    }
    for (node, (kind, text)) in values.iter().zip(expected) {
        if node.kind() != Some(kind) || node.source() != Some(text) {
            return Err("original scalar payload/source");
        }
        if node.children().next().is_some() {
            return Err("scalar subtree");
        }
    }
    let identifier = values.first().ok_or("identifier")?;
    let [reference] = identifier.references().ok_or("references")? else {
        return Err("actual scalar reference");
    };
    if reference.name.as_str() != "message"
        || reference.usage != Usage::Read
        || Some(reference.span) != identifier.span()
    {
        return Err("scalar reference identity");
    }
    if values
        .iter()
        .skip(1)
        .any(|node| node.references().is_none_or(|rows| !rows.is_empty()))
    {
        return Err("invented literal reference");
    }
    Ok(())
}

#[test]
fn moved_original_owner_keeps_unicode_and_escaped_identifier_payloads() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source = "/*kept*/ function render(作者) { return <div>{\\u4f5c者}{'前'}</div>; }";
    let owner = lower(&arena, source)?;
    let original = owner
        .observation()
        .admitted()
        .ok_or("admission")?
        .program()
        .body
        .as_ptr();
    let mut moved = vec![owner];
    moved.reserve(256);
    let owner = moved.pop().ok_or("owner")?;
    if owner
        .observation()
        .admitted()
        .ok_or("original")?
        .program()
        .body
        .as_ptr()
        != original
        || owner.observation().comments().len() != 1
    {
        return Err("original custody");
    }
    let node = owner
        .nodes()
        .find(|node| node.kind() == Some(Syntax::IdentifierExpression("作者")))
        .ok_or("original escaped identifier")?;
    if node.source() != Some("\\u4f5c者") {
        return Err("escaped authored source");
    }
    Ok(())
}
