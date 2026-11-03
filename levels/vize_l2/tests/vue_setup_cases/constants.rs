use super::*;
use vize_l2::file::{DeclarationKind, InitializerKind};

#[test]
fn genuine_const_primitives_keep_original_declaration_kinds_and_order() -> Result<(), String> {
    let arena = Allocator::default();
    let source = "<script setup>/* 雪 */const count=1, yes=true, nil=null, large=2n, text='🌸';let mutable=0;var previous=0;</script>";
    let original = Observed::new(&arena, source)?;
    let file = original.file(&arena)?;
    let setup = checked(&original, &file)?.map_err(|_| "genuine const setup")?;
    require!(file.is_complete());
    require!(core::ptr::eq(setup.source().root_source(), source));
    equal!(setup.bindings().count(), 7);
    equal!(original.syntax.comments().count(), 1);
    for (binding, (name, kind)) in setup.bindings().zip([
        ("count", DeclarationKind::Const),
        ("yes", DeclarationKind::Const),
        ("nil", DeclarationKind::Const),
        ("large", DeclarationKind::Const),
        ("text", DeclarationKind::Const),
        ("mutable", DeclarationKind::Let),
        ("previous", DeclarationKind::Var),
    ]) {
        require!(core::ptr::eq(binding.file(), &file));
        let declaration = binding.declaration().ok_or("actual binding")?;
        equal!(declaration.name.as_str(), name);
        equal!(declaration.kind, kind);
        equal!(declaration.initializer, InitializerKind::PrimitiveLiteral);
        equal!(declaration.scope, setup.exposure().scope());
        equal!(declaration.script_unit(), Some(setup.exposure().unit()));
        require!(declaration.is_direct_program());
    }
    Ok(())
}

#[test]
fn const_capability_still_requires_original_source_and_program_owners() -> Result<(), String> {
    let arena = Allocator::default();
    let source = String::from("<script setup>const value=1;</script>");
    let copy = source.clone();
    let original = Observed::new(&arena, &source)?;
    let foreign = Observed::new(&arena, &copy)?;
    let file = original.file(&arena)?;
    require!(matches!(checked(&foreign, &file)?, Err(issue)
        if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::Source)));
    let script = original.script()?;
    let second = Parser::new(&arena, script.block().source(), SourceType::mjs()).parse_observed();
    require!(matches!(
        VueSetup::checked(&file, script, second.admitted().ok_or("second admission")?),
        Err(issue) if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::ProgramOrigin)
    ));
    require!(checked(&original, &file)?.is_ok());
    Ok(())
}
