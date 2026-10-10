use super::*;

#[test]
fn late_header_and_body_refusals_keep_whole_preparations_without_fictional_attachment() -> Test {
    for source in [
        "<template><div title='&amp;lt;' title='second'>x</div></template>",
        "<template><div title='&amp;lt;' :id='1'>x</div></template>",
        "<template><div title='&amp;lt;'> &amp; </div></template>",
        "<template><div title='&#13;'>x</div></template>",
        "<template><div title='\0'>x</div></template>",
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let error = walk
                .child(selected.children().next().ok_or("actual child")?)
                .err()
                .ok_or("unsupported source admitted")?;
            same(error.kind, Kind::UnsupportedChild)?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("retained actual File")?;
        check(!file.is_complete())?;
        check(!file.native_attribute_values().is_empty())?;
        for record in file.native_attribute_values() {
            check(!matches!(record.state(), State::Attached { .. }))?;
            check(record.observation().is_some())?;
            check(record.failure().is_none())?;
            check(core::ptr::eq(
                record
                    .observation()
                    .ok_or("whole source")?
                    .source()
                    .authored_root(),
                source,
            ))?;
        }
    }
    Ok(())
}

#[test]
fn former_class_refusal_retains_its_original_preparation_and_actual_attachment() -> Test {
    let arena = Allocator::default();
    let source = "<template><div class='&amp;lt;'>x</div></template>";
    let output = lower(&arena, source)?;
    let file = output
        .view()
        .map_err(|_| "completed class")?
        .file()
        .ok_or("File")?;
    check(file.is_complete())?;
    let [record] = file.native_attribute_values() else {
        return Err("one class owner");
    };
    let value = record.observation().ok_or("original class")?;
    same(value.raw_value(), "&amp;lt;")?;
    same(value.source().text(), "&lt;")?;
    check(core::ptr::eq(value.source().authored_root(), source))?;
    let actual = element(file, 0)?;
    check(file.native_attribute_value_for(0, actual, 0).is_some())?;
    check(matches!(record.state(), State::Attached { slot: 0, .. }))?;
    Ok(())
}

#[test]
fn complete_foreign_preparation_failure_is_parked_before_original_join_and_policy() -> Test {
    let arena = Allocator::default();
    let source = "<template><p title='&amp;lt;'/></template>";
    let mut original = owner(&arena, source)?;
    let foreign = selected(&arena, source)?;
    let foreign_element = foreign
        .children()
        .next()
        .ok_or("foreign child")?
        .into_element()
        .ok_or("foreign Element")?;
    let original_attribute = foreign_element
        .attributes()
        .next()
        .ok_or("foreign attribute")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let error = walk
            .root
            .observe_attribute_value(selected, original_attribute, 0)
            .err()
            .ok_or("foreign preparation admitted")?;
        check(matches!(error, Kind::AttributeValue { .. }))?;
        let [record] = walk.root.facts.native_attribute_values.as_slice() else {
            return Err("whole result parked once");
        };
        let failure = record.failure().ok_or("full original failure")?;
        same(failure.raw_value(), Some("&amp;lt;"))?;
        same(failure.raw_name(), "title")?;
        same(failure.ordinal(), 0)?;
        check(core::ptr::eq(
            failure.block().source(),
            foreign.component().block().source(),
        ))?;
        same(
            failure
                .value_span()
                .ok_or("original value span")?
                .slice(source),
            "&amp;lt;",
        )?;
        same(record.state(), State::Refused(error))?;
        check(record.observation().is_none())?;
        // This private diagnostic experiment grants no original child/cursor completion.
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("normal parked failure")?;
    same(file.native_attribute_values().len(), 1)?;
    same(file.artifact().node_count(), 0)?;
    check(!file.is_complete())?;
    Ok(())
}

#[test]
fn duplicate_original_child_never_reprepares_or_reassociates_the_existing_slot() -> Test {
    let arena = Allocator::default();
    let source = "<template><p title='&amp;lt;'/></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("child")?;
        walk.child(child.reborrow())
            .map_err(|_| "original first event")?;
        same(
            walk.child(child)
                .err()
                .ok_or("duplicate child admitted")?
                .kind,
            Kind::InvalidEvent,
        )?;
        same(walk.root.facts.native_attribute_values.len(), 1)?;
        same(
            walk.complete().err().ok_or("refused completed")?.kind,
            Kind::InvalidEvent,
        )?;
    }
    let output = original.finish();
    let file = output.file().ok_or("prefix File")?;
    check(!file.is_complete())?;
    same(file.native_attribute_values().len(), 1)?;
    check(
        file.native_attribute_value_for(0, element(file, 0)?, 0)
            .is_some(),
    )?;
    Ok(())
}

#[test]
fn original_literal_for_collection_keeps_value_owner_but_refuses_unchanged_collection_shape() -> Test
{
    let arena = Allocator::default();
    let source = "<template><div title='&amp;lt;' v-for='item in 2' @click='$event.count++'>x</div></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let issue = walk
            .child(selected.children().next().ok_or("original child")?)
            .err()
            .ok_or("literal collection admitted")?;
        check(matches!(
            issue.kind,
            Kind::For {
                kind: crate::file::FileIssueKind::UnsupportedSyntax,
                ..
            }
        ))?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("retained File")?;
    let [record] = file.native_attribute_values() else {
        return Err("whole value parked");
    };
    same(record.state(), State::Pending)?;
    same(
        record.observation().ok_or("whole value")?.source().text(),
        "&lt;",
    )?;
    let [crate::file::RejectedFileFor::Syntax(refusal)] = file.rejected_for_heads() else {
        return Err("genuine original collection syntax refusal");
    };
    same(
        refusal.kind,
        vize_l1::embed::syntax::NativeForRefusal::CollectionShape,
    )?;
    same(refusal.operand().raw_value(), "item in 2")?;
    same(file.artifact().node_count(), 0)?;
    check(!file.is_complete())
}
