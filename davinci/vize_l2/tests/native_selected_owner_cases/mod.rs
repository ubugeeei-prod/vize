use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    embed::{
        EmbedSource,
        syntax::{NativeSyntax, ProgramOptions, parse_program_once},
    },
    markup::{NativeTemplateComponent, NativeTemplateGrammar},
};
use vize_l2::lang::js::{
    NativeTemplateIssue, NativeTemplateIssueKind as Kind, NativeTemplateOwner,
};

fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required condition")
    }
}
fn equal<T: PartialEq>(actual: T, expected: T) -> Result<(), &'static str> {
    check(actual == expected)
}
fn descriptor<'a>(arena: &'a Allocator, source: &'a str) -> DescriptorObservation<'a> {
    Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}
fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, &'static str> {
    NativeTemplateComponent::parse_in(
        arena,
        descriptor(arena, source)
            .admitted()
            .map_err(|_| "descriptor")?,
    )
    .map_err(|_| "component")?
    .ok_or("template")
}
fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    NativeTemplateOwner::new(selected(arena, source)?).map_err(|_| "file start")
}
fn kind<T>(result: Result<T, NativeTemplateIssue>) -> Result<Kind, &'static str> {
    match result {
        Err(issue) => Ok(issue.kind),
        Ok(_) => Err("genuine refusal"),
    }
}
fn syntax<'a>(
    arena: &'a Allocator,
    view: &DescriptorObservation<'a>,
    setup: bool,
) -> Result<NativeSyntax<'a>, &'static str> {
    let admitted = view.admitted().map_err(|_| "descriptor")?;
    let script = if setup {
        admitted.setup()
    } else {
        admitted.ordinary()
    }
    .ok_or("script")?;
    Ok(parse_program_once(
        arena,
        EmbedSource::authored(view.source(), script.block().span()).map_err(|_| "source")?,
        ProgramOptions::module(script.lang()),
    ))
}

#[test]
fn original_text_comment_order_file_identity_and_moved_owner() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>hé<!--original-->tail</template>";
    let mut owner = owner(&arena, source)?;
    {
        let mut walk = owner.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        equal(selected.grammar(), NativeTemplateGrammar::JavaScriptModule)?;
        for child in selected.children() {
            walk.child(child).map_err(|_| "child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    let result = core::hint::black_box(owner.finish());
    let view = result.view().map_err(|_| "view")?;
    let file = view.file().ok_or("actual file")?;
    check(file.is_complete())?;
    equal(file.artifact().node_count(), 3)?;
    check(file.units().is_empty())?;
    check(core::ptr::eq(file, result.file().ok_or("owner file")?))?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    equal(result.selected().children().len(), 3)?;
    Ok(())
}

#[test]
fn empty_selected_owner_never_uses_empty_backing_or_movable_root_identity()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template></template>";
    let mut original = owner(&arena, source)?;
    let foreign = owner(&arena, source)?.finish();
    check(foreign.file().ok_or("neutral file")?.is_complete())?;
    check(foreign.view().is_err())?;
    original
        .begin()
        .map_err(|_| "empty begin")?
        .complete()
        .map_err(|_| "empty complete")?;
    let moved = core::hint::black_box(original.finish());
    let view = moved.view().map_err(|_| "native empty")?;
    equal(view.file().ok_or("file")?.artifact().node_count(), 0)?;
    check(view.file().ok_or("file")?.units().is_empty())?;
    check(foreign.view().is_err())?;
    Ok(())
}

#[test]
fn skipped_duplicated_and_reordered_original_events_remain_sticky() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>first<!--second-->third</template>";
    for mode in 0..3 {
        let mut owner = owner(&arena, source)?;
        {
            let mut walk = owner.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            if mode == 0 {
                equal(kind(walk.complete())?, Kind::IncompleteChildren)?;
            } else if mode == 1 {
                walk.child(selected.children().next().ok_or("first")?)
                    .map_err(|_| "first mint")?;
                equal(
                    kind(walk.child(selected.children().next().ok_or("duplicate")?))?,
                    Kind::InvalidEvent,
                )?;
                equal(kind(walk.complete())?, Kind::InvalidEvent)?;
            } else {
                let child = selected.children().nth(1).ok_or("second")?;
                equal(kind(walk.child(child))?, Kind::InvalidEvent)?;
                equal(kind(walk.complete())?, Kind::InvalidEvent)?;
            }
        }
        let result = owner.finish();
        check(result.view().is_err())?;
        let file = result.file().ok_or("partial file")?;
        check(!file.is_complete())?;
        equal(file.artifact().node_count(), if mode == 1 { 1 } else { 0 })?;
        check(file.template_interruption().is_some())?;
        check(!file.template_issues().is_empty())?;
    }
    Ok(())
}

#[test]
fn equal_original_bytes_and_second_actual_parse_cannot_supply_a_foreign_child()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>original</template>";
    let copied = source.to_owned();
    for foreign_source in [source, copied.as_str()] {
        let foreign = selected(&arena, foreign_source)?;
        let mut owner = owner(&arena, source)?;
        {
            let mut walk = owner.begin().map_err(|_| "begin")?;
            equal(
                kind(walk.child(foreign.children().next().ok_or("foreign")?))?,
                Kind::InvalidEvent,
            )?;
            equal(kind(walk.complete())?, Kind::InvalidEvent)?;
        }
        let result = owner.finish();
        check(result.view().is_err())?;
        let file = result.file().ok_or("file")?;
        check(!file.is_complete())?;
        equal(file.artifact().node_count(), 0)?;
        check(core::ptr::eq(file.artifact().source(), source))?;
    }
    Ok(())
}

#[test]
fn actual_panic_before_first_between_and_after_last_child_retains_partial_owners()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>first<!--second-->third</template>";
    for stop in [0, 1, 3] {
        let mut owner = owner(&arena, source)?;
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut walk = owner.begin().unwrap();
            let selected = walk.selected();
            for child in selected.children().take(stop) {
                walk.child(child).unwrap();
            }
            std::panic::resume_unwind(Box::new("genuine root cursor interruption"));
        }));
        check(panic.is_err())?;
        let result = core::hint::black_box(owner.finish());
        check(result.view().is_err())?;
        equal(result.selected().children().len(), 3)?;
        let file = result.file().ok_or("partial file")?;
        check(!file.is_complete())?;
        equal(file.artifact().node_count(), stop as u32)?;
        check(file.template_interruption().is_some())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
    }
    Ok(())
}

#[test]
fn forgotten_whole_guard_and_caught_restart_never_seal_native_completion()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let mut owner = owner(&arena, "<template></template>")?;
    std::mem::forget(owner.begin().map_err(|_| "begin")?);
    equal(kind(owner.begin())?, Kind::Interrupted)?;
    let result = owner.finish();
    check(result.view().is_err())?;
    check(!result.file().ok_or("pending file")?.is_complete())?;
    Ok(())
}

#[test]
fn intrinsic_ts_setup_receipt_is_from_the_actual_selected_program() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<script setup lang='ts'>/*original*/ const value = 1;</script><template>text</template>";
    let observed = descriptor(&arena, source);
    let syntax = syntax(&arena, &observed, true)?;
    let original = syntax.program().ok_or("program")?;
    let mut owner = owner(&arena, source)?;
    let unit = owner
        .setup_program(syntax.admitted_program().ok_or("admitted")?)
        .map_err(|_| "actual setup")?;
    {
        let mut walk = owner.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        equal(selected.grammar(), NativeTemplateGrammar::TypeScriptModule)?;
        for child in selected.children() {
            walk.child(child).map_err(|_| "child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    let result = owner.finish();
    let file = result
        .view()
        .map_err(|_| "native view")?
        .file()
        .ok_or("file")?;
    check(file.is_complete())?;
    let [actual] = file.units() else {
        return Err("single unit");
    };
    equal(actual.id, unit)?;
    check(actual.profile.typescript)?;
    check(actual.profile.module)?;
    check(!actual.profile.jsx)?;
    check(core::ptr::eq(syntax.program().ok_or("retained")?, original))?;
    equal(syntax.comments().count(), 1)?;
    Ok(())
}

#[test]
fn missing_and_foreign_source_programs_cannot_bypass_selected_receipts() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source = "<script setup>let value=1;</script><template>text</template>";
    let mut missing = owner(&arena, source)?;
    equal(kind(missing.begin())?, Kind::MissingProgram)?;
    let result = missing.finish();
    check(result.view().is_err())?;
    check(result.file().ok_or("neutral")?.is_complete())?;
    let copy = source.to_owned();
    let foreign_view = descriptor(&arena, &copy);
    let foreign = syntax(&arena, &foreign_view, true)?;
    let mut original = owner(&arena, source)?;
    equal(
        kind(original.setup_program(foreign.admitted_program().ok_or("foreign admission")?))?,
        Kind::Program(vize_l2::file::FileIssueKind::InvalidSource),
    )?;
    let result = original.finish();
    check(result.view().is_err())?;
    check(result.file().ok_or("file")?.units().is_empty())?;
    Ok(())
}

mod element;
mod rejection;
mod text;
