use super::*;

mod admission;
mod interruption;
mod refusal;

#[test]
fn original_js_ts_empty_default_spans_comments_directives_and_layout_stay_bound()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for lang in ["", " lang='ts'"] {
        for body in [
            "export default {}",
            "/*雪🌸*/\r\n'custom';'use strict';;;export /*between*/ default { /*inside*/ };; //tail\r\n",
        ] {
            let source = format!("<script{lang}>{body}</script><template>original</template>");
            let original = Observed::new(&arena, &source)?;
            let parser = original
                .syntax
                .admitted_program()
                .ok_or("original Program")?;
            let body_pointer = parser.program().body.as_ptr();
            let directives = parser.program().directives.len();
            let producer = original.producer(&arena)?;
            let preflight = producer.ordinary_empty_script().ok_or("script proof")?;
            equal(preflight.scope().index(), 0)?;
            equal(preflight.script_span(), original.script()?.block().span())?;
            let statement = preflight.statement_span();
            let file = core::hint::black_box(producer.finish().map_err(|_| "File")?);
            let view = original.checked(&file)?.map_err(|_| "ordinary proof")?;
            check(core::ptr::eq(view.file(), &file))?;
            check(core::ptr::eq(view.source().root_source(), source.as_str()))?;
            check(core::ptr::eq(view.program().program(), parser.program()))?;
            equal(view.program().program().body.as_ptr(), body_pointer)?;
            equal(view.program().program().directives.len(), directives)?;
            equal(view.statement_span(), statement)?;
            equal(file.bindings().count(), 0)?;
            equal(file.references().len(), 0)?;
            equal(file.imports().len(), 0)?;
            equal(file.exports().len(), 1)?;
            equal(view.unit(), file.units().first().ok_or("unit")?.id)?;
            let slice = |span: vize_l0::Span| source.get(span.start as usize..span.end as usize);
            equal(slice(view.export_keyword_span()), Some("export"))?;
            equal(slice(view.default_keyword_span()), Some("default"))?;
            equal(
                slice(view.object_span()),
                Some(if body.contains("inside") {
                    "{ /*inside*/ }"
                } else {
                    "{}"
                }),
            )?;
            equal(
                view.original().statement_span(),
                parser
                    .sole_default_export()
                    .ok_or("entry")?
                    .statement_span(),
            )?;
            equal(original.syntax.diagnostics().count(), 0)?;
            equal(
                original.syntax.comments().count(),
                if body.contains("inside") { 4 } else { 0 },
            )?;
            check(file.is_complete())?;
        }
    }
    if cfg!(target_pointer_width = "64") {
        equal(core::mem::size_of::<vize_l2::file::ScriptUnit>(), 48)?;
        equal(core::mem::align_of::<vize_l2::file::ScriptUnit>(), 8)?;
    }
    Ok(())
}

#[test]
fn normally_moved_parser_and_file_owners_keep_the_original_certificate() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let original = Observed::new(
        &arena,
        "<script lang='ts'>/*kept*/export default {};</script>",
    )?;
    let body = original.syntax.program().ok_or("Program")?.body.as_ptr();
    let comments = original
        .syntax
        .program()
        .ok_or("Program")?
        .comments
        .as_ptr();
    let file = Box::new(original.file(&arena)?);
    let original = Box::new(original);
    let view = original.checked(&file)?.map_err(|_| "moved proof")?;
    equal(view.program().program().body.as_ptr(), body)?;
    equal(view.program().program().comments.as_ptr(), comments)?;
    check(core::ptr::eq(view.file(), &*file))?;
    equal(view.source().source(), "/*kept*/export default {};")?;
    Ok(())
}
