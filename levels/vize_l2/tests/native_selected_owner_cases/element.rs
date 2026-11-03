use super::{Allocator, Kind, check, equal, kind, owner, selected};
use vize_l2::{
    op::Op,
    walk::{NodeEvent, PageWalk, visit_events},
};

#[test]
fn original_nested_html_bodies_complete_in_the_shared_page_order() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><div>a<span>hé<!--kept--></span><br></div>tail</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "original begin")?;
        let mut children = walk.selected().children();
        equal(
            walk.child(children.next().ok_or("div")?)
                .map_err(|_| "body")?
                .index(),
            0,
        )?;
        equal(
            walk.child(children.next().ok_or("tail")?)
                .map_err(|_| "tail")?
                .index(),
            6,
        )?;
        walk.complete().map_err(|_| "normal root end")?;
    }
    let result = original.finish();
    let view = result.view().map_err(|_| "original complete")?;
    check(core::ptr::eq(view.owner(), &result))?;
    let file = view.file().ok_or("same File")?;
    equal(file.artifact().node_count(), 7)?;
    check(file.is_complete())?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    let [Op::Element(div), Op::Text(tail)] = file.artifact().root().ops.as_slice() else {
        return Err("original root shape");
    };
    equal(div.tag, "div")?;
    check(div.attributes.is_empty() && div.bindings.is_empty())?;
    equal(tail.content, "tail")?;
    let [Op::Text(first), Op::Element(span), Op::Element(br)] = div.children.ops.as_slice() else {
        return Err("actual original siblings");
    };
    equal(first.content, "a")?;
    equal(span.tag, "span")?;
    equal(br.tag, "br")?;
    check(br.children.ops.is_empty())?;
    let [Op::Text(text), Op::Comment(comment)] = span.children.ops.as_slice() else {
        return Err("original inner siblings");
    };
    equal(text.content, "hé")?;
    equal(comment.content, "kept")?;
    let original_div = result
        .selected()
        .children()
        .next()
        .ok_or("selected div")?
        .into_element()
        .ok_or("original Element")?;
    check(core::ptr::eq(original_div.surface().tag(), div.tag))?;
    let mut enters = Vec::new();
    visit_events(
        &mut PageWalk::new(),
        &file.artifact().root().ops,
        &mut |event| {
            if let NodeEvent::Enter {
                id, node, parent, ..
            } = event
            {
                enters.push((id.index(), parent.map(|node| node.index()), node.span()));
            }
        },
    )
    .map_err(|_| "actual traversal")?;
    equal(
        enters.iter().map(|row| (row.0, row.1)).collect::<Vec<_>>(),
        vec![
            (0, None),
            (1, Some(0)),
            (2, Some(0)),
            (3, Some(2)),
            (4, Some(2)),
            (5, Some(0)),
            (6, None),
        ],
    )?;
    for (_, _, span) in enters {
        check(source.get(span.start as usize..span.end as usize).is_some())?;
    }
    Ok(())
}

#[test]
fn original_empty_explicit_self_closing_and_void_html_elements_are_distinct()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template><div/></template>",
        "<template><div></div></template>",
        "<template><input></template>",
    ] {
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            walk.child(walk.selected().children().next().ok_or("child")?)
                .map_err(|_| "zero-header element")?;
            walk.complete().map_err(|_| "actual end")?;
        }
        let result = original.finish();
        let file = result
            .view()
            .map_err(|_| "complete")?
            .file()
            .ok_or("file")?;
        equal(file.artifact().node_count(), 1)?;
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            return Err("single original Element");
        };
        check(element.children.ops.is_empty())?;
        equal(
            &source[element.span.start as usize..element.span.end as usize],
            &source["<template>".len()..source.len() - "</template>".len()],
        )?;
    }
    Ok(())
}

#[test]
fn nested_refusal_is_sticky_and_retains_the_actual_constructed_prefix() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source = "<template><div>ok<span>bad  text</span></div>tail</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let mut children = walk.selected().children();
        equal(
            kind(walk.child(children.next().ok_or("div")?))?,
            Kind::UnsupportedChild,
        )?;
        equal(
            kind(walk.child(children.next().ok_or("tail")?))?,
            Kind::UnsupportedChild,
        )?;
        equal(kind(walk.complete())?, Kind::UnsupportedChild)?;
    }
    equal(kind(original.begin())?, Kind::Interrupted)?;
    let result = original.finish();
    check(result.view().is_err())?;
    let file = result.file().ok_or("partial File")?;
    check(!file.is_complete())?;
    equal(file.artifact().node_count(), 3)?;
    let [Op::Element(div)] = file.artifact().root().ops.as_slice() else {
        return Err("partial original root");
    };
    let [Op::Text(ok), Op::Element(span)] = div.children.ops.as_slice() else {
        return Err("original completed prefix");
    };
    equal(ok.content, "ok")?;
    check(span.children.ops.is_empty())?;
    equal(result.selected().children().len(), 2)?;
    Ok(())
}

#[test]
fn foreign_reordered_and_nested_children_cannot_stand_in_for_a_root_element()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><div><span/></div><br/></template>";
    let foreign_source = source.to_owned();
    let foreign = selected(&arena, &foreign_source)?;
    for mode in 0..3 {
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = match mode {
                0 => foreign.children().next().ok_or("foreign")?,
                1 => walk.selected().children().nth(1).ok_or("reordered")?,
                _ => walk
                    .selected()
                    .children()
                    .next()
                    .ok_or("div")?
                    .into_element()
                    .ok_or("actual div")?
                    .children()
                    .next()
                    .ok_or("span")?,
            };
            equal(kind(walk.child(child))?, Kind::InvalidEvent)?;
            check(walk.complete().is_err())?;
        }
        let result = original.finish();
        check(result.view().is_err())?;
        equal(result.file().ok_or("file")?.artifact().node_count(), 0)?;
    }
    Ok(())
}

#[test]
fn directive_namespace_special_and_operand_routes_remain_unavailable() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for body in [
        "<div :id='x'/>",
        "<div v-pre/>",
        "<div v-if='true'/>",
        "<div>&amp;</div>",
        "<div> </div>",
        "<svg/>",
        "<math/>",
        "<Widget/>",
        "<DIV/>",
        "<template/>",
        "<slot/>",
        "<pre>x</pre>",
        "<textarea>x</textarea>",
        "<title>x</title>",
        "<script>x</script>",
        "<style>x</style>",
    ] {
        let source = format!("<template>{body}</template>");
        let mut original = owner(&arena, &source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = walk.selected().children().next().ok_or("original child")?;
            equal(kind(walk.child(child))?, Kind::UnsupportedChild)?;
            check(walk.complete().is_err())?;
        }
        let result = original.finish();
        check(result.view().is_err())?;
        check(!result.file().ok_or("partial file")?.is_complete())?;
    }
    Ok(())
}

#[test]
fn dropping_after_an_original_element_is_not_root_completion() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let mut original = owner(&arena, "<template><div>x</div></template>")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        walk.child(walk.selected().children().next().ok_or("div")?)
            .map_err(|_| "body")?;
    }
    let result = original.finish();
    check(result.view().is_err())?;
    let file = result.file().ok_or("partial File")?;
    equal(file.artifact().node_count(), 2)?;
    check(!file.is_complete())?;
    check(file.template_interruption().is_some())?;
    Ok(())
}

#[test]
fn forgotten_and_unwound_complete_element_prefixes_cannot_mint_a_view() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    for forgotten in [true, false] {
        let mut original = owner(&arena, "<template><div>x</div></template>")?;
        if forgotten {
            let mut walk = original.begin().map_err(|_| "begin")?;
            walk.child(walk.selected().children().next().ok_or("div")?)
                .map_err(|_| "body")?;
            core::mem::forget(walk);
        } else {
            let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> Result<(), &'static str> {
                    let mut walk = original.begin().map_err(|_| "begin")?;
                    walk.child(walk.selected().children().next().ok_or("div")?)
                        .map_err(|_| "body")?;
                    std::panic::resume_unwind(Box::new(()));
                },
            ));
            check(interrupted.is_err())?;
        }
        equal(kind(original.begin())?, Kind::Interrupted)?;
        let result = original.finish();
        check(result.view().is_err())?;
        let file = result.file().ok_or("same partial File")?;
        equal(file.artifact().node_count(), 2)?;
        check(!file.is_complete())?;
    }
    Ok(())
}
