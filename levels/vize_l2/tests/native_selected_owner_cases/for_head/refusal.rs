use super::*;
use vize_l1::embed::syntax::{ForHeadPart, NativeForRefusal};
use vize_l2::{
    file::{FileIssueKind, RejectedFileFor, RejectedFileHandler},
    resolution::{ForResolutionErrorKind, ResolutionErrorKind},
};

#[test]
fn syntax_source_and_pattern_refusals_keep_the_complete_original_for_owners()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (value, expected) in [
        ("item in value + /*kept*/", None),
        (
            "item /*original*/ in items",
            Some(NativeForRefusal::Comment),
        ),
        ("{item} in items", Some(NativeForRefusal::AliasShape)),
        ("item in &#105;tems", Some(NativeForRefusal::EntityOutput)),
    ] {
        let source = format!(
            "<script setup lang='ts'>const items=2;</script><template><div v-for='{value}'/></template>",
        );
        let descriptor = descriptor(&arena, &source);
        let program = syntax(&arena, &descriptor, true)?;
        let mut original = owner(&arena, &source)?;
        original
            .setup_program(program.admitted_program().ok_or("TS setup Program")?)
            .map_err(|_| "setup")?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let issue = walk
                .child(walk.selected().children().next().ok_or("original root")?)
                .err()
                .ok_or("actual refusal")?;
            let Kind::For { span, kind } = issue.kind else {
                return Err("For syntax refusal");
            };
            equal(kind, FileIssueKind::UnsupportedSyntax)?;
            equal(span.slice(&source), value)?;
            equal(issue.span, span)?;
            equal(walk.complete().err(), Some(issue))?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("whole partial File")?;
        equal(file.artifact().node_count(), 0)?;
        equal(file.template_declarations().count(), 0)?;
        let [RejectedFileFor::Syntax(rejected)] = file.rejected_for_heads() else {
            return Err("complete typed syntax rejection");
        };
        let operand = rejected.operand();
        equal(operand.raw_value(), value)?;
        equal(operand.value_span().slice(&source), value)?;
        check(core::ptr::eq(
            operand.syntax().source().authored_root(),
            source.as_str(),
        ))?;
        equal(operand.syntax().grammar().lang, vize_l1::embed::Lang::Ts)?;
        if let Some(expected) = expected {
            equal(rejected.kind, expected)?;
        } else {
            check(operand.syntax().diagnostics().next().is_some())?;
            equal(operand.syntax().comments().count(), 1)?;
        }
        if value.contains("/*original*/") {
            equal(operand.syntax().comments().count(), 1)?;
        }
        if value.contains('&') {
            check(operand.syntax().source().decode_map().is_some())?;
        }
        check(operand.syntax().aliases().is_some())?;
        check(operand.syntax().collection().is_some())?;
        check(!file.is_complete())?;
    }
    Ok(())
}

#[test]
fn unbound_collection_and_alias_policy_fail_before_any_for_node_or_scope_is_minted()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (value, expected, part, authored) in [
        (
            "item in missing",
            ForResolutionErrorKind::Reference(ResolutionErrorKind::MissingBinding),
            ForHeadPart::Collection,
            "missing",
        ),
        (
            "(item,item) in items",
            ForResolutionErrorKind::DuplicateAlias,
            ForHeadPart::Aliases,
            "item",
        ),
        (
            "_helper in items",
            ForResolutionErrorKind::ReservedAlias,
            ForHeadPart::Aliases,
            "_helper",
        ),
    ] {
        let source = format!(
            "<script setup>const items=2;</script><template><div v-for='{value}'/></template>",
        );
        let descriptor = descriptor(&arena, &source);
        let program = syntax(&arena, &descriptor, true)?;
        let mut original = owner(&arena, &source)?;
        original
            .setup_program(program.admitted_program().ok_or("setup Program")?)
            .map_err(|_| "setup")?;
        let reported_span;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let issue = walk
                .child(walk.selected().children().next().ok_or("original root")?)
                .err()
                .ok_or("semantic refusal")?;
            let Kind::For { span, kind } = issue.kind else {
                return Err("precise For refusal");
            };
            equal(
                kind,
                if part == ForHeadPart::Collection {
                    FileIssueKind::UnresolvedReference
                } else {
                    FileIssueKind::UnsupportedSyntax
                },
            )?;
            // The original part's coordinate map, not the complete head or an
            // invented script wrapper, selects this precise error spelling.
            equal(span.slice(&source), authored)?;
            equal(issue.span, span)?;
            reported_span = span;
            equal(walk.complete().err(), Some(issue))?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("whole rejected File")?;
        equal(file.artifact().node_count(), 0)?;
        equal(file.scopes().len(), 2)?;
        equal(file.bindings().count(), 1)?;
        equal(file.template_declarations().count(), 0)?;
        let [RejectedFileFor::Resolution { input, error }] = file.rejected_for_heads() else {
            return Err("whole semantic input");
        };
        equal(error.kind, expected)?;
        equal(error.part, part)?;
        let syntax = input.operand().syntax();
        let part_source = match part {
            ForHeadPart::Aliases => syntax
                .aliases()
                .ok_or("original aliases")?
                .map_err(|_| "alias owner")?
                .source(),
            ForHeadPart::Collection => syntax
                .collection()
                .ok_or("original collection")?
                .map_err(|_| "collection owner")?
                .source(),
        };
        equal(
            reported_span,
            part_source
                .authored_covering_span(error.span)
                .map_err(|_| "original part projection")?,
        )?;
        equal(input.operand().raw_value(), value)?;
        check(input.operand().syntax().admitted_dense().is_some())?;
        let element = output
            .selected()
            .children()
            .next()
            .ok_or("original root")?
            .into_element()
            .ok_or("original Element")?;
        check(
            input
                .admitted_for(
                    output.selected(),
                    element.attributes().next().ok_or("original For")?,
                )
                .is_some(),
        )?;
    }
    Ok(())
}

#[test]
fn later_unsupported_attribute_retains_the_parked_original_for_before_any_mint()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template>prefix<div v-for='item in items' @click='item' :id='bad'/></template>";
    let descriptor = descriptor(&arena, source);
    let program = syntax(&arena, &descriptor, true)?;
    let mut original = owner(&arena, source)?;
    original
        .setup_program(program.admitted_program().ok_or("setup Program")?)
        .map_err(|_| "setup")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let mut children = walk.selected().children();
        walk.child(children.next().ok_or("prefix")?)
            .map_err(|_| "prefix mint")?;
        equal(
            walk.child(children.next().ok_or("actual Element")?)
                .err()
                .ok_or("late refusal")?
                .kind,
            Kind::UnsupportedChild,
        )?;
        check(walk.complete().is_err())?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("partial File")?;
    equal(file.artifact().node_count(), 1)?;
    equal(file.template_declarations().count(), 0)?;
    let mut pending = file.unattached_for_heads();
    let input = pending.next().ok_or("parked normal For input")?;
    check(pending.next().is_none())?;
    equal(input.operand().raw_value(), "item in items")?;
    let element = output
        .selected()
        .children()
        .nth(1)
        .ok_or("original Element child")?
        .into_element()
        .ok_or("original Element")?;
    check(
        input
            .admitted_for(
                output.selected(),
                element.attributes().next().ok_or("original For token")?,
            )
            .is_some(),
    )?;
    let mut handlers = file.unattached_handlers();
    equal(
        handlers
            .next()
            .ok_or("earlier parked event")?
            .operand()
            .raw_value(),
        "item",
    )?;
    check(handlers.next().is_none())?;
    check(file.rejected_for_heads().is_empty())?;
    Ok(())
}

#[test]
fn a_next_root_sibling_cannot_observe_an_alias_from_the_previous_original_for()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'/><button @click='item'/></template>";
    let descriptor = descriptor(&arena, source);
    let program = syntax(&arena, &descriptor, true)?;
    let mut original = owner(&arena, source)?;
    original
        .setup_program(program.admitted_program().ok_or("setup Program")?)
        .map_err(|_| "setup")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let mut children = walk.selected().children();
        walk.child(children.next().ok_or("original For child")?)
            .map_err(|_| "For mint")?;
        let issue = walk
            .child(children.next().ok_or("next original root")?)
            .err()
            .ok_or("no alias leakage")?;
        let Kind::Handler { span, kind } = issue.kind else {
            return Err("next-root handler refusal");
        };
        equal(span.slice(source), "item")?;
        equal(kind, FileIssueKind::UnresolvedReference)?;
        equal(walk.complete().err(), Some(issue))?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("partial File")?;
    equal(file.artifact().node_count(), 2)?;
    equal(file.template_declarations().count(), 1)?;
    let [Op::OriginalFor(original_for)] = file.artifact().root().ops.as_slice() else {
        return Err("completed first original For remains");
    };
    let joined = file.for_head_for(original_for).ok_or("first exact join")?;
    check(
        file.lookup(
            joined.enclosing_scope().ok_or("parent scope")?,
            "item",
            Namespace::Value,
        )
        .is_none(),
    )?;
    let [RejectedFileHandler::Resolution { input, .. }] = file.rejected_handlers() else {
        return Err("retained next-root original handler");
    };
    equal(input.operand().raw_value(), "item")?;
    Ok(())
}
