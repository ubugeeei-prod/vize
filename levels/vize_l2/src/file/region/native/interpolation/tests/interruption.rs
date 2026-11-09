extern crate std;
use super::{
    Allocator, Fault, Kind, NativeInterpolationInput, State, Test, check, owner, same, set_fault,
};
use vize_l0::id::NodeId;

#[test]
fn root_drop_or_forget_keeps_admitted_input_and_denies_completion() -> Test {
    for forgotten in [false, true] {
        let arena = Allocator::default();
        let source = "<template>{{ 7 }}</template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("child")?;
            let input = NativeInterpolationInput::from_operand(
                selected
                    .observe_interpolation_expression(child.reborrow())
                    .map_err(|_| "once original syntax")?,
            );
            walk.root_interpolation(child, input)
                .map_err(|_| "whole original event")?;
            if forgotten {
                core::mem::forget(walk);
            }
        }
        let output = original.finish();
        same(
            output.view().err().ok_or("unfinished root admitted")?.kind,
            Kind::Interrupted,
        )?;
        let file = output.file().ok_or("retained File")?;
        check(!file.is_complete())?;
        same(file.artifact().node_count(), 1)?;
        same(file.native_interpolations().len(), 1)?;
        same(
            file.native_interpolation(NodeId::FIRST)
                .ok_or("actual prefix association")?
                .state(),
            State::Admitted(NodeId::FIRST),
        )?;
        same(file.template_interruption().is_some(), !forgotten)?;
    }
    Ok(())
}

#[test]
fn caught_unwind_after_park_or_mint_keeps_owner_and_cannot_revive_root_completion() -> Test {
    for (fault, nodes) in [(Fault::AfterPark, 0), (Fault::AfterMint, 1)] {
        let arena = Allocator::default();
        let source = "<template>{{ /*kept*/ 7 }}</template>";
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("child")?;
            let input = NativeInterpolationInput::from_operand(
                selected
                    .observe_interpolation_expression(child.reborrow())
                    .map_err(|_| "whole original owner")?,
            );
            set_fault(fault);
            let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = walk.root_interpolation(child.reborrow(), input);
            }));
            check(unwind.is_err())?;
            check(walk.root.facts.template_walk.interruption().is_some())?;
            let retry = NativeInterpolationInput::from_operand(
                selected
                    .observe_interpolation_expression(child.reborrow())
                    .map_err(|_| "retry owner")?,
            );
            same(
                walk.root_interpolation(child, retry)
                    .err()
                    .ok_or("interrupted retry admitted")?
                    .kind,
                Kind::Interrupted,
            )?;
            same(
                walk.complete()
                    .err()
                    .ok_or("interrupted root completed")?
                    .kind,
                Kind::Interrupted,
            )?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("actual prefix File")?;
        same(file.artifact().node_count(), nodes)?;
        check(!file.is_complete())?;
        check(file.template_interruption().is_some())?;
        let [pending, retry] = file.native_interpolations() else {
            return Err("both normally retained owners");
        };
        same(pending.state(), State::Pending)?;
        same(retry.state(), State::Refused(Kind::Interrupted))?;
        same(pending.input().operand().syntax().comments().count(), 1)?;
        same(
            pending.input().operand().full_span().slice(source),
            "{{ /*kept*/ 7 }}",
        )?;
        check(file.native_interpolation(NodeId::FIRST).is_none())?;
    }
    Ok(())
}
