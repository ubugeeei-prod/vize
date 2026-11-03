use super::*;

#[test]
fn entities_comments_and_empty_handlers_keep_original_profiles_and_maps() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    for body in ["", "/*original*/", "return $event &amp;&amp; 1"] {
        let source = format!("<template><button v-on:click='{body}'/></template>");
        let mut original = owner(&arena, &source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            walk.child(walk.selected().children().next().ok_or("root")?)
                .map_err(|_| "original event")?;
            walk.complete().map_err(|_| "normal end")?;
        }
        let output = original.finish();
        let file = output.view().map_err(|_| "native")?.file().ok_or("File")?;
        let [Op::Element(button)] = file.artifact().root().ops.as_slice() else {
            return Err("button");
        };
        let [BindingOp::On(on)] = button.bindings.as_slice() else {
            return Err("event");
        };
        let id = on.handler.and_then(OnHandlerRef::body).ok_or("body ref")?;
        let facts = file
            .handler(id)
            .ok_or("same File")?
            .resolution()
            .ok_or("facts")?;
        let syntax = facts.input().operand().syntax();
        equal(facts.input().operand().raw_value(), body)?;
        check(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str(),
        ))?;
        equal(syntax.parser_prefix(), 6)?;
        if body.contains('&') {
            equal(syntax.source().text(), "return $event && 1")?;
            check(syntax.source().decode_map().is_some())?;
            let amp = syntax
                .source()
                .authored_span(Span::new(14, 16))
                .map_err(|_| "entity projection")?;
            equal(amp.slice(&source), "&amp;&amp;")?;
        }
    }
    Ok(())
}

#[test]
fn authentic_completed_setup_bindings_resolve_in_the_exact_original_handler_scope()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<script setup>const value=1;</script><template><button @click='return value'/></template>";
    let descriptor = super::descriptor(&arena, source);
    let program = syntax(&arena, &descriptor, true)?;
    let mut original = owner(&arena, source)?;
    original
        .setup_program(program.admitted_program().ok_or("original Program")?)
        .map_err(|_| "setup")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        walk.child(walk.selected().children().next().ok_or("root")?)
            .map_err(|_| "genuine visible setup")?;
        walk.complete().map_err(|_| "normal root")?;
    }
    let output = original.finish();
    let file = output
        .view()
        .map_err(|_| "native File")?
        .file()
        .ok_or("File")?;
    equal(file.bindings().count(), 1)?;
    check(file.rejected_handlers().is_empty())?;
    check(file.unattached_handlers().next().is_none())?;
    let [unit] = file.units() else {
        return Err("exact original setup unit");
    };
    let binding = file
        .lookup(unit.scope, "value", vize_l2::file::Namespace::Value)
        .ok_or("actual setup binding")?;
    let declaration = binding.declaration().ok_or("actual script declaration")?;
    equal(declaration.script_unit(), Some(unit.id))?;
    equal(declaration.scope, unit.scope)?;
    let [Op::Element(button)] = file.artifact().root().ops.as_slice() else {
        return Err("button");
    };
    let [BindingOp::On(on)] = button.bindings.as_slice() else {
        return Err("actual attached event");
    };
    let handler = file.handler_for(on).ok_or("exact File/On join")?;
    equal(handler.scope(), Some(unit.scope))?;
    check(handler.accepts(binding))?;
    let facts = handler.resolution().ok_or("whole handler facts")?;
    let [reference] = facts.references() else {
        return Err("one actual reference");
    };
    equal(reference.binding, HandlerBindingRef::Outer(binding.id()))?;
    equal(reference.span, Span::new(7, 12))?;
    equal(facts.input().operand().raw_value(), "return value")?;
    check(core::ptr::eq(
        facts.input().operand().syntax().source().authored_root(),
        source,
    ))?;
    let element = output
        .selected()
        .children()
        .next()
        .ok_or("original root")?
        .into_element()
        .ok_or("original Element")?;
    let attribute = element.attributes().next().ok_or("original event")?;
    check(
        facts
            .input()
            .admitted_for(output.selected(), attribute)
            .is_some(),
    )?;
    let foreign = selected(&arena, source)?;
    let foreign_element = foreign
        .children()
        .next()
        .ok_or("foreign root")?
        .into_element()
        .ok_or("foreign Element")?;
    check(
        facts
            .input()
            .admitted_for(
                &foreign,
                foreign_element.attributes().next().ok_or("foreign event")?,
            )
            .is_none(),
    )?;
    Ok(())
}

#[test]
fn ordinary_script_rows_never_gain_selected_setup_visibility() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<script>const value=1;</script><template><button @click='return value'/></template>";
    let descriptor = super::descriptor(&arena, source);
    let program = syntax(&arena, &descriptor, false)?;
    let mut original = owner(&arena, source)?;
    original
        .ordinary_program(program.admitted_program().ok_or("original Program")?)
        .map_err(|_| "setup")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let issue = walk
            .child(walk.selected().children().next().ok_or("root")?)
            .err()
            .ok_or("visibility refusal")?;
        let Kind::Handler { span, kind } = issue.kind else {
            return Err("handler refusal");
        };
        equal(kind, FileIssueKind::UnresolvedReference)?;
        equal(span.slice(source), "value")?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("File")?;
    equal(file.bindings().count(), 1)?;
    equal(file.rejected_handlers().len(), 1)?;
    Ok(())
}

#[test]
fn actual_file_handlers_retain_intrinsic_ts_profile_and_precise_annotation_refusal()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (body, supported) in [
        ("/*typed*/ let x=1;x", true),
        ("/*typed*/ let x:number=1;x", false),
    ] {
        let source = format!(
            "<script setup lang='ts'></script><template><button @click='{body}'/></template>"
        );
        let descriptor = super::descriptor(&arena, &source);
        let program = syntax(&arena, &descriptor, true)?;
        let mut original = owner(&arena, &source)?;
        original
            .setup_program(program.admitted_program().ok_or("original TS Program")?)
            .map_err(|_| "setup")?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let result = walk.child(walk.selected().children().next().ok_or("root")?);
            if supported {
                result.map_err(|_| "unannotated TS")?;
                walk.complete().map_err(|_| "normal root")?;
            } else {
                let Kind::Handler { span, kind } = result.err().ok_or("annotation refusal")?.kind
                else {
                    return Err("precise handler refusal");
                };
                equal(kind, FileIssueKind::UnsupportedSyntax)?;
                equal(span.slice(&source), "x:number=1")?;
                check(walk.complete().is_err())?;
            }
        }
        let output = original.finish();
        let file = output.file().ok_or("File")?;
        let input = if supported {
            let [Op::Element(button)] = file.artifact().root().ops.as_slice() else {
                return Err("button");
            };
            let [BindingOp::On(on)] = button.bindings.as_slice() else {
                return Err("actual On");
            };
            file.handler_for(on)
                .ok_or("exact membership")?
                .resolution()
                .ok_or("whole facts")?
                .input()
        } else {
            let [RejectedFileHandler::Resolution { input, .. }] = file.rejected_handlers() else {
                return Err("whole typed owner");
            };
            input
        };
        equal(input.operand().raw_value(), body)?;
        check(input.operand().syntax().source_type().is_typescript())?;
        equal(input.operand().syntax().comments().count(), 1)?;
        equal(output.view().is_ok(), supported)?;
    }
    Ok(())
}
