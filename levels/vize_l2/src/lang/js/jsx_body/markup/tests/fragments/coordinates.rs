#![expect(
    clippy::panic_in_result_fn,
    reason = "exact source ownership witnesses"
)]

use super::{Allocator, Law};
use crate::lang::js::{JsxFile, JsxFileProducer, JsxMarkupFile};
use alloc::{vec, vec::Vec};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{SourceBlock, SourceRoot, Span, cstr};

fn native<'a>(
    arena: &'a Allocator,
    block: SourceBlock<'a>,
    script: &'a str,
    typescript: bool,
) -> Result<JsxMarkupFile<'a>, &'static str> {
    let original = Parser::new(
        arena,
        script,
        if typescript {
            SourceType::tsx().with_module(true)
        } else {
            SourceType::jsx()
        },
    )
    .parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, original, block, 7).map_err(|_| "block producer")?;
    producer.walk().map_err(|_| "block walk")?;
    producer.finish_markup().map_err(|_| "block native")
}

#[test]
fn original_fragment_boundaries_keep_absolute_bytes_utf16_and_owner_after_moves() -> Law {
    assert_eq!(
        core::mem::size_of::<JsxMarkupFile<'_>>(),
        core::mem::size_of::<JsxFile<'_>>(),
        "no additional product storage"
    );
    for typescript in [false, true] {
        for newline in ["\n", "\r\n"] {
            let arena = Allocator::default();
            let source =
                cstr!("前😀{newline}<script>const view = <><p>😀</p><><br/></></>;</script>");
            let begin = source.find("const view").ok_or("block begin")?;
            let end = source.find("</script>").ok_or("block end")?;
            let script = source.get(begin..end).ok_or("script")?;
            let block = SourceRoot::new(&source)
                .map_err(|_| "source root")?
                .block(script, begin as u32)
                .map_err(|_| "source block")?;
            let first = native(&arena, block, script, typescript)?;
            let body = first
                .owner()
                .observation()
                .admitted()
                .ok_or("original body")?
                .program()
                .body
                .as_ptr();
            let mut moved = vec![first];
            for _ in 0..12 {
                moved.push(native(&arena, block, script, typescript)?);
            }
            let first = moved.first().ok_or("moved native")?;
            let foreign = moved.last().ok_or("foreign native")?;
            assert_eq!(
                first
                    .owner()
                    .observation()
                    .admitted()
                    .ok_or("moved body")?
                    .program()
                    .body
                    .as_ptr(),
                body
            );
            assert!(core::ptr::eq(
                first.file().artifact().source(),
                source.as_str()
            ));
            assert_eq!(
                first
                    .file()
                    .units()
                    .iter()
                    .map(|u| (u.id.index(), u.span))
                    .collect::<Vec<_>>(),
                vec![(7, Span::new(begin as u32, end as u32))]
            );
            let boundaries: Vec<_> = first.fragment_boundaries().collect();
            let foreign_node = foreign
                .fragment_boundaries()
                .next()
                .ok_or("foreign boundary")?;
            let expected = [
                ("<>", 21, 23),
                ("<>", 32, 34),
                ("</>", 39, 42),
                ("</>", 42, 45),
            ];
            assert_eq!(boundaries.len(), expected.len());
            for (node, (text, start, end)) in boundaries.iter().zip(expected) {
                assert_eq!(node.source(), Some(text));
                assert!(!node.same_owner(foreign_node));
                let span = node.span().ok_or("boundary range")?;
                let position = |offset: u32| -> Result<(usize, usize), &'static str> {
                    let prefix = source.get(..offset as usize).ok_or("UTF16 boundary")?;
                    Ok((
                        prefix.bytes().filter(|b| *b == b'\n').count(),
                        prefix
                            .rsplit('\n')
                            .next()
                            .ok_or("line prefix")?
                            .encode_utf16()
                            .count(),
                    ))
                };
                assert_eq!(
                    (position(span.start)?, position(span.end)?),
                    ((1, start), (1, end))
                );
            }
            assert_eq!(
                super::realize::boundaries(first)?,
                super::realize::boundaries(foreign)?
            );
            assert_eq!(
                super::realize::mounted(first)?,
                super::realize::mounted(foreign)?
            );
        }
    }
    Ok(())
}
