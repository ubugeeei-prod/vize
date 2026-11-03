use super::*;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_l2::file::FileIssueKind;

#[test]
fn foreign_bytes_second_program_and_foreign_same_numeric_unit_refuse() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = String::from("<script>export default {};</script>");
    let copy = source.clone();
    let original = Observed::new(&arena, &source)?;
    let foreign = Observed::new(&arena, &copy)?;
    let file = original.file(&arena)?;
    equal(kind(foreign.checked(&file)?)?, OrdinaryIssueKind::Source)?;
    let script = original.script()?;
    let second =
        Parser::new(arena.as_oxc(), script.block().source(), SourceType::mjs()).parse_observed();
    equal(
        kind(VueOrdinaryEmpty::checked(
            &file,
            script,
            second.admitted().ok_or("second admission")?,
        ))?,
        OrdinaryIssueKind::ProgramOrigin,
    )?;
    let second_original = Observed::new(&arena, &source)?;
    let second_file = second_original.file(&arena)?;
    equal(
        second_file.units().first().ok_or("second unit")?.id,
        file.units().first().ok_or("original unit")?.id,
    )?;
    equal(
        kind(original.checked(&second_file)?)?,
        OrdinaryIssueKind::ProgramOrigin,
    )?;
    check(original.checked(&file)?.is_ok())?;
    Ok(())
}

#[test]
fn actual_roles_nested_scope_and_extra_units_cannot_supply_the_root_family()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let setup = Observed::new(&arena, "<script setup>export default {};</script>")?;
    let file = setup.file(&arena)?;
    equal(kind(setup.checked(&file)?)?, OrdinaryIssueKind::Role)?;
    check(file.ordinary_empty_script().is_none())?;
    let original = Observed::new(&arena, "<script>export default {};</script>")?;
    let script = original.script()?;
    let input = || {
        ProgramInput::checked(
            original.syntax.admitted_program().ok_or("Program")?,
            script.block(),
            script.container_index(),
        )
        .map_err(|_| "input")
    };
    let mut nested = FileProducer::new(&arena, original.descriptor.source()).map_err(|_| "file")?;
    nested
        .program(input()?, ProgramScope::Nested)
        .map_err(|_| "walk")?;
    check(nested.ordinary_empty_script().is_none())?;
    let nested = nested.finish().map_err(|_| "file")?;
    check(nested.is_complete())?;
    equal(kind(original.checked(&nested)?)?, OrdinaryIssueKind::Scope)?;
    let mut extra = original.producer(&arena)?;
    extra
        .program(
            ProgramInput::checked(
                original.syntax.admitted_program().ok_or("Program")?,
                script.block(),
                script.container_index() + 1,
            )
            .map_err(|_| "input")?,
            ProgramScope::Module,
        )
        .map_err(|_| "extra walk")?;
    check(extra.ordinary_empty_script().is_none())?;
    let extra = extra.finish().map_err(|_| "file")?;
    check(extra.is_complete())?;
    equal(extra.exports().len(), 2)?;
    equal(
        kind(original.checked(&extra)?)?,
        OrdinaryIssueKind::MissingUnit,
    )?;
    Ok(())
}

#[test]
fn original_js_ts_profile_and_default_options_are_required_before_ast_shape()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<script>export default {};</script>",
        "<script lang='ts'>export default {};</script>",
    ] {
        let original = Observed::new(&arena, source)?;
        let file = original.file(&arena)?;
        let script = original.script()?;
        let requested = if script.lang() == vize_l1::embed::Lang::Ts {
            SourceType::ts()
        } else {
            SourceType::mjs()
        };
        for profile in [
            SourceType::jsx(),
            SourceType::ts().with_typescript_definition(true),
            SourceType::unambiguous(),
            if requested.is_typescript() {
                SourceType::mjs()
            } else {
                SourceType::ts()
            },
        ] {
            let parser =
                Parser::new(arena.as_oxc(), script.block().source(), profile).parse_observed();
            let admitted = parser.admitted().ok_or("original alternate profile")?;
            equal(
                kind(VueOrdinaryEmpty::checked(&file, script, admitted))?,
                OrdinaryIssueKind::Profile,
            )?;
        }
        let options = ParseOptions {
            preserve_parens: false,
            ..ParseOptions::default()
        };
        let parser = Parser::new(arena.as_oxc(), script.block().source(), requested)
            .with_options(options)
            .parse_observed();
        equal(
            kind(VueOrdinaryEmpty::checked(
                &file,
                script,
                parser.admitted().ok_or("nondefault original")?,
            ))?,
            OrdinaryIssueKind::Profile,
        )?;
        equal(
            ProgramInput::checked(
                parser.admitted().ok_or("nondefault original")?,
                script.block(),
                script.container_index(),
            )
            .err()
            .ok_or("input refusal")?
            .kind,
            FileIssueKind::InvalidProfile,
        )?;
        check(original.checked(&file)?.is_ok())?;
    }
    Ok(())
}
