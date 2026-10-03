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
fn recorded_setup_bindings_cannot_bypass_the_actual_native_visibility_policy()
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
