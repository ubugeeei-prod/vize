use super::*;
use vize_l2::file::{DeclarationKind, InitializerKind};

#[test]
fn actual_strict_binding_names_and_escaped_aliases_refuse_all_declaration_kinds()
-> Result<(), String> {
    let arena = Allocator::default();
    let mut admitted = 0;
    for (keyword, kind) in [
        ("const", DeclarationKind::Const),
        ("let", DeclarationKind::Let),
        ("var", DeclarationKind::Var),
    ] {
        for (name, escaped) in [
            ("eval", r"\u0065val"),
            ("arguments", r"\u0061rguments"),
            ("implements", r"\u0069mplements"),
            ("interface", r"\u0069nterface"),
            ("let", r"\u006cet"),
            ("package", r"\u0070ackage"),
            ("private", r"\u0070rivate"),
            ("protected", r"\u0070rotected"),
            ("public", r"\u0070ublic"),
            ("static", r"\u0073tatic"),
            ("yield", r"\u0079ield"),
        ] {
            for spelling in [name, escaped] {
                let source = cstr!("<script setup>{keyword} {spelling}=1;</script>");
                let original = Observed::new(&arena, &source)?;
                let Some(program) = original.syntax.admitted_program() else {
                    require!(
                        original.syntax.diagnostics().count() > 0,
                        "{keyword} {spelling}"
                    );
                    continue;
                };
                admitted += 1;
                require!(!program.has_legacy_literals());
                let file = original.file(&arena)?;
                require!(file.is_complete(), "{keyword} {spelling}");
                require!(original.view(&file)?.is_ok());
                equal!(file.bindings().count(), 1);
                let declaration = file
                    .bindings()
                    .next()
                    .ok_or("strict binding")?
                    .declaration()
                    .ok_or("declaration")?;
                equal!(declaration.name.as_str(), name);
                equal!(declaration.kind, kind);
                equal!(declaration.initializer, InitializerKind::PrimitiveLiteral);
                require!(
                    matches!(checked(&original, &file)?, Err(issue)
                    if issue.kind == SetupIssueKind::UnsupportedSyntax),
                    "{keyword} {spelling}"
                );
                equal!(original.syntax.diagnostics().count(), 0);
            }
        }
    }
    require!(admitted > 0, "actual stock parser observations");
    Ok(())
}

#[test]
fn original_legacy_literal_receipts_refuse_const_let_and_var_without_changing_file_facts()
-> Result<(), String> {
    let arena = Allocator::default();
    for keyword in ["const", "let", "var"] {
        for literal in ["010", "08", "09.5", r"'\1'", r"'\8'", r"'\9'"] {
            let source = cstr!("<script setup>{keyword} value={literal};</script>");
            let original = Observed::new(&arena, &source)?;
            let program = original
                .syntax
                .admitted_program()
                .ok_or("legacy original admission")?;
            require!(program.has_legacy_literals(), "{keyword} {literal}");
            let file = original.file(&arena)?;
            require!(file.is_complete());
            require!(original.view(&file)?.is_ok());
            equal!(file.bindings().count(), 1);
            equal!(
                file.bindings()
                    .next()
                    .ok_or("actual binding")?
                    .declaration()
                    .ok_or("declaration")?
                    .initializer,
                InitializerKind::PrimitiveLiteral
            );
            require!(
                matches!(checked(&original, &file)?, Err(issue)
                if issue.kind == SetupIssueKind::UnsupportedSyntax),
                "{keyword} {literal}"
            );
            require!(core::ptr::eq(file.artifact().source(), source.as_str()));
            equal!(original.syntax.diagnostics().count(), 0);
        }
    }
    Ok(())
}

#[test]
fn legal_original_numeric_and_string_literals_keep_exact_owners_and_setup_admission()
-> Result<(), String> {
    let arena = Allocator::default();
    for (keyword, kind) in [
        ("const", DeclarationKind::Const),
        ("let", DeclarationKind::Let),
        ("var", DeclarationKind::Var),
    ] {
        for (literal, expected) in [
            ("0", InitializerKind::PrimitiveLiteral),
            ("0o10", InitializerKind::PrimitiveLiteral),
            ("0x10", InitializerKind::PrimitiveLiteral),
            ("0b10", InitializerKind::PrimitiveLiteral),
            (r"'\0'", InitializerKind::PrimitiveStringWithNulOrCr),
            (r"'\x01'", InitializerKind::PrimitiveLiteral),
            (r"'\u0001'", InitializerKind::PrimitiveLiteral),
            (r"'\\1'", InitializerKind::PrimitiveLiteral),
        ] {
            let source = cstr!("<script setup>/* kept */{keyword} value={literal};</script>");
            let original = Observed::new(&arena, &source)?;
            let program = original
                .syntax
                .admitted_program()
                .ok_or("valid original admission")?;
            require!(!program.has_legacy_literals(), "{keyword} {literal}");
            let file = original.file(&arena)?;
            let setup = checked(&original, &file)?.map_err(|_| "valid strict setup")?;
            require!(core::ptr::eq(setup.source().root_source(), source.as_str()));
            require!(core::ptr::eq(setup.exposure().file(), &file));
            require!(core::ptr::eq(
                setup.exposure().program().program(),
                program.program()
            ));
            let binding = setup.bindings().next().ok_or("genuine strict binding")?;
            require!(core::ptr::eq(binding.file(), &file));
            let declaration = binding.declaration().ok_or("genuine strict declaration")?;
            equal!(declaration.kind, kind);
            equal!(declaration.initializer, expected);
            equal!(declaration.scope, setup.exposure().scope());
            equal!(declaration.script_unit(), Some(setup.exposure().unit()));
            equal!(original.syntax.comments().count(), 1);
            equal!(original.syntax.diagnostics().count(), 0);
        }
    }
    Ok(())
}

#[test]
fn invalid_original_names_and_literals_stay_rejected_before_and_after_valid_declarations()
-> Result<(), String> {
    let arena = Allocator::default();
    for invalid in [
        "const eval=1;",
        r"let \u0061rguments=1;",
        "var value=010;",
        r"const value='\1';",
    ] {
        for body in [
            cstr!("{invalid}const after=1;"),
            cstr!("const before=1;{invalid}"),
            cstr!("const before=1;{invalid}const after=2;"),
        ] {
            let source = cstr!("<script setup>{body}</script>");
            let original = Observed::new(&arena, &source)?;
            let file = original.file(&arena)?;
            require!(file.is_complete());
            require!(original.view(&file)?.is_ok());
            require!(file.bindings().count() >= 2);
            require!(
                matches!(checked(&original, &file)?, Err(issue)
                if issue.kind == SetupIssueKind::UnsupportedSyntax),
                "{body}"
            );
            equal!(original.syntax.diagnostics().count(), 0);
        }
    }
    Ok(())
}

#[test]
fn literal_receipts_do_not_bypass_foreign_source_or_original_program_ownership()
-> Result<(), String> {
    let arena = Allocator::default();
    for literal in [r"'\1'", r"'\x01'"] {
        let source = cstr!("<script setup>const value={literal};</script>");
        let copy = source.clone();
        let original = Observed::new(&arena, &source)?;
        let foreign = Observed::new(&arena, &copy)?;
        let file = original.file(&arena)?;
        require!(matches!(checked(&foreign, &file)?, Err(issue)
            if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::Source)));
        let script = original.script()?;
        let second =
            Parser::new(&arena, script.block().source(), SourceType::mjs()).parse_observed();
        require!(
            matches!(VueSetup::checked(&file, script, second.admitted().ok_or("second original")?),
            Err(issue) if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::ProgramOrigin))
        );
    }
    Ok(())
}
