use super::*;

#[test]
fn duplicate_omission_and_later_receipt_are_sticky_original_cursor_refusals()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template> \t<!--middle--> tail </template>";
    for mode in 0..2 {
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let first = selected
                .prepare_condensed_root_text(selected.children().next().ok_or("first")?)
                .map_err(|_| "original omission")?;
            let tail = selected
                .prepare_condensed_root_text(selected.children().nth(2).ok_or("tail")?)
                .map_err(|_| "original later receipt")?;
            if mode == 0 {
                equal(walk.root_text(&first).map_err(|_| "first omission")?, None)?;
                equal(kind(walk.root_text(&first))?, Kind::InvalidEvent)?;
            } else {
                equal(kind(walk.root_text(&tail))?, Kind::InvalidEvent)?;
            }
            equal(kind(walk.root_text(&tail))?, Kind::InvalidEvent)?;
            equal(kind(walk.complete())?, Kind::InvalidEvent)?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("retained file")?;
        equal(file.artifact().node_count(), 0)?;
        equal(file.template_issues().len(), 1)?;
        check(!file.is_complete() && file.template_interruption().is_some())?;
        equal(output.selected().children().len(), 3)?;
    }
    Ok(())
}

#[test]
fn identical_bytes_actual_reparse_and_copied_source_cannot_supply_root_text()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!--前雪🌸--><template>  original </template>";
    let copy = source.to_owned();
    for foreign_source in [source, copy.as_str()] {
        let foreign = selected(&arena, foreign_source)?;
        let receipt = foreign
            .prepare_condensed_root_text(foreign.children().next().ok_or("foreign")?)
            .map_err(|_| "actual foreign receipt")?;
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            equal(kind(walk.root_text(&receipt))?, Kind::InvalidEvent)?;
            let local = walk.selected();
            let actual = local
                .prepare_condensed_root_text(local.children().next().ok_or("local")?)
                .map_err(|_| "actual local receipt")?;
            equal(kind(walk.root_text(&actual))?, Kind::InvalidEvent)?;
            equal(kind(walk.complete())?, Kind::InvalidEvent)?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("actual file")?;
        equal(file.artifact().node_count(), 0)?;
        equal(file.template_issues().len(), 1)?;
        check(core::ptr::eq(file.artifact().source(), source))?;
    }
    Ok(())
}

#[test]
fn unconsumed_final_omission_never_certifies_a_completed_original_prefix()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!--前雪🌸--><template> first <!--middle--> \n </template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let mut children = selected.children();
        let first = selected
            .prepare_condensed_root_text(children.next().ok_or("first")?)
            .map_err(|_| "prefix receipt")?;
        walk.root_text(&first).map_err(|_| "prefix")?;
        walk.child(children.next().ok_or("middle")?)
            .map_err(|_| "comment")?;
        let tail = selected
            .prepare_condensed_root_text(children.next().ok_or("last")?)
            .map_err(|_| "original omitted final")?;
        equal(tail.content(), None)?;
        equal(kind(walk.complete())?, Kind::IncompleteChildren)?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("actual prefix")?;
    check(!file.is_complete())?;
    let [Op::Text(text), Op::Comment(comment)] = file.artifact().root().ops.as_slice() else {
        return Err("whole completed prefix retained");
    };
    equal(text.content, " first ")?;
    equal(text.span.slice(source), " first ")?;
    equal(comment.content, "middle")?;
    equal(file.artifact().node_count(), 2)?;
    Ok(())
}

#[test]
fn dropping_after_omission_interrupts_even_without_a_minted_node() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template> \n </template>",
        "<template> \n<!--later--></template>",
    ] {
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let receipt = selected
                .prepare_condensed_root_text(selected.children().next().ok_or("first")?)
                .map_err(|_| "original omission")?;
            equal(walk.root_text(&receipt).map_err(|_| "omission")?, None)?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("actual interrupted File")?;
        equal(file.artifact().node_count(), 0)?;
        check(!file.is_complete() && file.template_interruption().is_some())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
    }
    Ok(())
}

#[test]
fn omitted_text_does_not_hide_later_unsupported_body_or_lose_prior_ops() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source = "<template> \n<!--done--><svg/></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let mut children = selected.children();
        let first = selected
            .prepare_condensed_root_text(children.next().ok_or("first")?)
            .map_err(|_| "omission")?;
        equal(
            walk.root_text(&first).map_err(|_| "consume omission")?,
            None,
        )?;
        equal(
            walk.child(children.next().ok_or("comment")?)
                .map_err(|_| "prior comment")?,
            NodeId::FIRST,
        )?;
        equal(
            kind(walk.child(children.next().ok_or("actual unsupported")?))?,
            Kind::UnsupportedChild,
        )?;
        equal(kind(walk.root_text(&first))?, Kind::UnsupportedChild)?;
        equal(kind(walk.complete())?, Kind::UnsupportedChild)?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("actual prefix")?;
    let [Op::Comment(comment)] = file.artifact().root().ops.as_slice() else {
        return Err("exact completed comment prefix");
    };
    equal(comment.content, "done")?;
    equal(file.artifact().node_count(), 1)?;
    equal(file.template_issues().len(), 1)?;
    check(!file.is_complete())?;
    Ok(())
}
