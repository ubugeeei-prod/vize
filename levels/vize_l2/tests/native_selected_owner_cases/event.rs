use super::{Allocator, Kind, check, descriptor, equal, owner, selected, syntax};
use vize_l0::Span;
use vize_l2::{
    file::{FileIssueKind, RejectedFileHandler},
    op::{BindingOp, OnHandlerRef, Op},
    resolution::HandlerBindingRef,
};

#[test]
fn original_header_body_and_file_handlers_preserve_owner_binding_child_order()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><button id='one' @click='let x=$event; return x' title='two'>ok<span @blur='return $event'/></button></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "actual child")?;
        }
        walk.complete().map_err(|_| "normal root")?;
    }
    let output = core::hint::black_box(original.finish());
    let file = output
        .view()
        .map_err(|_| "native view")?
        .file()
        .ok_or("File")?;
    equal(file.artifact().node_count(), 5)?;
    check(file.unattached_handlers().next().is_none())?;
    let [Op::Element(button)] = file.artifact().root().ops.as_slice() else {
        return Err("button");
    };
    equal(button.attributes.len(), 2)?;
    equal(button.attributes[0].value, Some("one"))?;
    equal(button.attributes[1].value, Some("two"))?;
    let [BindingOp::On(on)] = button.bindings.as_slice() else {
        return Err("attached click");
    };
    let [Op::Text(text), Op::Element(span)] = button.children.ops.as_slice() else {
        return Err("original children");
    };
    equal(text.content, "ok")?;
    let [BindingOp::On(blur)] = span.bindings.as_slice() else {
        return Err("attached blur");
    };
    let (Some(OnHandlerRef::Body(click_id)), Some(OnHandlerRef::Body(blur_id))) =
        (on.handler, blur.handler)
    else {
        return Err("genuine body refs");
    };
    equal(click_id.node().index(), 1)?;
    equal(blur_id.node().index(), 4)?;
    let click = file
        .handler_for(on)
        .ok_or("click same File and actual On")?;
    let nested = file
        .handler_for(blur)
        .ok_or("blur same File and actual On")?;
    check(click.same_owner(nested))?;
    check(core::ptr::eq(click.file(), file))?;
    equal(click.scope().ok_or("scope")?.index(), 0)?;
    let facts = click.resolution().ok_or("whole handler facts")?;
    equal(
        facts.input().operand().raw_value(),
        "let x=$event; return x",
    )?;
    equal(facts.references().len(), 2)?;
    equal(facts.references()[0].span, Span::new(6, 12))?;
    equal(facts.references()[1].span, Span::new(21, 22))?;
    check(matches!(
        facts.references()[0].binding,
        HandlerBindingRef::Local(_)
    ))?;
    check(core::ptr::eq(
        facts.input().body(),
        facts
            .input()
            .operand()
            .syntax()
            .body()
            .ok_or("stock body")?,
    ))?;
    let original_element = output
        .selected()
        .children()
        .next()
        .ok_or("root")?
        .into_element()
        .ok_or("Element")?;
    let attribute = original_element
        .attributes()
        .nth(1)
        .ok_or("original event")?;
    check(
        facts
            .input()
            .admitted_for(output.selected(), attribute)
            .is_some(),
    )?;
    let independent = selected(&arena, source)?;
    let foreign = independent
        .children()
        .next()
        .ok_or("foreign root")?
        .into_element()
        .ok_or("foreign Element")?;
    check(
        facts
            .input()
            .admitted_for(
                &independent,
                foreign.attributes().nth(1).ok_or("foreign event")?,
            )
            .is_none(),
    )?;
    equal(on.span.slice(source), "@click='let x=$event; return x'")?;
    equal(
        facts.input().operand().argument_span().slice(source),
        "click",
    )?;
    Ok(())
}

#[test]
fn a_later_refused_header_keeps_the_unattached_original_body_and_diagnostics()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<template>prefix<div @click='/*keep*/ return $event' :id='bad'>unvisited</div></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let mut children = walk.selected().children();
        walk.child(children.next().ok_or("prefix")?)
            .map_err(|_| "prefix")?;
        equal(
            walk.child(children.next().ok_or("refused Element")?)
                .err()
                .ok_or("refusal")?
                .kind,
            Kind::UnsupportedChild,
        )?;
        check(walk.complete().is_err())?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("partial File")?;
    equal(file.artifact().node_count(), 1)?;
    let mut pending = file.unattached_handlers();
    let input = pending.next().ok_or("parked original")?;
    check(pending.next().is_none())?;
    equal(input.operand().raw_value(), "/*keep*/ return $event")?;
    equal(input.operand().syntax().comments().count(), 1)?;
    check(input.operand().syntax().admitted_body().is_some())?;
    check(!file.is_complete())?;
    Ok(())
}

#[test]
fn actual_header_syntax_and_semantic_refusals_keep_complete_owners_and_precise_ranges()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (body, expected) in [
        ("/*kept*/ return missing", "missing"),
        ("while(true){}", "while(true){}"),
        ("return (", "return ("),
        ("return /x/uv; // kept", "return /x/uv; // kept"),
    ] {
        let source = format!("<template><button @click='{body}'/></template>");
        let mut original = owner(&arena, &source)?;
        let issue;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            issue = walk
                .child(walk.selected().children().next().ok_or("original root")?)
                .err()
                .ok_or("precise refusal")?;
            equal(walk.complete().err(), Some(issue))?;
        }
        let Kind::Handler { span, kind } = issue.kind else {
            return Err("handler refusal");
        };
        equal(span.slice(&source), expected)?;
        equal(issue.span, span)?;
        equal(
            kind,
            if body.contains("missing") {
                FileIssueKind::UnresolvedReference
            } else {
                FileIssueKind::UnsupportedSyntax
            },
        )?;
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("original partial File")?;
        equal(file.artifact().node_count(), 0)?;
        let [rejected] = file.rejected_handlers() else {
            return Err("retained rejection");
        };
        match rejected {
            RejectedFileHandler::Resolution { input, .. } => {
                equal(input.operand().raw_value(), body)?
            }
            RejectedFileHandler::Syntax(input) => {
                equal(input.operand().raw_value(), body)?;
                let syntax = input.operand().syntax();
                equal(syntax.source().text(), body)?;
                equal(syntax.admitted_body().is_none(), true)?;
                if body == "return (" {
                    // The unchanged resource guard refuses unbalanced framing
                    // before parsing; do not invent a stock syntax diagnostic.
                    equal(
                        syntax.hole(),
                        Some(vize_l1::embed::syntax::EmbedHole::SafetyAdmission),
                    )?;
                    equal(syntax.diagnostics().count(), 0)?;
                } else {
                    equal(
                        syntax.hole(),
                        Some(vize_l1::embed::syntax::EmbedHole::Syntax),
                    )?;
                    check(syntax.diagnostics().next().is_some())
                        .map_err(|_| "stock syntax diagnostic")?;
                    equal(
                        syntax
                            .comments()
                            .next()
                            .ok_or("stock comment")?
                            .text()
                            .map_err(|_| "comment span")?,
                        "// kept",
                    )?;
                }
            }
            _ => return Err("original syntax or semantic owner"),
        }
    }
    Ok(())
}

mod identity;
mod source;
