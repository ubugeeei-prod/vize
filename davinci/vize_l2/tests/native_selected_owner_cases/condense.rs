use super::*;
use vize_l0::id::NodeId;
use vize_l1::SurfaceChild;
use vize_l2::op::Op;

mod refusal;

#[test]
fn exact_root_cursor_omissions_and_unicode_values_make_complete_original_files()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let cases: &[(&str, &[(&str, &str)])] = &[
        ("<template> \t\r\n\x0c </template>", &[]),
        (
            "<template> \ta\r\n  b\x0c </template>",
            &[("text", " a b ")],
        ),
        (
            "<!--前雪🌸--><template> \t前\u{a0}\u{85}\u{2028}\u{2029}尾 \r\n </template>",
            &[("text", " 前\u{a0}\u{85}\u{2028}\u{2029}尾 ")],
        ),
        (
            "<template>\u{a0}\u{85}\u{1680}\u{2000}\u{3000}\u{feff}</template>",
            &[("text", "\u{a0}\u{85}\u{1680}\u{2000}\u{3000}\u{feff}")],
        ),
        (
            "<template><!--first--> \t <!--last--></template>",
            &[("comment", "first"), ("comment", "last")],
        ),
        (
            "<template><!--first--> \t <i/></template>",
            &[("comment", "first"), ("element", "i")],
        ),
        (
            "<template><i/> \x0c <!--last--></template>",
            &[("element", "i"), ("comment", "last")],
        ),
        (
            "<template><i/> \r\n <b/></template>",
            &[("element", "i"), ("element", "b")],
        ),
        (
            "<template><i/>\x0c<b/></template>",
            &[("element", "i"), ("text", " "), ("element", "b")],
        ),
        (
            "<template><i/>  <b/></template>",
            &[("element", "i"), ("text", " "), ("element", "b")],
        ),
        (
            "<template> \n<!--middle--> \t </template>",
            &[("comment", "middle")],
        ),
    ];
    for &(source, expected) in cases {
        let mut original = owner(&arena, source)?;
        let source_children = original.selected().children().len();
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let mut emitted = 0;
            let mut consumed = 0;
            for child in selected.children() {
                let actual = if matches!(child.surface(), SurfaceChild::Text(_)) {
                    let receipt = selected
                        .prepare_condensed_root_text(child)
                        .map_err(|_| "actual receipt")?;
                    let actual = walk.root_text(&receipt).map_err(|_| "actual root text")?;
                    equal(actual.is_some(), receipt.content().is_some())?;
                    equal(receipt.span().slice(source), receipt.raw_text())?;
                    actual
                } else {
                    Some(walk.child(child).map_err(|_| "actual nontext")?)
                };
                if let Some(node) = actual {
                    equal(node, NodeId::from_index(emitted).ok_or("node ordinal")?)?;
                    emitted += 1;
                }
                consumed += 1;
            }
            equal(consumed, source_children)?;
            equal(emitted as usize, expected.len())?;
            walk.complete().map_err(|_| "complete actual cursor")?;
        }
        let output = core::hint::black_box(original.finish());
        let file = output
            .view()
            .map_err(|_| "completion view")?
            .file()
            .ok_or("file")?;
        check(file.is_complete())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
        equal(file.artifact().node_count() as usize, expected.len())?;
        equal(output.selected().children().len(), source_children)?;
        let ops = file.artifact().root().ops.as_slice();
        equal(ops.len(), expected.len())?;
        for (op, &(kind, content)) in ops.iter().zip(expected) {
            let (actual_kind, actual_content, span) = match op {
                Op::Text(text) => ("text", text.content, text.span),
                Op::Comment(comment) => ("comment", comment.content, comment.span),
                Op::Element(element) => ("element", element.tag, element.span),
                _ => return Err("bounded actual root operation"),
            };
            equal((actual_kind, actual_content), (kind, content))?;
            check(!span.slice(source).is_empty())?;
        }
        equal(file.units().len(), 0)?;
    }
    Ok(())
}

#[test]
fn pending_receipts_survive_owner_moves_and_join_only_the_original_cursor()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!--前雪🌸--><template> \tfirst<!--middle--> \r\n </template>";
    let selected = selected(&arena, source)?;
    let mut children = selected.children();
    let first = selected
        .prepare_condensed_root_text(children.next().ok_or("first")?)
        .map_err(|_| "first receipt")?;
    children.next().ok_or("middle")?;
    let last = selected
        .prepare_condensed_root_text(children.next().ok_or("last")?)
        .map_err(|_| "last receipt")?;
    let mut pending = vec![first, last];
    pending.reserve(64);
    let mut original = NativeTemplateOwner::new(core::hint::black_box(selected))
        .map_err(|_| "moved original owner")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        equal(
            walk.root_text(&pending[0]).map_err(|_| "moved first")?,
            Some(NodeId::FIRST),
        )?;
        let comment = walk.selected().children().nth(1).ok_or("actual comment")?;
        equal(
            walk.child(comment).map_err(|_| "comment")?,
            NodeId::from_index(1).ok_or("second")?,
        )?;
        equal(
            walk.root_text(&pending[1])
                .map_err(|_| "moved last omission")?,
            None,
        )?;
        walk.complete().map_err(|_| "complete moved owner")?;
    }
    let output = core::hint::black_box(original.finish());
    let file = output
        .view()
        .map_err(|_| "original view")?
        .file()
        .ok_or("file")?;
    equal(file.artifact().node_count(), 2)?;
    let [Op::Text(text), Op::Comment(comment)] = file.artifact().root().ops.as_slice() else {
        return Err("exact moved original operations");
    };
    equal(text.content, " first")?;
    equal(text.span.slice(source), " \tfirst")?;
    equal(comment.content, "middle")?;
    Ok(())
}
