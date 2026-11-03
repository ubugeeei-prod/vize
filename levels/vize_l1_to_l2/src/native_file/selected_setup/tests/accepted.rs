use super::*;

#[test]
fn whole_js_ts_descriptor_program_and_completed_file_remain_one_moved_owner()
-> Result<(), &'static str> {
    for source in [
        "<!--🦀 outside--><script setup>/*whole*/ const rows='雪'; let count=1;</script><template>{{ rows }}<i/></template>",
        "<template>{{ rows }}<i/></template><script setup lang='ts'>/*whole*/ const rows:string='雪'; let count:number=1;</script><!--outside-->",
    ] {
        let arena = Allocator::default();
        let owner =
            core::hint::black_box(lower_selected_setup_sfc_native(&arena, source, options()));
        check(owner.original().issues().is_empty())?;
        let descriptor = owner
            .original()
            .descriptor()
            .admitted()
            .map_err(|_| "whole descriptor")?;
        let script = descriptor.setup().ok_or("original script")?;
        let template = descriptor.template().ok_or("original template")?;
        check(owner.original().descriptor().container().blocks.len() == 2)?;
        check(core::ptr::eq(
            owner.original().descriptor().source(),
            source,
        ))?;
        check(owner.original().admitted().is_none())?;
        let view = owner.admitted().ok_or("whole original setup envelope")?;
        let setup = view.setup();
        check(core::ptr::eq(view.observation(), &owner))?;
        check(core::ptr::eq(
            setup.owner(),
            owner.original().template().ok_or("real native File")?,
        ))?;
        check(core::ptr::eq(
            setup.program().source(),
            script.block().source(),
        ))?;
        check(core::ptr::eq(
            setup.syntax().source().authored_root(),
            source,
        ))?;
        check(setup.syntax().comments().count() == 1 && setup.syntax().diagnostics().count() == 0)?;
        check(setup.program().program().body.len() == 2)?;
        check(setup.unit().index() as usize == script.container_index())?;
        check(setup.unit_record().span == script.block().span())?;
        check(setup.file().bindings().count() == 2)?;
        check(setup.bindings().count() == 2)?;
        check(setup.type_annotations().count() == if script.lang() == Lang::Ts { 2 } else { 0 })?;
        for annotation in setup.type_annotations() {
            check(core::ptr::eq(annotation.file(), setup.file()))?;
            check(
                annotation.span().start >= script.block().span().start
                    && annotation.span().end <= script.block().span().end,
            )?;
        }
        check(setup.file().native_interpolations().len() == 1)?;
        let original_file = setup.file();
        let original_body = setup.program().program().body.as_ptr();
        let native = view.into_template_view();
        check(core::ptr::eq(
            native.file().ok_or("same complete File")?,
            original_file,
        ))?;
        check(native.owner().selected().template_index() == template.container_index())?;
        check(native.owner().selected().component().block().span() == template.block().span())?;
        check(
            native
                .owner()
                .retained_setup()
                .ok_or("parked whole stock owner")?
                .program()
                .ok_or("actual Program")?
                .body
                .as_ptr()
                == original_body,
        )?;
    }
    Ok(())
}

#[test]
fn same_file_setup_declarations_reach_original_for_alias_and_handler_resolution()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><i v-for='item in rows'/><button @click='rows'/></template><script setup>const rows='ab'</script>";
    let owner = lower_selected_setup_sfc_native(&arena, source, options());
    check(owner.original().issues().is_empty())?;
    let view = owner.admitted().ok_or("whole setup owner")?;
    let setup = view.setup();
    let binding = setup
        .file()
        .bindings()
        .find(|binding| {
            binding
                .declaration()
                .is_some_and(|declaration| declaration.name.as_str() == "rows")
        })
        .ok_or("actual selected setup declaration")?;
    check(binding.declaration().ok_or("same declaration")?.unit == setup.unit())?;
    let alias = setup
        .file()
        .template_declarations()
        .next()
        .ok_or("original For alias")?;
    check(core::ptr::eq(alias.file(), setup.file()))?;
    check(alias.declaration().original().is_some())?;
    check(setup.bindings().count() == 1)?;
    let actual_alias = setup
        .file()
        .binding(alias.declaration().id())
        .ok_or("actual File alias row")?;
    check(setup.binding(actual_alias).is_err())?;
    let Some(vize_l2::op::Op::Element(button)) = setup.file().artifact().root().ops.last() else {
        return Err("original button");
    };
    let [vize_l2::op::BindingOp::On(on)] = button.bindings.as_slice() else {
        return Err("original On");
    };
    let handler = setup.file().handler_for(on).ok_or("same File actual On")?;
    check(handler.accepts(binding))?;
    check(handler.scope() == Some(setup.scope()))?;
    let resolution = handler.resolution().ok_or("whole handler resolution")?;
    let [reference] = resolution.references() else {
        return Err("original outer read");
    };
    check(reference.binding == vize_l2::resolution::HandlerBindingRef::Outer(binding.id()))?;
    check(setup.file().units().len() == 1)?;
    Ok(())
}

#[test]
fn equal_sources_and_shared_source_keep_distinct_normal_whole_owners() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let first_source = vize_l0::String::from(
        "<template>{{ 1 }}</template><script setup>const value='雪'</script>",
    );
    let equal_source = first_source.clone();
    let a = lower_selected_setup_sfc_native(&arena, &first_source, options());
    let b = lower_selected_setup_sfc_native(&arena, &equal_source, options());
    let c = lower_selected_setup_sfc_native(&arena, &first_source, options());
    let first = a.admitted().ok_or("first")?;
    for other in [
        b.admitted().ok_or("equal bytes")?,
        c.admitted().ok_or("shared bytes")?,
    ] {
        check(!core::ptr::eq(first.observation(), other.observation()))?;
        check(!core::ptr::eq(
            first.setup().syntax(),
            other.setup().syntax(),
        ))?;
        let foreign_binding = other
            .setup()
            .bindings()
            .next()
            .ok_or("actual foreign setup row")?;
        check(
            matches!(first.setup().binding(foreign_binding), Err(issue) if issue.kind == vize_l2::file::vue::ExposureIssueKind::ForeignBinding),
        )?;
        check(!core::ptr::eq(first.setup().file(), other.setup().file()))?;
        check(first.setup().program().source() == other.setup().program().source())?;
        check(
            first.setup().program().program().body.as_ptr()
                != other.setup().program().program().body.as_ptr(),
        )?;
    }
    Ok(())
}
