use super::{Allocator, Kind, check, descriptor, equal, kind, owner, syntax};

#[test]
fn neutral_no_observer_program_completion_cannot_bypass_private_vue_refusals()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (source, expected) in [
        (
            "<script setup>const fn=1;const value=fn();</script><template>original</template>",
            Kind::UnsupportedInvocation,
        ),
        (
            "<script setup>const Ctor=1;const value=new Ctor();</script><template>original</template>",
            Kind::UnsupportedInvocation,
        ),
        (
            "<script setup>const tag=1;const value=tag`actual`;</script><template>original</template>",
            Kind::UnsupportedInvocation,
        ),
        (
            "<script setup>const value=import('actual');</script><template>original</template>",
            Kind::UnsupportedInvocation,
        ),
        (
            "<script setup>export const value=1;</script><template>original</template>",
            Kind::UnsupportedExport,
        ),
        (
            "<script setup>let __props=1;</script><template>original</template>",
            Kind::ReservedBinding,
        ),
        (
            "<script setup>let \\u005f\\u005fprops=1;</script><template>original</template>",
            Kind::ReservedBinding,
        ),
    ] {
        let observed = descriptor(&arena, source);
        let parsed = syntax(&arena, &observed, true)?;
        let mut original = owner(&arena, source)?;
        original
            .setup_program(parsed.admitted_program().ok_or("actual stock input")?)
            .map_err(|_| "same Program walk")?;
        equal(kind(original.begin())?, expected)?;
        let result = original.finish();
        check(result.view().is_err())?;
        let file = result.file().ok_or("actual neutral complete file")?;
        check(file.is_complete())?;
        equal(file.units().len(), 1)?;
        equal(file.artifact().node_count(), 0)?;
        check(core::ptr::eq(file.artifact().source(), source))?;
        equal(result.selected().children().len(), 1)?;
    }
    Ok(())
}

#[test]
fn absent_template_has_no_selected_owner_but_an_empty_template_does() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let absent = descriptor(&arena, "<script setup>const value=1;</script>");
    check(
        vize_l1::markup::NativeTemplateComponent::parse_in(
            &arena,
            absent.admitted().map_err(|_| "descriptor")?,
        )
        .map_err(|_| "component")?
        .is_none(),
    )?;
    let mut empty = owner(&arena, "<template></template>")?;
    empty
        .begin()
        .map_err(|_| "empty begin")?
        .complete()
        .map_err(|_| "empty complete")?;
    check(empty.finish().view().is_ok())?;
    Ok(())
}

#[test]
fn dropping_after_the_last_real_child_is_not_normal_completion() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let mut original = owner(&arena, "<template>actual</template>")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "child")?;
        }
        drop(walk);
    }
    let result = original.finish();
    check(result.view().is_err())?;
    let file = result.file().ok_or("partial file")?;
    equal(file.artifact().node_count(), 1)?;
    check(!file.is_complete())?;
    check(file.template_interruption().is_some())?;
    Ok(())
}

#[test]
fn authentic_nonmodule_jsx_and_definition_profiles_are_refused() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for source_type in [
        oxc_span::SourceType::mjs().with_script(true),
        oxc_span::SourceType::jsx(),
        oxc_span::SourceType::ts().with_typescript_definition(true),
    ] {
        let source = if source_type.is_typescript() {
            "<script setup lang='ts'>const value=1;</script><template></template>"
        } else {
            "<script setup>const value=1;</script><template></template>"
        };
        let observed = descriptor(&arena, source);
        let selection = observed
            .admitted()
            .map_err(|_| "descriptor")?
            .setup()
            .ok_or("script")?;
        let parsed =
            oxc_parser::Parser::new(arena.as_oxc(), selection.block().source(), source_type)
                .parse_observed();
        let mut original = owner(&arena, source)?;
        equal(
            kind(original.setup_program(parsed.admitted().ok_or("actual stock profile")?))?,
            Kind::InvalidProfile,
        )?;
        let result = original.finish();
        check(result.view().is_err())?;
        check(result.file().ok_or("file")?.units().is_empty())?;
    }
    Ok(())
}

#[test]
fn genuine_ts_selection_refuses_a_js_program_even_on_the_same_original_slice()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup lang='ts'>const value=1;</script><template></template>";
    let observed = descriptor(&arena, source);
    let selected = observed
        .admitted()
        .map_err(|_| "descriptor")?
        .setup()
        .ok_or("setup")?;
    let js = vize_l1::embed::syntax::parse_program_once(
        &arena,
        vize_l1::embed::EmbedSource::authored(source, selected.block().span())
            .map_err(|_| "source")?,
        vize_l1::embed::syntax::ProgramOptions::module(vize_l1::embed::Lang::Js),
    );
    let mut original = owner(&arena, source)?;
    equal(
        kind(original.setup_program(js.admitted_program().ok_or("real JS admission")?))?,
        Kind::InvalidProfile,
    )?;
    let result = original.finish();
    check(result.view().is_err())?;
    check(result.file().ok_or("file")?.units().is_empty())?;
    Ok(())
}

#[test]
fn original_empty_program_receipt_does_not_authenticate_a_foreign_empty_source()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup></script><template></template>";
    let observed = descriptor(&arena, source);
    let original_syntax = syntax(&arena, &observed, true)?;
    let copied = source.to_owned();
    let foreign_view = descriptor(&arena, &copied);
    let foreign = syntax(&arena, &foreign_view, true)?;
    check(
        original_syntax
            .program()
            .ok_or("original empty")?
            .body
            .is_empty(),
    )?;
    check(foreign.program().ok_or("foreign empty")?.body.is_empty())?;
    let mut rejected = owner(&arena, source)?;
    equal(
        kind(rejected.setup_program(foreign.admitted_program().ok_or("foreign admission")?))?,
        Kind::Program(vize_l2::file::FileIssueKind::InvalidSource),
    )?;
    check(rejected.finish().view().is_err())?;
    let mut positive = owner(&arena, source)?;
    positive
        .setup_program(
            original_syntax
                .admitted_program()
                .ok_or("original admission")?,
        )
        .map_err(|_| "original program")?;
    positive
        .begin()
        .map_err(|_| "empty root")?
        .complete()
        .map_err(|_| "normal end")?;
    let result = positive.finish();
    check(result.view().is_ok())?;
    equal(result.file().ok_or("file")?.units().len(), 1)?;
    Ok(())
}

#[test]
fn ordinary_setup_roles_are_intrinsic_and_duplicate_profile_semantics_are_refused()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>const first=1;</script><script setup>const second=2;</script><template>text</template>";
    let observed = descriptor(&arena, source);
    let ordinary = syntax(&arena, &observed, false)?;
    let setup = syntax(&arena, &observed, true)?;
    let mut original = owner(&arena, source)?;
    original
        .ordinary_program(ordinary.admitted_program().ok_or("ordinary")?)
        .map_err(|_| "actual ordinary")?;
    original
        .setup_program(setup.admitted_program().ok_or("setup")?)
        .map_err(|_| "actual setup")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    let file = original.finish();
    check(file.view().is_ok())?;
    equal(file.file().ok_or("file")?.bindings().count(), 2)?;
    let mut swapped = owner(&arena, source)?;
    equal(
        kind(swapped.setup_program(ordinary.admitted_program().ok_or("ordinary")?))?,
        Kind::Program(vize_l2::file::FileIssueKind::InvalidSource),
    )?;
    let mut duplicate = owner(&arena, source)?;
    duplicate
        .setup_program(setup.admitted_program().ok_or("setup")?)
        .map_err(|_| "setup")?;
    equal(
        kind(duplicate.setup_program(setup.admitted_program().ok_or("setup")?))?,
        Kind::DuplicateProgram,
    )?;
    check(duplicate.finish().view().is_err())?;
    Ok(())
}

#[test]
fn unresolved_program_and_original_element_entity_roots_stay_unadmitted() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source = "<script setup>const value=unknown;</script><template>text</template>";
    let observed = descriptor(&arena, source);
    let syntax = syntax(&arena, &observed, true)?;
    let mut original = owner(&arena, source)?;
    original
        .setup_program(syntax.admitted_program().ok_or("admission")?)
        .map_err(|_| "walk")?;
    equal(
        kind(original.begin())?,
        Kind::Program(vize_l2::file::FileIssueKind::UnsupportedSyntax),
    )?;
    check(original.finish().view().is_err())?;
    for source in ["<template><svg/></template>", "<template>&amp;</template>"] {
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = walk.selected().children().next().ok_or("original child")?;
            equal(kind(walk.child(child))?, Kind::UnsupportedChild)?;
            equal(kind(walk.complete())?, Kind::UnsupportedChild)?;
        }
        let result = original.finish();
        check(result.view().is_err())?;
        check(!result.file().ok_or("file")?.is_complete())?;
    }
    Ok(())
}
