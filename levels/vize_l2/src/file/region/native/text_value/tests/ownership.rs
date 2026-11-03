use super::*;

#[test]
fn moved_owner_and_grown_normal_storage_keep_the_same_original_text_allocation() -> Test {
    let arena = Allocator::default();
    let source = "<template>&amp;lt;</template>";
    let mut original = owner(&arena, source)?;
    let allocation;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("actual child")?;
        reset_observations();
        walk.root_text_value(child).map_err(|_| "original event")?;
        let record = walk
            .root
            .facts
            .native_text_values
            .first()
            .ok_or("parked row")?;
        allocation = record.text.ok_or("same mint allocation")?;
        walk.root.facts.native_text_values.reserve(128);
        walk.complete().map_err(|_| "complete")?;
    }
    let mut parked = alloc::vec![original.finish()];
    parked.reserve(128);
    let original = parked.pop().ok_or("moved whole owner")?;
    let file = original.file().ok_or("File")?;
    let actual = text(file)?;
    same(core::ptr::NonNull::from(actual), allocation)?;
    same(observations(), 1)?;
    let joined = file
        .native_text_value_for(NodeId::FIRST, actual)
        .ok_or("same File allocation")?;
    let value = joined.observation().ok_or("whole source")?;
    let child = original
        .selected()
        .children()
        .next()
        .ok_or("retained original child")?;
    check(value.admitted_for(original.selected(), child).is_some())?;
    same(value.source().text(), "&lt;")?;
    check(core::ptr::eq(value.source().authored_root(), source))?;
    Ok(())
}

#[test]
fn equal_foreign_files_numeric_nodes_and_populated_neutral_text_never_join() -> Test {
    let arena = Allocator::default();
    let source = "<template>&amp;lt;</template>";
    let original = completed(&arena, source)?;
    let foreign = completed(&arena, source)?;
    let file = original.file().ok_or("original File")?;
    let other = foreign.file().ok_or("foreign File")?;
    let actual = text(file)?;
    let foreign_text = text(other)?;
    same(actual.content, foreign_text.content)?;
    same(actual.span, foreign_text.span)?;
    check(file.native_text_value_for(NodeId::FIRST, actual).is_some())?;
    check(
        file.native_text_value_for(NodeId::FIRST, foreign_text)
            .is_none(),
    )?;
    check(other.native_text_value_for(NodeId::FIRST, actual).is_none())?;
    check(
        file.native_text_value_for(NodeId::from_index(1).ok_or("id")?, actual)
            .is_none(),
    )?;
    let neutral = TextOp {
        content: actual.content,
        span: actual.span,
    };
    check(
        file.native_text_value_for(NodeId::FIRST, &neutral)
            .is_none(),
    )?;
    let value = file.native_text_values()[0]
        .observation()
        .ok_or("whole original source")?;
    check(
        value
            .admitted_for(
                foreign.selected(),
                foreign
                    .selected()
                    .children()
                    .next()
                    .ok_or("foreign child")?,
            )
            .is_none(),
    )?;
    Ok(())
}

#[test]
fn foreign_same_buffer_child_keeps_its_complete_failure_before_root_policy() -> Test {
    let arena = Allocator::default();
    let source = "<template>&amp;lt;</template>";
    let mut original = owner(&arena, source)?;
    let foreign = selected(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = foreign.children().next().ok_or("foreign original Text")?;
        let error = walk
            .root_text_value(child)
            .err()
            .ok_or("foreign admitted")?;
        check(matches!(
            error.kind,
            Kind::TextValuePreparation {
                kind: vize_l1::markup::NativeTextValueError::ForeignComponent,
                ..
            }
        ))?;
    }
    let original = original.finish();
    check(original.view().is_err())?;
    let file = original.file().ok_or("normal failed File")?;
    let [record] = file.native_text_values() else {
        return Err("whole failure row");
    };
    let failure = record.failure().ok_or("complete original failure")?;
    same(failure.ordinal(), 0)?;
    same(
        failure.token().ok_or("actual original token")?.text,
        "&amp;lt;",
    )?;
    same(failure.span(), Some(Span::new(10, 18)))?;
    check(core::ptr::eq(failure.block().root_source(), source))?;
    check(record.observation().is_none())?;
    same(file.artifact().node_count(), 0)?;
    check(!file.is_complete())?;
    Ok(())
}

#[test]
fn duplicate_event_refuses_without_repreparing_or_replacing_the_existing_whole_row() -> Test {
    let arena = Allocator::default();
    let source = "<template>&amp;lt;</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("one child")?;
        reset_observations();
        walk.root_text_value(child.reborrow())
            .map_err(|_| "first event")?;
        same(
            walk.root_text_value(child)
                .err()
                .ok_or("repeat admitted")?
                .kind,
            Kind::InvalidEvent,
        )?;
        same(observations(), 1)?;
        same(walk.root.facts.native_text_values.len(), 1)?;
        same(
            walk.complete().err().ok_or("repeat completed")?.kind,
            Kind::InvalidEvent,
        )?;
    }
    let original = original.finish();
    check(original.view().is_err())?;
    let file = original.file().ok_or("attached diagnostic prefix")?;
    check(!file.is_complete())?;
    same(file.artifact().node_count(), 1)?;
    check(
        file.native_text_value_for(NodeId::FIRST, text(file)?)
            .is_some(),
    )?;
    Ok(())
}
