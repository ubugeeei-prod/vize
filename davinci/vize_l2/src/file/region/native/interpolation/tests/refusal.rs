use super::{Allocator, Kind, NativeInterpolationInput, State, Test, check, owner, same, selected};
use vize_l0::id::NodeId;

#[test]
fn repeated_root_event_retains_original_prefix_and_refused_input_without_duplicate_node() -> Test {
    let arena = Allocator::default();
    let source = "<template>{{ 7 }}</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("child")?;
        let first = NativeInterpolationInput::from_operand(
            selected
                .observe_interpolation_expression(child.reborrow())
                .map_err(|_| "first owner")?,
        );
        let duplicate = NativeInterpolationInput::from_operand(
            selected
                .observe_interpolation_expression(child.reborrow())
                .map_err(|_| "duplicate owner")?,
        );
        same(
            walk.root_interpolation(child.reborrow(), first)
                .map_err(|_| "first actual event")?,
            NodeId::FIRST,
        )?;
        let issue = walk
            .root_interpolation(child, duplicate)
            .err()
            .ok_or("duplicate admitted")?;
        same(issue.kind, Kind::InvalidEvent)?;
        same(
            walk.complete().err().ok_or("refused root completed")?.kind,
            Kind::InvalidEvent,
        )?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("original prefix File")?;
    same(file.artifact().node_count(), 1)?;
    let [first, refused] = file.native_interpolations() else {
        return Err("both normal owners");
    };
    same(first.state(), State::Admitted(NodeId::FIRST))?;
    same(refused.state(), State::Refused(Kind::InvalidEvent))?;
    check(core::ptr::eq(
        file.native_interpolation(NodeId::FIRST)
            .ok_or("actual node association")?,
        first,
    ))?;
    same(
        refused.input().operand().full_span().slice(source),
        "{{ 7 }}",
    )
}

#[test]
fn foreign_equal_source_and_nested_events_never_advance_the_root_cursor() -> Test {
    for nested in [false, true] {
        let arena = Allocator::default();
        let source = if nested {
            "<template><p>{{ 7 }}</p></template>"
        } else {
            "<template>{{ 7 }}</template>"
        };
        let mut original = owner(&arena, source)?;
        let foreign = selected(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            if nested {
                let selected = walk.selected();
                let element = selected
                    .children()
                    .next()
                    .ok_or("parent")?
                    .into_element()
                    .ok_or("Element")?;
                let child = element.children().next().ok_or("nested child")?;
                let input = NativeInterpolationInput::from_operand(
                    selected
                        .observe_interpolation_expression(child.reborrow())
                        .map_err(|_| "nested observation")?,
                );
                same(
                    walk.root_interpolation(child, input)
                        .err()
                        .ok_or("nested admitted")?
                        .kind,
                    Kind::InvalidEvent,
                )?;
            } else {
                let child = foreign.children().next().ok_or("foreign child")?;
                let input = NativeInterpolationInput::from_operand(
                    foreign
                        .observe_interpolation_expression(child.reborrow())
                        .map_err(|_| "foreign observation")?,
                );
                same(
                    walk.root_interpolation(child, input)
                        .err()
                        .ok_or("foreign admitted")?
                        .kind,
                    Kind::InvalidEvent,
                )?;
            }
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("retained File")?;
        same(file.artifact().node_count(), 0)?;
        let [record] = file.native_interpolations() else {
            return Err("normal refused input");
        };
        same(record.state(), State::Refused(Kind::InvalidEvent))?;
        check(record.input().operand().syntax().expression().is_some())?;
        check(file.native_interpolation(NodeId::FIRST).is_none())?;
    }
    Ok(())
}

#[test]
fn syntax_holes_and_unresolved_reads_retain_full_owner_and_actual_static_prefix() -> Test {
    for content in ["", "value +", "missing"] {
        let arena = Allocator::default();
        let source = alloc::format!("<template>prefix{{{{ {content} }}}}</template>");
        let mut original = owner(&arena, &source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let mut children = selected.children();
            walk.child(children.next().ok_or("prefix")?)
                .map_err(|_| "actual prefix")?;
            let child = children.next().ok_or("original interpolation")?;
            let input = NativeInterpolationInput::from_operand(
                selected
                    .observe_interpolation_expression(child.reborrow())
                    .map_err(|_| "normal whole syntax")?,
            );
            let issue = walk
                .root_interpolation(child, input)
                .err()
                .ok_or("invalid syntax/read admitted")?;
            check(if content == "missing" {
                matches!(issue.kind, Kind::Artifact(_))
            } else {
                matches!(issue.kind, Kind::Interpolation(_))
            })?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("actual prefix File")?;
        same(file.artifact().node_count(), 1)?;
        let [record] = file.native_interpolations() else {
            return Err("retained refused owner");
        };
        check(matches!(record.state(), State::Refused(_)))?;
        check(core::ptr::eq(
            record.input().operand().syntax().source().authored_root(),
            &*source,
        ))?;
        same(
            record.input().operand().raw_content(),
            alloc::format!(" {content} ").as_str(),
        )?;
        check(file.native_interpolation(NodeId::FIRST).is_none())?;
        same(file.expression(NodeId::FIRST).is_some(), false)?;
    }
    Ok(())
}
