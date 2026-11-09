extern crate std;
use super::super::{Fault, set_fault};
use super::{Allocator, Kind, State, Test, check, owner, same};

#[test]
fn nested_unwind_before_observe_after_park_or_mint_keeps_owners_and_denies_retry_completion() -> Test
{
    for (fault, inputs) in [
        (Fault::BeforeObserve, 0),
        (Fault::AfterPark, 1),
        (Fault::AfterMint, 1),
    ] {
        let arena = Allocator::default();
        let source = "<template><p>{{ /*kept*/ 7 }}</p></template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("actual original parent")?;
            set_fault(fault);
            check(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = walk.child(child.reborrow());
                }))
                .is_err(),
            )?;
            check(walk.root.facts.template_walk.interruption().is_some())?;
            same(walk.root.facts.native_interpolations.len(), inputs)?;
            let pointer = walk
                .root
                .facts
                .native_interpolations
                .first()
                .and_then(|record| record.input().operand().syntax().expression())
                .map(core::ptr::from_ref);
            same(
                walk.child(child)
                    .err()
                    .ok_or("interrupted retry admitted")?
                    .kind,
                Kind::Interrupted,
            )?;
            same(walk.root.facts.native_interpolations.len(), inputs)?;
            same(
                walk.root
                    .facts
                    .native_interpolations
                    .first()
                    .and_then(|record| record.input().operand().syntax().expression())
                    .map(core::ptr::from_ref),
                pointer,
            )?;
            same(
                walk.complete()
                    .err()
                    .ok_or("interrupted completion admitted")?
                    .kind,
                Kind::Interrupted,
            )?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let (records, interruption) = if let Some(file) = output.file() {
            check(!file.is_complete())?;
            (file.native_interpolations(), file.template_interruption())
        } else {
            let file = output
                .rejected_file()
                .ok_or("normally retained rejected File")?;
            (file.native_interpolations(), file.template_interruption())
        };
        same(records.len(), inputs)?;
        check(interruption.is_some())?;
        if let Some(record) = records.first() {
            same(record.state(), State::Pending)?;
            same(record.input().operand().syntax().comments().count(), 1)?;
            same(
                record.input().operand().full_span().slice(source),
                "{{ /*kept*/ 7 }}",
            )?;
        }
        check(core::ptr::eq(
            output.selected().component().block().root_source(),
            source,
        ))?;
    }
    Ok(())
}

#[test]
fn preparation_failure_unwind_keeps_the_original_failure_and_cannot_revive_walk() -> Test {
    let arena = Allocator::default();
    let source = "<template><p>{{ 7 }}</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let parent = selected
            .children()
            .next()
            .ok_or("actual parent")?
            .into_element()
            .ok_or("actual Element")?;
        let child = parent.children().next().ok_or("actual recovered child")?;
        set_fault(Fault::AfterFailurePark);
        check(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = super::super::super::observe(selected, child, &mut walk.root);
            }))
            .is_err(),
        )?;
        same(walk.root.facts.native_interpolation_failures.len(), 1)?;
        same(
            walk.child(selected.children().next().ok_or("retry")?)
                .err()
                .ok_or("interrupted retry admitted")?
                .kind,
            Kind::Interrupted,
        )?;
        same(
            walk.complete()
                .err()
                .ok_or("interrupted completion admitted")?
                .kind,
            Kind::Interrupted,
        )?;
    }
    let output = original.finish();
    let file = output
        .file()
        .ok_or("normally retained failed observation")?;
    check(!file.is_complete())?;
    let [failure] = file.native_interpolation_failures() else {
        return Err("complete failure remains parked across unwind");
    };
    same(failure.span().slice(source), "{{ 7 }}")?;
    same(
        failure.failure().kind(),
        vize_l1::markup::NativeInterpolationError::RecoveredComponent,
    )?;
    check(file.template_interruption().is_some())
}

#[test]
fn nested_drop_or_forget_keeps_actual_input_but_never_completes_root_custody() -> Test {
    for forgotten in [false, true] {
        let arena = Allocator::default();
        let source = "<template><p>{{ /*kept*/ 7 }}</p></template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            walk.child(walk.selected().children().next().ok_or("original parent")?)
                .map_err(|_| "actual nested child")?;
            if forgotten {
                core::mem::forget(walk);
            }
        }
        let output = original.finish();
        same(
            output.view().err().ok_or("unfinished root admitted")?.kind,
            Kind::Interrupted,
        )?;
        let file = output.file().ok_or("normally retained prefix")?;
        check(!file.is_complete())?;
        let [record] = file.native_interpolations() else {
            return Err("whole original nested operand");
        };
        let State::Admitted(node) = record.state() else {
            return Err("actual admitted prefix node");
        };
        check(file.native_interpolation(node).is_some())?;
        same(record.input().operand().syntax().comments().count(), 1)?;
        same(file.template_interruption().is_some(), !forgotten)?;
    }
    Ok(())
}
