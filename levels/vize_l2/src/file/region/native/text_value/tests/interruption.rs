extern crate std;
use super::*;

#[test]
fn caught_fault_before_observe_after_park_mint_or_attach_is_sticky_and_retains_actual_owner() -> Test
{
    for (fault, rows, nodes, attached) in [
        (Fault::BeforeObserve, 0, 0, false),
        (Fault::AfterPark, 1, 0, false),
        (Fault::AfterMint, 1, 1, false),
        (Fault::AfterAttach, 1, 1, true),
    ] {
        let arena = Allocator::default();
        let source = "<template>&amp;lt;</template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("root event")?;
            reset_observations();
            set_fault(fault);
            check(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = walk.root_text_value(child.reborrow());
                }))
                .is_err(),
            )?;
            same(walk.root.facts.native_text_values.len(), rows)?;
            check(walk.root.facts.template_walk.interruption().is_some())?;
            same(observations(), rows)?;
            same(
                walk.root_text_value(child)
                    .err()
                    .ok_or("retry revived")?
                    .kind,
                Kind::Interrupted,
            )?;
            same(observations(), rows)?;
            same(
                walk.complete().err().ok_or("unwind completed")?.kind,
                Kind::Interrupted,
            )?;
        }
        let original = original.finish();
        check(original.view().is_err())?;
        let file = original.file().ok_or("full interrupted owner")?;
        check(!file.is_complete() && file.template_interruption().is_some())?;
        same(file.artifact().node_count(), nodes)?;
        same(file.native_text_values().len(), rows)?;
        if let Some(record) = file.native_text_values().first() {
            let value = record
                .observation()
                .ok_or("whole success remained parked")?;
            same(value.raw_text(), "&amp;lt;")?;
            same(value.source().text(), "&lt;")?;
            same(
                value
                    .source()
                    .decode_map()
                    .ok_or("whole map")?
                    .segments()
                    .len(),
                2,
            )?;
            same(
                record.state(),
                if attached {
                    State::Attached(NodeId::FIRST)
                } else {
                    State::Pending
                },
            )?;
        }
        if nodes == 1 {
            same(
                file.native_text_value_for(NodeId::FIRST, text(file)?)
                    .is_some(),
                attached,
            )?;
        }
    }
    Ok(())
}

#[test]
fn fault_after_full_foreign_failure_park_keeps_original_source_and_failure_kind() -> Test {
    let arena = Allocator::default();
    let source = "<template>&amp;lt;</template>";
    let foreign = selected(&arena, source)?;
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = foreign.children().next().ok_or("foreign Text")?;
        set_fault(Fault::AfterPark);
        check(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = walk.root_text_value(child);
            }))
            .is_err(),
        )?;
        check(walk.complete().is_err())?;
    }
    let original = original.finish();
    let file = original.file().ok_or("full failure owner")?;
    let [record] = file.native_text_values() else {
        return Err("normally owned failure");
    };
    same(record.state(), State::Pending)?;
    let failure = record.failure().ok_or("actual parked failure")?;
    same(
        failure.kind(),
        vize_l1::markup::NativeTextValueError::ForeignComponent,
    )?;
    same(failure.token().ok_or("full Text token")?.text, "&amp;lt;")?;
    check(core::ptr::eq(failure.block().root_source(), source))?;
    check(!file.is_complete() && original.view().is_err())?;
    Ok(())
}

#[test]
fn drop_and_forget_keep_attached_diagnostic_prefix_but_never_grant_completion() -> Test {
    for forgotten in [false, true] {
        let arena = Allocator::default();
        let source = "<template>&amp;lt;</template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            walk.root_text_value(selected.children().next().ok_or("Text")?)
                .map_err(|_| "actual attachment")?;
            if forgotten {
                core::mem::forget(walk);
            }
        }
        let original = original.finish();
        same(
            original
                .view()
                .err()
                .ok_or("unfinished cursor admitted")?
                .kind,
            Kind::Interrupted,
        )?;
        let file = original.file().ok_or("attached diagnostic File")?;
        check(!file.is_complete())?;
        same(file.template_interruption().is_some(), !forgotten)?;
        same(file.native_text_values().len(), 1)?;
        check(
            file.native_text_value_for(NodeId::FIRST, text(file)?)
                .is_some(),
        )?;
    }
    Ok(())
}
