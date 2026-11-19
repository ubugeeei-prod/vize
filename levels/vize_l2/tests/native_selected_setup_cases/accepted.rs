use super::*;

#[test]
fn original_js_ts_whole_program_and_actual_unit_survive_owner_moves() -> Result<(), &'static str> {
    for source in [
        "<!--🦀--><script setup>/*whole*/ const snow='雪'; let count=1; var ready=true; const large=3n;</script><template><div/></template>",
        "<template><div/></template><script setup lang=ts>/*whole*/ const snow:string='雪'; let count:number=1; var ready:boolean=true; const large:bigint=3n;</script>",
    ] {
        let arena = Allocator::default();
        let original = descriptor(&arena, source);
        let script = original
            .admitted()
            .map_err(|_| "original descriptor")?
            .setup()
            .ok_or("setup")?;
        let mut owner = owner(&arena, source)?;
        check(core::ptr::eq(
            owner.selected().component().allocator(),
            &arena,
        ))?;
        let unit = owner.parse_setup_program().map_err(|_| "owning setup")?;
        let syntax = owner.retained_setup().ok_or("parked stock owner")?;
        let body = syntax.program().ok_or("original Program")?.body.as_ptr();
        let length = syntax.program().ok_or("original Program")?.body.len();
        check(length == 4)?;
        check(syntax.comments().count() == 1)?;
        check(syntax.diagnostics().count() == 0)?;
        check(core::ptr::eq(syntax.source().authored_root(), source))?;
        check(core::ptr::eq(
            syntax.source().text(),
            script.block().source(),
        ))?;
        check(syntax.source().span() == script.block().span())?;
        let moved = complete(owner)?;
        let setup = moved.setup().map_err(|_| "sealed owning join")?;
        check(core::ptr::eq(setup.owner(), &moved))?;
        check(core::ptr::eq(
            setup.syntax(),
            moved.retained_setup().ok_or("retained after move")?,
        ))?;
        check(core::ptr::eq(
            setup.file(),
            moved.file().ok_or("actual File")?,
        ))?;
        check(core::ptr::eq(
            setup.unit_record(),
            setup.file().units().first().ok_or("actual unit")?,
        ))?;
        check(core::ptr::eq(setup.selected().owner(), moved.selected()))?;
        check(setup.unit() == unit && unit.index() as usize == script.container_index())?;
        check(setup.unit_record().span == script.block().span())?;
        check(setup.program().program().body.as_ptr() == body)?;
        check(setup.program().program().body.len() == length)?;
        check(
            setup.program().source_type().is_typescript()
                == (script.lang() == vize_l1::embed::Lang::Ts),
        )?;
        check(setup.file().bindings().count() == 4)?;
        for binding in setup.file().bindings() {
            let row = binding.declaration().ok_or("direct declaration")?;
            check(row.unit == unit && row.scope == setup.scope() && row.is_direct_program())?;
        }
    }
    Ok(())
}

#[test]
fn equal_sources_and_shared_buffer_owners_keep_distinct_whole_program_file_identity()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let original =
        std::string::String::from("<template></template><script setup>const value='🦀'</script>");
    let equal = original.clone();
    let mut first = owner(&arena, &original)?;
    let mut second = owner(&arena, &equal)?;
    let mut same_buffer = owner(&arena, &original)?;
    first.parse_setup_program().map_err(|_| "first")?;
    second.parse_setup_program().map_err(|_| "second")?;
    same_buffer
        .parse_setup_program()
        .map_err(|_| "same buffer")?;
    let first = complete(first)?;
    let second = complete(second)?;
    let same_buffer = complete(same_buffer)?;
    let a = first.setup().map_err(|_| "first receipt")?;
    for other in [
        second.setup().map_err(|_| "second receipt")?,
        same_buffer.setup().map_err(|_| "same buffer receipt")?,
    ] {
        check(a.program().source() == other.program().source())?;
        check(!core::ptr::eq(a.owner(), other.owner()))?;
        check(!core::ptr::eq(a.file(), other.file()))?;
        check(!core::ptr::eq(a.syntax(), other.syntax()))?;
        check(a.program().program().body.as_ptr() != other.program().program().body.as_ptr())?;
    }
    check(!core::ptr::eq(
        a.syntax().source().authored_root(),
        second
            .retained_setup()
            .ok_or("second syntax")?
            .source()
            .authored_root(),
    ))?;
    Ok(())
}
