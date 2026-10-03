use super::*;
use oxc_parser::ParseOptions;
use vize_l1::embed::Lang;
use vize_l2::file::{DeclarationKind, InitializerKind, Namespace};

#[test]
fn original_ts_module_primitives_keep_profile_body_source_kind_and_declaration_order()
-> Result<(), String> {
    let arena = Allocator::default();
    let source = r"<script setup lang=ts>/* 雪 */;const \u0063ount=1, yes=true;let nil=null, large=2n;var 日本語='🌸';;</script>";
    let original = Observed::new(&arena, source)?;
    let file = original.file(&arena)?;
    let setup = checked(&original, &file)?.map_err(|_| "genuine TS setup")?;
    let exposure = setup.exposure();
    let program = exposure.program();
    require!(file.is_complete());
    equal!(exposure.script().lang(), Lang::Ts);
    require!(program.source_type().is_typescript());
    require!(program.source_type().is_module());
    require!(!program.source_type().is_unambiguous());
    require!(!program.source_type().is_typescript_definition());
    require!(!program.source_type().is_jsx());
    equal!(program.options(), ParseOptions::default());
    require!(core::ptr::eq(
        program.program(),
        original.syntax.program().ok_or("Program")?
    ));
    require!(core::ptr::eq(exposure.file(), &file));
    require!(core::ptr::eq(setup.source().root_source(), source));
    require!(core::ptr::eq(
        setup.source().source(),
        original.script()?.block().source()
    ));
    equal!(original.syntax.comments().count(), 1);
    equal!(original.syntax.diagnostics().count(), 0);
    equal!(setup.bindings().count(), 5);
    for (binding, (name, kind)) in setup.bindings().zip([
        ("count", DeclarationKind::Const),
        ("yes", DeclarationKind::Const),
        ("nil", DeclarationKind::Let),
        ("large", DeclarationKind::Let),
        ("日本語", DeclarationKind::Var),
    ]) {
        require!(core::ptr::eq(binding.file(), &file));
        let declaration = binding.declaration().ok_or("actual TS binding")?;
        equal!(declaration.name.as_str(), name);
        equal!(declaration.kind, kind);
        equal!(declaration.initializer, InitializerKind::PrimitiveLiteral);
        equal!(declaration.namespace, Namespace::Value);
        equal!(declaration.scope, exposure.scope());
        equal!(declaration.script_unit(), Some(exposure.unit()));
        require!(declaration.is_direct_program());
    }
    let empty = Observed::new(&arena, "<script setup lang=ts>/* kept */;;;</script>")?;
    let empty_file = empty.file(&arena)?;
    equal!(
        checked(&empty, &empty_file)?
            .map_err(|_| "empty TS statements")?
            .bindings()
            .count(),
        0
    );
    Ok(())
}

#[test]
fn ts_setup_rejects_type_bearing_and_other_nonprimitive_syntax_without_erasure()
-> Result<(), String> {
    let arena = Allocator::default();
    for body in [
        "const value:number=1;",
        "let value!:number;",
        "declare const value:number;",
        "const value=1 as number;",
        "const value=1 as const;",
        "const value=1 satisfies number;",
        "const value=<number>1;",
        "const value=1!;",
        "type Name=number;const value=1;",
        "interface Name {value:number} const value=1;",
        "enum Name {Value=1} const value=1;",
        "const enum Name {Value=1} const value=1;",
        "namespace Name {export const value=1;}const visible=1;",
        "module Name {export const value=1;}const visible=1;",
        "import type {Name} from 'types';const value=1;",
        "import value=require('module');",
        "export type Name=number;const value=1;",
        "export const value=1;",
        "export = 1;",
        "function hidden(value:number):number{return value;}const visible=1;",
        "class Hidden {value:number=1;}const visible=1;",
        "using value=1;",
        "await using value=1;",
        "const {value}={value:1};",
        "const [value]=[1];",
        "const value={};",
        "const value=[];",
        "const value=-1;",
        "const value=`text`;",
        "const value=1;value=2;",
        "const value=1;42;",
        "function hidden(){}const value=1;",
        "'use strict';const value=1;",
        "#!/usr/bin/env node\nconst value=1;",
    ] {
        let source = cstr!("<script setup lang=ts>{body}</script>");
        let original = Observed::new(&arena, &source)?;
        require!(
            original.syntax.admitted_program().is_some(),
            "actual TS parse: {body}"
        );
        let file = original.file(&arena)?;
        require!(checked(&original, &file)?.is_err(), "{body}");
        require!(core::ptr::eq(file.artifact().source(), source.as_str()));
        equal!(original.syntax.diagnostics().count(), 0);
    }
    Ok(())
}

#[test]
fn ts_setup_retains_foreign_source_and_second_original_program_refusals() -> Result<(), String> {
    let arena = Allocator::default();
    let source = String::from("<script setup lang=ts>const value=1;</script>");
    let copy = source.clone();
    let original = Observed::new(&arena, &source)?;
    let foreign = Observed::new(&arena, &copy)?;
    let file = original.file(&arena)?;
    require!(matches!(checked(&foreign, &file)?, Err(issue)
        if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::Source)));
    let script = original.script()?;
    let second = Parser::new(
        &arena,
        script.block().source(),
        SourceType::ts().with_module(true),
    )
    .parse_observed();
    require!(matches!(
        VueSetup::checked(&file, script, second.admitted().ok_or("second TS admission")?),
        Err(issue) if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::ProgramOrigin)
    ));
    require!(checked(&original, &file)?.is_ok());
    Ok(())
}

#[test]
fn ts_setup_requires_original_module_language_and_default_parser_profile() -> Result<(), String> {
    let arena = Allocator::default();
    let original = Observed::new(&arena, "<script setup lang=ts>const value=1;</script>")?;
    let file = original.file(&arena)?;
    let script = original.script()?;
    for profile in [
        SourceType::mjs(),
        SourceType::ts().with_script(true),
        SourceType::ts(),
        SourceType::tsx().with_module(true),
        SourceType::d_ts(),
    ] {
        let wrong = Parser::new(&arena, script.block().source(), profile).parse_observed();
        require!(matches!(
            VueSetup::checked(&file, script, wrong.admitted().ok_or("profile admission")?),
            Err(issue) if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::Profile)
        ));
    }
    for options in [
        ParseOptions {
            preserve_parens: false,
            ..ParseOptions::default()
        },
        ParseOptions {
            enable_ident_hashes: false,
            ..ParseOptions::default()
        },
        ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        },
        ParseOptions {
            allow_v8_intrinsics: true,
            ..ParseOptions::default()
        },
    ] {
        let wrong = Parser::new(
            &arena,
            script.block().source(),
            SourceType::ts().with_module(true),
        )
        .with_options(options)
        .parse_observed();
        let admitted = wrong.admitted().ok_or("nondefault TS admission")?;
        equal!(admitted.options(), options);
        require!(matches!(
            ProgramInput::checked(admitted, script.block(), script.container_index()),
            Err(issue) if issue.kind == FileIssueKind::InvalidProfile
        ));
        require!(matches!(
            VueSetup::checked(&file, script, wrong.admitted().ok_or("nondefault TS admission")?),
            Err(issue) if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::Profile)
        ));
    }
    require!(checked(&original, &file)?.is_ok());
    Ok(())
}

#[test]
fn every_ts_setup_callback_interruption_preserves_original_owners_and_refuses_sealing()
-> Result<(), String> {
    let arena = Allocator::default();
    let original = Observed::new(
        &arena,
        "<script setup lang=ts>/* kept */const first=1;let mutable=2;var after=3;</script>",
    )?;
    let script = original.script()?;
    for point in [Point::Unit, Point::Statement, Point::Declared] {
        let mut producer =
            FileProducer::new(&arena, original.descriptor.source()).map_err(|_| "file")?;
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer
                .program_observed(
                    ProgramInput::checked(
                        original.syntax.admitted_program().ok_or("TS Program")?,
                        script.block(),
                        script.container_index(),
                    )
                    .map_err(|_| "input")?,
                    ProgramScope::Nested,
                    &mut Interrupt(point),
                )
                .map_err(|_| "unit")
        }));
        let payload = interrupted.err().ok_or("actual TS interruption")?;
        equal!(
            payload.downcast_ref::<&str>(),
            Some(&"actual setup walk interruption")
        );
        let file = producer.finish().map_err(|_| "artifact")?;
        require!(!file.is_complete(), "{point:?}");
        require!(file.issues().is_empty());
        equal!(file.interrupted_programs().count(), 1);
        equal!(
            file.bindings().count(),
            usize::from(point == Point::Declared)
        );
        let unit = file.units().first().ok_or("retained TS unit")?;
        require!(unit.profile.typescript);
        equal!(
            unit.interruption().ok_or("TS interruption")?.kind,
            FileIssueKind::InterruptedProgram
        );
        require!(matches!(checked(&original, &file)?, Err(issue)
            if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::IncompleteFile)));
        require!(core::ptr::eq(
            file.artifact().source(),
            original.descriptor.source()
        ));
        equal!(original.syntax.comments().count(), 1);
        equal!(original.syntax.diagnostics().count(), 0);
    }
    let completed = original.file(&arena)?;
    require!(checked(&original, &completed)?.is_ok());
    equal!(completed.bindings().count(), 3);
    Ok(())
}

#[test]
fn ts_setup_keeps_strict_original_literal_and_binding_refusals() -> Result<(), String> {
    let arena = Allocator::default();
    for kind in ["const", "let", "var"] {
        for declaration in [
            "eval=1",
            "arguments=1",
            "public=1",
            r"\u0065val=1",
            "value=010",
            "value=08",
            r"value='\1'",
            r"value='\8'",
        ] {
            let source = cstr!("<script setup lang=ts>{kind} {declaration};</script>");
            let original = Observed::new(&arena, &source)?;
            let file = original.file(&arena)?;
            require!(file.is_complete());
            require!(
                matches!(checked(&original, &file)?, Err(issue) if issue.kind == SetupIssueKind::UnsupportedSyntax),
                "{kind} {declaration}"
            );
            require!(original.syntax.admitted_program().is_some());
            equal!(original.syntax.diagnostics().count(), 0);
        }
    }
    for literal in ["0o10", "0x10", r"'\0'", r"'\x01'", r"'\u0001'", r"'\\1'"] {
        let source = cstr!("<script setup lang=ts>const value={literal};</script>");
        let original = Observed::new(&arena, &source)?;
        let file = original.file(&arena)?;
        require!(
            checked(&original, &file)?.is_ok(),
            "strict-safe TS: {literal}"
        );
    }
    Ok(())
}
