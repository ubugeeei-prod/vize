#![expect(
    clippy::panic_in_result_fn,
    reason = "native ownership laws use exact assertions"
)]
use super::{
    Allocator, JsxFileProducer, Law, Namespace, Parser, SourceRoot, SourceType, Span, Tree, span,
    tree,
};
use crate::op::Op;
use alloc::{vec, vec::Vec};

#[test]
fn embedded_original_block_keeps_absolute_coordinates_and_identity_after_moves() -> Law {
    let arena = Allocator::default();
    let source = "前😀\r\n<script>const view = <p title=\"😀\">ok</p>;</script>";
    let begin = source.find("const view").ok_or("block begin")?;
    let end = source.find("</script>").ok_or("block end")?;
    let script = source.get(begin..end).ok_or("block slice")?;
    let block = SourceRoot::new(source)
        .map_err(|_| "root")?
        .block(script, begin as u32)
        .map_err(|_| "frame")?;
    let mut producer = JsxFileProducer::new(
        &arena,
        Parser::new(&arena, script, SourceType::jsx()).parse_observed(),
        block,
        7,
    )
    .map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    let body = producer
        .observation()
        .admitted()
        .ok_or("admission")?
        .program()
        .body
        .as_ptr();
    let native = producer.finish_markup().map_err(|_| "native")?;
    let expected = vec![Tree::Element(
        "p",
        Namespace::Html,
        vec![("title", Some("😀"), span(source, "title=\"😀\"")?)],
        span(source, "<p title=\"😀\">ok</p>")?,
        vec![Tree::Text("ok", span(source, "ok")?)],
    )];
    let mut moved = vec![native];
    for _ in 0..12 {
        moved.push(producer_for_move(&arena, source, block, script)?);
    }
    let native = moved.first().ok_or("moved owner")?;
    assert_eq!(tree(&native.file().artifact().root().ops)?, expected);
    let Op::Element(element) = native
        .file()
        .artifact()
        .root()
        .ops
        .first()
        .ok_or("root op")?
    else {
        return Err("root element");
    };
    let position = |offset: u32| -> Result<(usize, usize), &'static str> {
        let prefix = source
            .get(..offset as usize)
            .ok_or("UTF16 source position")?;
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
        let column = prefix
            .rsplit('\n')
            .next()
            .ok_or("line prefix")?
            .encode_utf16()
            .count();
        Ok((line, column))
    };
    assert_eq!(
        (position(element.span.start)?, position(element.span.end)?),
        ((1, 21), (1, 41))
    );

    assert_eq!(
        native
            .file()
            .units()
            .iter()
            .map(|unit| (unit.id.index(), unit.span))
            .collect::<Vec<_>>(),
        vec![(7, Span::new(begin as u32, end as u32))]
    );
    assert_eq!(
        native
            .owner()
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr(),
        body
    );
    assert!(core::ptr::eq(native.file().artifact().source(), source));
    let original = native.owner().node(0).ok_or("first record")?;
    let foreign = moved
        .last()
        .ok_or("foreign owner")?
        .owner()
        .node(0)
        .ok_or("foreign record")?;
    assert!(!original.same_owner(foreign));
    Ok(())
}

fn producer_for_move<'a>(
    arena: &'a Allocator,
    source: &'a str,
    block: vize_l0::SourceBlock<'a>,
    script: &'a str,
) -> Result<super::super::JsxMarkupFile<'a>, &'static str> {
    let mut producer = JsxFileProducer::new(
        arena,
        Parser::new(arena, script, SourceType::jsx()).parse_observed(),
        block,
        7,
    )
    .map_err(|_| "move producer")?;
    producer.walk().map_err(|_| "move walk")?;
    let native = producer.finish_markup().map_err(|_| "move native")?;
    assert!(core::ptr::eq(native.file().artifact().source(), source));
    Ok(native)
}
