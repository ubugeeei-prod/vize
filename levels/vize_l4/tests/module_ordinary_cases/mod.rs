use super::*;

mod assembly;

#[test]
fn original_recorded_and_unrecorded_fragments_keep_exact_leading_object_and_tail_bytes()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for lang in ["", " lang='ts'"] {
        for (leading, prefix, object, tail, comments) in [
            ("", "export default ", "{}", ";", 0),
            ("", "export default ", "{}", "", 0),
            (
                "/*雪🌸*/\r\n'custom';'use strict';;;",
                "export /*removedA*/ default /*removedB*/ ",
                "{ /*inside*/ }",
                ";; //tail",
                5,
            ),
            (
                ";\r\n",
                "export\n/*removedA*/default\r\n/*removedB*/",
                "{}",
                " //tail\r\n; /*end*/",
                4,
            ),
        ] {
            let source = format!(
                "<template>雪🌸</template><script{lang}>{leading}{prefix}{object}{tail}</script>"
            );
            let observed = Observed::new(&arena, &source)?;
            let program = observed.syntax.program().ok_or("original Program")?;
            let file = observed.file(&arena)?;
            let view = observed.checked(&file)?;
            let recorded = emit_ordinary_empty::<Recorded>(&view)
                .map_err(|_| "recorded writer")?
                .finish();
            let plain = emit_ordinary_empty::<NoLinks>(&view)
                .map_err(|_| "plain writer")?
                .finish();
            let expected = format!("{leading}const {COMPONENT_BINDING} = {object}{tail}");
            equal(recorded.text.as_str(), expected.as_str())?;
            equal(&recorded.text, &plain.text)?;
            equal(&recorded.helpers, &plain.helpers)?;
            check(recorded.helpers.is_empty())?;
            equal(
                recorded.links.links().len(),
                if leading.is_empty() { 1 } else { 2 },
            )?;
            for link in recorded.links.links() {
                equal(
                    source.get(link.authored.start as usize..link.authored.end as usize),
                    recorded
                        .text
                        .get(link.generated.start as usize..link.generated.end as usize),
                )?;
                check(link.segment && link.name.is_none())?;
                check(
                    link.authored.end <= view.statement_span().start
                        || link.authored.start >= view.object_span().start,
                )?;
            }
            if !leading.is_empty() {
                let first = recorded.links.links().first().ok_or("leading link")?;
                equal(
                    first.authored,
                    Span::new(view.source().start(), view.statement_span().start),
                )?;
                equal(first.generated, Span::new(0, leading.len() as u32))?;
            }
            let last = recorded.links.links().last().ok_or("object/tail link")?;
            equal(
                last.authored,
                Span::new(view.object_span().start, view.source().end()),
            )?;
            equal(
                last.generated,
                Span::new(
                    (leading.len() + format!("const {COMPONENT_BINDING} = ").len()) as u32,
                    recorded.text.len() as u32,
                ),
            )?;
            check(core::ptr::eq(view.program().program(), program))?;
            check(core::ptr::eq(view.file(), &file))?;
            check(core::ptr::eq(view.source().root_source(), source.as_str()))?;
            equal(observed.syntax.comments().count(), comments)?;
            equal(observed.syntax.diagnostics().count(), 0)?;
            equal(file.bindings().count(), 0)?;
            check(file.is_complete())?;
            check(plain.into_document().links().is_empty())?;
        }
    }
    Ok(())
}

#[test]
fn moved_original_owners_keep_the_same_full_file_recording_and_unmodified_observation()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script lang='ts'>/*leading*/export /*removed*/ default { /*inside*/ }; //tail\r\n</script>";
    let observed = Observed::new(&arena, source)?;
    let body = observed.syntax.program().ok_or("Program")?.body.as_ptr();
    let comments = observed
        .syntax
        .program()
        .ok_or("Program")?
        .comments
        .as_ptr();
    let file = Box::new(observed.file(&arena)?);
    let observed = Box::new(observed);
    let view = observed.checked(&file)?;
    let emitted = emit_ordinary_empty::<Recorded>(&view)
        .map_err(|_| "emit")?
        .finish();
    equal(
        emitted.text.as_str(),
        "/*leading*/const _sfc_main = { /*inside*/ }; //tail\r\n",
    )?;
    equal(view.program().program().body.as_ptr(), body)?;
    equal(view.program().program().comments.as_ptr(), comments)?;
    equal(observed.syntax.comments().count(), 4)?;
    check(core::ptr::eq(view.source().root_source(), source))?;
    let expected_text = "/*leading*/const _sfc_main = { /*inside*/ }; //tail\r\n";
    let leading_length = "/*leading*/".len() as u32;
    let generated_tail = leading_length + "const _sfc_main = ".len() as u32;
    let expected = vize_l4::write::EmitDocument::from_parts(
        vize_l0::String::from(expected_text),
        vec![
            vize_l4::write::SpanLink {
                generated: Span::new(0, leading_length),
                authored: Span::new(view.source().start(), view.statement_span().start),
                name: None,
                segment: true,
            },
            vize_l4::write::SpanLink {
                generated: Span::new(generated_tail, expected_text.len() as u32),
                authored: Span::new(view.object_span().start, view.source().end()),
                name: None,
                segment: true,
            },
        ],
    );
    equal(emitted.into_document(), expected)?;
    Ok(())
}

#[test]
fn foreign_parser_and_nonempty_options_cannot_supply_emitter_input() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>/*original*/export default {};</script>";
    let observed = Observed::new(&arena, source)?;
    let file = observed.file(&arena)?;
    let script = observed.script()?;
    let second = oxc_parser::Parser::new(
        arena.as_oxc(),
        script.block().source(),
        oxc_span::SourceType::mjs(),
    )
    .parse_observed();
    check(
        VueOrdinaryEmpty::checked(&file, script, second.admitted().ok_or("actual second")?)
            .is_err(),
    )?;
    for source in [
        "<script>export default {name:'options'};</script>",
        "<script>export default ({});</script>",
        "<script lang='ts'>export default {} as const;</script>",
    ] {
        let observed = Observed::new(&arena, source)?;
        let file = observed.file(&arena)?;
        check(observed.checked(&file).is_err())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
        equal(observed.syntax.diagnostics().count(), 0)?;
    }
    Ok(())
}
