use super::super::{NativeInterpolationInput, selected};
use super::{Allocator, Kind, State, Test, check, owner, same};
use vize_l1::markup::NativeInterpolationError;

#[test]
fn the_static_root_child_entry_still_requires_the_separate_whole_interpolation_input() -> Test {
    let arena = Allocator::default();
    let source = "<template>{{ 7 }}</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = walk
            .selected()
            .children()
            .next()
            .ok_or("actual root child")?;
        same(
            walk.child(child)
                .err()
                .ok_or("root shortcut admitted")?
                .kind,
            Kind::UnsupportedChild,
        )?;
        check(walk.complete().is_err())?;
    }
    let output = original.finish();
    let file = output.file().ok_or("normally retained root refusal")?;
    same(file.artifact().node_count(), 0)?;
    check(file.native_interpolations().is_empty())?;
    check(file.native_interpolation_failures().is_empty())
}

#[test]
fn late_nested_holes_and_missing_bindings_keep_prefix_and_whole_failed_observations() -> Test {
    for content in ["/*kept*/ value +", "/*kept*/ missing", ""] {
        let arena = Allocator::default();
        let source = alloc::format!("<template><p>{{{{ 7 }}}}{{{{{content}}}}}</p></template>");
        let mut original = owner(&arena, source.as_str())?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = walk.selected().children().next().ok_or("original parent")?;
            check(walk.child(child).is_err())?;
            check(walk.complete().is_err())?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("normally retained original prefix")?;
        check(!file.is_complete())?;
        same(file.artifact().node_count(), 2)?;
        let [prefix, failed] = file.native_interpolations() else {
            return Err("both whole original operands");
        };
        let State::Admitted(prefix_node) = prefix.state() else {
            return Err("actual admitted prefix");
        };
        check(file.native_interpolation(prefix_node).is_some())?;
        check(matches!(failed.state(), State::Refused(_)))?;
        same(failed.input().operand().raw_content(), content)?;
        let syntax = failed.input().operand().syntax();
        same(syntax.comments().count(), usize::from(!content.is_empty()))?;
        if content.contains("missing") {
            check(syntax.hole().is_none())?;
            check(syntax.expression().is_some())?;
            check(file.template_issues().iter().any(|issue| {
                issue.kind == crate::file::FileIssueKind::UnresolvedReference
                    && issue.span.slice(source.as_str()) == "missing"
            }))?;
        } else {
            check(syntax.hole().is_some())?;
            check(syntax.diagnostics().count() > 0)?;
        }
        check(file.native_interpolation_failures().is_empty())?;
    }
    Ok(())
}

#[test]
fn foreign_nested_child_and_foreign_whole_input_never_mint_or_complete_the_actual_file() -> Test {
    for foreign_input in [false, true] {
        let arena = Allocator::default();
        let source = "<template><p>{{ /*kept*/ 7 }}</p></template>";
        let mut original = owner(&arena, source)?;
        let other = selected(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let foreign_parent = other
                .children()
                .next()
                .ok_or("foreign parent")?
                .into_element()
                .ok_or("foreign Element")?;
            let foreign_child = foreign_parent.children().next().ok_or("foreign child")?;
            let kind = if foreign_input {
                let input = NativeInterpolationInput::from_operand(
                    other
                        .observe_interpolation_expression(foreign_child)
                        .map_err(|_| "whole foreign original")?,
                );
                let parent = selected
                    .children()
                    .next()
                    .ok_or("own parent")?
                    .into_element()
                    .ok_or("own Element")?;
                let child = parent.children().next().ok_or("own original child")?;
                walk.root
                    .with_walk(selected.component().block().span(), |region| {
                        super::super::super::construct(selected, child, input, region, Ok(()))
                    })
                    .err()
                    .ok_or("foreign input joined")?
            } else {
                super::super::super::observe(selected, foreign_child, &mut walk.root)
                    .err()
                    .ok_or("foreign child observed")?
            };
            check(walk.reject::<()>(kind).is_err())?;
            check(walk.complete().is_err())?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("normally owned refused File")?;
        same(file.artifact().node_count(), 0)?;
        if foreign_input {
            let [record] = file.native_interpolations() else {
                return Err("normally retained whole foreign input");
            };
            same(
                record.state(),
                State::Refused(Kind::Interpolation(
                    crate::lang::js::NativeInterpolationInputError::UnadmittedOriginal,
                )),
            )?;
            same(record.input().operand().syntax().comments().count(), 1)?;
            check(file.native_interpolation_failures().is_empty())?;
        } else {
            check(file.native_interpolations().is_empty())?;
            let [record] = file.native_interpolation_failures() else {
                return Err("normally retained complete L1 failure");
            };
            same(
                record.failure().kind(),
                NativeInterpolationError::ForeignComponent,
            )?;
        }
    }
    Ok(())
}

#[test]
fn actual_recovered_parent_preparation_failure_retains_exact_construct_and_typed_span() -> Test {
    let arena = Allocator::default();
    let source = "<template><p>{{ 7 }}</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let parent = selected
            .children()
            .next()
            .ok_or("original recovered parent")?
            .into_element()
            .ok_or("original Element")?;
        let child = parent.children().next().ok_or("original interpolation")?;
        // Private refusal seam only: the public Element preflight still refuses
        // this incomplete frame before its children and grants no completion.
        let kind = super::super::super::observe(selected, child, &mut walk.root)
            .err()
            .ok_or("recovered preparation admitted")?;
        let Kind::InterpolationPreparation { span, kind: error } = kind else {
            return Err("precise original preparation kind");
        };
        same(error, NativeInterpolationError::RecoveredComponent)?;
        same(span.slice(source), "{{ 7 }}")?;
        same(
            walk.reject::<()>(kind)
                .err()
                .ok_or("failure admitted")?
                .span,
            span,
        )?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("normally owned failure File")?;
    check(file.native_interpolations().is_empty())?;
    let [failure] = file.native_interpolation_failures() else {
        return Err("one complete preparation failure");
    };
    same(failure.span().slice(source), "{{ 7 }}")?;
    same(
        failure.failure().kind(),
        NativeInterpolationError::RecoveredComponent,
    )?;
    check(core::ptr::eq(
        output.selected().component().block().root_source(),
        source,
    ))
}
