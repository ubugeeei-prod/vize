extern crate std;
use super::super::interruption::{Fault, set_fault};
use super::*;

#[test]
fn every_value_event_unwind_retains_original_owner_and_stickily_refuses_retry_and_completion()
-> Test {
    for fault in [
        Fault::BeforeObserve,
        Fault::AfterPark,
        Fault::AfterClose,
        Fault::AfterAttach,
    ] {
        let arena = Allocator::default();
        let source = "<template><p title='雪&amp;lt;'/></template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("child")?;
            set_fault(fault);
            check(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = walk.child(child.reborrow());
                }))
                .is_err(),
            )?;
            check(walk.root.facts.template_walk.interruption().is_some())?;
            same(
                walk.child(child).err().ok_or("retry completed")?.kind,
                Kind::Interrupted,
            )?;
            same(
                walk.complete().err().ok_or("interrupted completed")?.kind,
                Kind::Interrupted,
            )?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("normal original prefix File")?;
        check(!file.is_complete())?;
        check(file.template_interruption().is_some())?;
        check(core::ptr::eq(
            output.selected().component().block().root_source(),
            source,
        ))?;
        same(
            file.native_attribute_values().len(),
            usize::from(fault != Fault::BeforeObserve),
        )?;
        if let Some(record) = file.native_attribute_values().first() {
            let value = record.observation().ok_or("entire result parked")?;
            same(value.raw_value(), "雪&amp;lt;")?;
            same(value.source().text(), "雪&lt;")?;
            check(value.source().decode_map().is_some())?;
            same(
                matches!(record.state(), State::Attached { .. }),
                fault == Fault::AfterAttach,
            )?;
        }
    }
    Ok(())
}

#[test]
fn caught_unwind_after_failure_park_preserves_the_entire_failed_original_token() -> Test {
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
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        set_fault(Fault::AfterPark);
        check(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = walk.root.observe_attribute_value(
                    selected,
                    foreign_element
                        .attributes()
                        .next()
                        .ok_or("foreign attribute")?,
                    0,
                );
                Ok::<(), &'static str>(())
            }))
            .is_err(),
        )?;
        same(
            walk.complete().err().ok_or("failed park completed")?.kind,
            Kind::Interrupted,
        )?;
    }
    let output = original.finish();
    let file = output.file().ok_or("retained failure File")?;
    let [record] = file.native_attribute_values() else {
        return Err("entire failed preparation retained");
    };
    same(record.state(), State::Pending)?;
    same(
        record.failure().ok_or("actual failure")?.raw_value(),
        Some("&amp;lt;"),
    )?;
    check(record.observation().is_none())?;
    check(file.template_interruption().is_some())?;
    check(!file.is_complete())?;
    Ok(())
}

#[test]
fn dropping_or_forgetting_whole_root_denies_completion_while_attached_value_remains_owned() -> Test
{
    for forgotten in [false, true] {
        let arena = Allocator::default();
        let source = "<template><p title='&amp;lt;'/></template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            walk.child(selected.children().next().ok_or("child")?)
                .map_err(|_| "original event")?;
            if forgotten {
                core::mem::forget(walk);
            }
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("retained File")?;
        check(!file.is_complete())?;
        same(file.native_attribute_values().len(), 1)?;
        check(
            file.native_attribute_value_for(0, element(file, 0)?, 0)
                .is_some(),
        )?;
        same(file.template_interruption().is_some(), !forgotten)?;
    }
    Ok(())
}
