use super::*;
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::{NativeSetupIssueKind as SetupKind, NativeTemplateIssueKind as Kind};

#[test]
fn borrowed_program_cannot_mint_owning_receipt_or_reparse_selected_slot() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source = "<template></template><script setup>const value=1</script>";
    let descriptor = descriptor(&arena, source);
    let script = descriptor
        .admitted()
        .map_err(|_| "descriptor")?
        .setup()
        .ok_or("setup")?;
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, script.block().span()).map_err(|_| "source")?,
        ProgramOptions::module(script.lang()),
    );
    let mut original = owner(&arena, source)?;
    original
        .setup_program(syntax.admitted_program().ok_or("stock Program")?)
        .map_err(|_| "borrowed route")?;
    let completed = complete(original)?;
    check(completed.retained_setup().is_none())?;
    check(matches!(completed.setup(), Err(issue) if issue.kind == SetupKind::MissingOwner))?;
    let mut borrowed = owner(&arena, source)?;
    borrowed
        .setup_program(syntax.admitted_program().ok_or("stock Program")?)
        .map_err(|_| "borrowed route")?;
    check(
        matches!(borrowed.parse_setup_program(), Err(issue) if issue.kind == Kind::DuplicateProgram),
    )?;
    check(borrowed.retained_setup().is_none())?;
    check(borrowed.begin().is_err())?;
    Ok(())
}

#[test]
fn owning_route_refuses_duplicate_and_borrowed_mix_without_replacing_stock_program()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template></template><script setup>/*whole*/ const value=1</script>";
    for mix_borrowed in [false, true] {
        let descriptor = descriptor(&arena, source);
        let script = descriptor
            .admitted()
            .map_err(|_| "descriptor")?
            .setup()
            .ok_or("setup")?;
        let foreign = parse_program_once(
            &arena,
            EmbedSource::authored(source, script.block().span()).map_err(|_| "source")?,
            ProgramOptions::module(script.lang()),
        );
        let mut original = owner(&arena, source)?;
        original.parse_setup_program().map_err(|_| "owning route")?;
        let body = original
            .retained_setup()
            .ok_or("whole Program")?
            .program()
            .ok_or("Program")?
            .body
            .as_ptr();
        let result = if mix_borrowed {
            original.setup_program(foreign.admitted_program().ok_or("foreign parse")?)
        } else {
            original.parse_setup_program()
        };
        check(matches!(result, Err(issue) if issue.kind == Kind::DuplicateProgram))?;
        check(
            original
                .retained_setup()
                .ok_or("still whole")?
                .program()
                .ok_or("Program")?
                .body
                .as_ptr()
                == body,
        )?;
        check(original.begin().is_err())?;
        let rejected = original.finish();
        check(
            rejected
                .retained_setup()
                .ok_or("finished custody")?
                .comments()
                .count()
                == 1,
        )?;
        check(rejected.view().is_err() && rejected.setup().is_err())?;
    }
    Ok(())
}

#[test]
fn syntax_semantic_and_whole_setup_refusals_preserve_complete_original_observation()
-> Result<(), &'static str> {
    for content in [
        "/*whole*/ const =",
        "/*whole*/ const value=[1]",
        "/*whole*/ const value={x:1}",
        "/*whole*/ const value=run()",
        "/*whole*/ export const value=1",
        "/*whole*/ const __expose=1",
        "/*whole*/ const value=/x/",
        "/*whole*/ const value=-1",
        "/*whole*/ const value=010",
        "/*whole*/ function value(){}",
        "/*whole*/ const value:string[]=[]",
        "/*whole*/",
        "",
        "'use strict'; const value=1",
    ] {
        let arena = Allocator::default();
        let source = std::format!("<template></template><script setup lang=ts>{content}</script>");
        let mut original = owner(&arena, &source)?;
        let result = original.parse_setup_program();
        let syntax = original.retained_setup().ok_or("whole refused owner")?;
        check(syntax.source().text() == content)?;
        check(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str(),
        ))?;
        if content.contains("/*whole*/") {
            check(syntax.comments().count() == 1)?;
        }
        if content == "/*whole*/ const =" {
            check(matches!(result, Err(issue) if matches!(issue.kind, Kind::SetupSyntax(_))))?;
            check(syntax.diagnostics().count() != 0 && syntax.program().is_none())?;
        }
        let body = syntax.program().map(|program| program.body.as_ptr());
        {
            if let Ok(walk) = original.begin() {
                let _ = walk.complete();
            }
        }
        let finished = original.finish();
        check(finished.setup().is_err())?;
        check(
            finished
                .retained_setup()
                .ok_or("finished stock custody")?
                .source()
                .text()
                == content,
        )?;
        check(
            finished
                .retained_setup()
                .ok_or("finished stock custody")?
                .program()
                .map(|program| program.body.as_ptr())
                == body,
        )?;
    }
    Ok(())
}

#[test]
fn ordinary_style_and_missing_setup_refuse_before_parse_and_descriptor_profiles_stay_closed()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (source, expected) in [
        ("<template></template>", Kind::MissingProgram),
        (
            "<template></template><script>const ordinary=1</script><script setup>const value=1</script>",
            Kind::UnsupportedOrdinaryScript,
        ),
        (
            "<template></template><script setup>const value=1</script><style>.x{}</style>",
            Kind::UnsupportedStyle,
        ),
    ] {
        let mut original = owner(&arena, source)?;
        check(matches!(original.parse_setup_program(), Err(issue) if issue.kind == expected))?;
        check(original.retained_setup().is_none())?;
        check(original.finish().setup().is_err())?;
    }
    for source in [
        "<template></template><script setup src='external.js'></script>",
        "<template></template><script setup lang=tsx>const x=1</script>",
        "<template></template><script setup lang='t&#115;'>const x=1</script>",
        "<template></template><script setup='true'>const x=1</script>",
        "<template></template><script setup>const x=1</script><script setup>const y=2</script>",
        "<template></template><custom>whole</custom><script setup>const x=1</script>",
    ] {
        let descriptor = descriptor(&arena, source);
        check(descriptor.admitted().is_err())?;
        check(core::ptr::eq(descriptor.source(), source))?;
        check(!descriptor.issues().is_empty())?;
    }
    Ok(())
}
