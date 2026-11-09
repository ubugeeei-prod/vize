use super::{Allocator, Kind, check, descriptor, equal, kind, owner, syntax};
use vize_l1::markup::NativeTemplateGrammar;
use vize_l2::file::FileIssueKind;

#[test]
fn original_empty_default_js_and_ts_reach_the_selected_template_with_zero_bindings()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (content, ts, comments) in [
        ("export default {};", false, 0),
        (
            "'use strict'; ; export /*間*/ default /*before*/ { /*inside*/ }; ;",
            false,
            3,
        ),
        (
            "\r\n/*before*/'雪';\r\nexport default {}\r\n/*tail*/",
            true,
            2,
        ),
    ] {
        let profile = if ts { " lang='ts'" } else { "" };
        let source = format!(
            "<script{profile}>{content}</script><template>first<!--second-->third</template>"
        );
        let observed = descriptor(&arena, &source);
        let parsed = syntax(&arena, &observed, false)?;
        let original = parsed.program().ok_or("original Program")?;
        let mut original_owner = owner(&arena, &source)?;
        let unit = original_owner
            .ordinary_program(parsed.admitted_program().ok_or("ordinary admission")?)
            .map_err(|_| "original ordinary")?;
        {
            let mut walk = original_owner
                .begin()
                .map_err(|_| "empty default preflight")?;
            let selected = walk.selected();
            equal(
                selected.grammar(),
                if ts {
                    NativeTemplateGrammar::TypeScriptModule
                } else {
                    NativeTemplateGrammar::JavaScriptModule
                },
            )?;
            for child in selected.children() {
                walk.child(child).map_err(|_| "actual child")?;
            }
            walk.complete().map_err(|_| "normal end")?;
        }
        let result = core::hint::black_box(original_owner.finish());
        let file = result
            .view()
            .map_err(|_| "native complete")?
            .file()
            .ok_or("file")?;
        check(file.is_complete())?;
        equal(file.units().len(), 1)?;
        equal(file.units()[0].id, unit)?;
        check(file.units()[0].interruption().is_none())?;
        equal(file.units()[0].profile.typescript, ts)?;
        equal(file.bindings().count(), 0)?;
        check(file.imports().is_empty())?;
        check(file.references().is_empty())?;
        equal(file.exports().len(), 1)?;
        equal(file.exports()[0].name.as_str(), "default")?;
        equal(file.exports()[0].unit, unit)?;
        check(file.ordinary_empty_script().is_some())?;
        equal(file.artifact().node_count(), 3)?;
        check(core::ptr::eq(file.artifact().source(), source.as_str()))?;
        check(core::ptr::eq(file, result.file().ok_or("same owner File")?))?;
        check(core::ptr::eq(
            parsed.program().ok_or("retained Program")?,
            original,
        ))?;
        equal(parsed.comments().count(), comments)?;
        equal(parsed.diagnostics().count(), 0)?;
    }
    Ok(())
}

#[test]
fn selected_empty_default_exception_does_not_admit_other_export_families()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (content, ts, expected) in [
        (
            "export default {name: 'options'};",
            false,
            Kind::UnsupportedExport,
        ),
        ("export default ({});", false, Kind::UnsupportedExport),
        (
            "export default {} as const;",
            true,
            Kind::Program(FileIssueKind::UnsupportedSyntax),
        ),
        (
            "export default {} satisfies object;",
            true,
            Kind::Program(FileIssueKind::UnsupportedSyntax),
        ),
        (
            "import 'dep'; export default {};",
            false,
            Kind::UnsupportedExport,
        ),
        (
            "export default {}; export default {};",
            true,
            Kind::UnsupportedExport,
        ),
        (
            "export default {}; export {};",
            false,
            Kind::UnsupportedExport,
        ),
        (
            "; 'post'; export default {};",
            false,
            Kind::UnsupportedExport,
        ),
        (
            "const local=1; export default {};",
            false,
            Kind::UnsupportedExport,
        ),
    ] {
        let profile = if ts { " lang='ts'" } else { "" };
        let source = format!("<script{profile}>{content}</script><template>kept</template>");
        let observed = descriptor(&arena, &source);
        let parsed = syntax(&arena, &observed, false)?;
        equal(parsed.diagnostics().count(), 0)?;
        let mut original = owner(&arena, &source)?;
        original
            .ordinary_program(
                parsed
                    .admitted_program()
                    .ok_or("actual ordinary admission")?,
            )
            .map_err(|_| "ordinary walk")?;
        equal(kind(original.begin())?, expected)?;
        let result = original.finish();
        check(result.view().is_err())?;
        let file = result.file().ok_or("retained neutral File")?;
        equal(file.units().len(), 1)?;
        check(file.units()[0].interruption().is_none())?;
        check(file.ordinary_empty_script().is_none())?;
        equal(file.artifact().node_count(), 0)?;
        check(core::ptr::eq(file.artifact().source(), source.as_str()))?;
        equal(result.selected().children().len(), 1)?;
    }
    Ok(())
}

#[test]
fn even_an_empty_setup_unit_keeps_ordinary_default_export_refused() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<script>export default {};</script><script setup></script><template>kept</template>";
    let observed = descriptor(&arena, source);
    let ordinary = syntax(&arena, &observed, false)?;
    let setup = syntax(&arena, &observed, true)?;
    let mut original = owner(&arena, source)?;
    original
        .ordinary_program(ordinary.admitted_program().ok_or("ordinary")?)
        .map_err(|_| "ordinary walk")?;
    original
        .setup_program(setup.admitted_program().ok_or("empty setup")?)
        .map_err(|_| "setup walk")?;
    equal(kind(original.begin())?, Kind::UnsupportedExport)?;
    let result = original.finish();
    check(result.view().is_err())?;
    let file = result.file().ok_or("retained File")?;
    check(file.is_complete())?;
    equal(file.units().len(), 2)?;
    equal(file.bindings().count(), 0)?;
    check(file.ordinary_empty_script().is_none())?;
    equal(file.artifact().node_count(), 0)?;
    Ok(())
}

#[test]
fn ordinary_no_export_route_retains_its_existing_behavior() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>const local=1;</script><template>kept</template>";
    let observed = descriptor(&arena, source);
    let parsed = syntax(&arena, &observed, false)?;
    let mut original = owner(&arena, source)?;
    original
        .ordinary_program(parsed.admitted_program().ok_or("ordinary")?)
        .map_err(|_| "ordinary walk")?;
    {
        let mut walk = original
            .begin()
            .map_err(|_| "existing no-export preflight")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    let result = original.finish();
    check(result.view().is_ok())?;
    let file = result.file().ok_or("File")?;
    equal(file.bindings().count(), 1)?;
    check(file.exports().is_empty())?;
    check(file.ordinary_empty_script().is_none())?;
    equal(file.artifact().node_count(), 1)?;
    Ok(())
}

#[test]
fn empty_default_does_not_turn_dropped_template_prefixes_into_completion()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>/*original*/ export default {};</script><template>first<!--second-->third</template>";
    let observed = descriptor(&arena, source);
    let parsed = syntax(&arena, &observed, false)?;
    for stop in [0, 1, 3] {
        let mut original = owner(&arena, source)?;
        original
            .ordinary_program(parsed.admitted_program().ok_or("ordinary")?)
            .map_err(|_| "ordinary walk")?;
        {
            let mut walk = original.begin().map_err(|_| "preflight")?;
            for child in walk.selected().children().take(stop) {
                walk.child(child).map_err(|_| "prefix child")?;
            }
            drop(walk);
        }
        equal(kind(original.begin())?, Kind::Interrupted)?;
        let result = original.finish();
        check(result.view().is_err())?;
        let file = result.file().ok_or("partial File")?;
        check(!file.is_complete())?;
        check(file.template_interruption().is_some())?;
        check(file.ordinary_empty_script().is_none())?;
        equal(file.units().len(), 1)?;
        check(file.units()[0].interruption().is_none())?;
        equal(file.bindings().count(), 0)?;
        equal(file.exports().len(), 1)?;
        equal(file.artifact().node_count(), stop as u32)?;
        check(core::ptr::eq(file.artifact().source(), source))?;
        equal(parsed.comments().count(), 1)?;
    }
    Ok(())
}

#[test]
fn copied_equal_ordinary_source_cannot_establish_a_selected_program() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>export default {};</script><template>kept</template>";
    let copied = source.to_owned();
    let foreign_descriptor = descriptor(&arena, &copied);
    let foreign = syntax(&arena, &foreign_descriptor, false)?;
    let mut original = owner(&arena, source)?;
    equal(
        kind(original.ordinary_program(foreign.admitted_program().ok_or("foreign")?))?,
        Kind::Program(FileIssueKind::InvalidSource),
    )?;
    let result = original.finish();
    check(result.view().is_err())?;
    let file = result.file().ok_or("original File")?;
    check(file.units().is_empty())?;
    equal(file.artifact().node_count(), 0)?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    Ok(())
}
