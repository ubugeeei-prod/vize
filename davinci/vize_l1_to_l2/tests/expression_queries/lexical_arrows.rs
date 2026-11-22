//! One-way construction of original JS/TS arrow scopes and their consuming tool queries.
use super::lexical_symbols as oracle;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{
    NativeSfcObservation, lower_sfc_with_lexical_initializers, lower_sfc_with_listener_references,
};
use vize_l2::{
    file::{DeclarationKind, FileArtifact, InitializerKind},
    lang::js::{FileProducer, ProgramInput, ProgramScope},
    query::Model,
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}

fn standalone<'a>(
    arena: &'a Allocator,
    source: &'a str,
    ty: SourceType,
    arrows: bool,
) -> Option<FileArtifact<'a>> {
    let parse = Parser::new(arena.as_oxc(), source, ty).parse_observed();
    let block = SourceRoot::new(source).ok()?.whole_block();
    let input = ProgramInput::checked(parse.admitted()?, block, 0).ok()?;
    let mut producer = FileProducer::new(arena, source).ok()?;
    producer
        .program(
            if arrows {
                input.with_lexical_arrow_initializers()
            } else {
                input
            },
            ProgramScope::Module,
        )
        .ok()?;
    let file = producer.finish().ok()?;
    file.is_complete().then_some(file)
}

fn count_arrows(file: &FileArtifact<'_>) -> usize {
    file.bindings()
        .filter_map(|b| b.declaration())
        .filter(|d| {
            d.initializer == InitializerKind::Function
                && matches!(
                    d.kind,
                    DeclarationKind::Const | DeclarationKind::Let | DeclarationKind::Var
                )
        })
        .count()
}

fn original_file<'f, 'a>(source: &'f NativeSfcObservation<'a>) -> Option<&'f FileArtifact<'a>> {
    source
        .file()
        .map(|f| f.file())
        .or_else(|| source.rejected_file()?.file())
}

#[test]
fn lexical_shadowing_closures_and_write_targets_match_oxc_and_definition_queries() {
    let source = "let n=1; const 外=4; const save=(n: number): number=>{const local=n; return local+外}; const inc=()=>{n+=1;return n}; const plain=()=>0;";
    let arena = Allocator::new();
    let ty = SourceType::ts().with_module(true);
    assert!(standalone(&arena, source, ty, false).is_none());
    let file = standalone(&arena, source, ty, true).unwrap();
    let block = SourceRoot::new(source).unwrap().whole_block();
    let expected = oracle::oxc(block, ty);
    assert_eq!(oracle::file(&file), expected);
    assert_eq!(count_arrows(&file), 3);
    for &(start, _, target, _, _, _) in &expected.references {
        let decl = Model::new(&file)
            .binding_at_offset(start)
            .unwrap()
            .unwrap()
            .declaration()
            .unwrap();
        assert_eq!(decl.span.start, target);
    }
    assert!(expected.references.iter().any(|r| r.3 && r.4 && r.5));
}

#[test]
fn the_template_handler_is_the_original_setup_declaration_and_keeps_its_captures() {
    let source = "<script setup lang='ts'>import Card from './Card.vue'; const 外=2; const save=(n:number)=>n+外;</script>\r\n<template><Card @save.once='save'/></template>";
    let arena = Allocator::new();
    assert!(
        lower_sfc_with_listener_references(&arena, source, options())
            .setup_component_imports()
            .is_err()
    );
    let observed = lower_sfc_with_lexical_initializers(&arena, source, options());
    let imports = observed.setup_component_imports().unwrap();
    let [site] = imports.sites() else {
        panic!("one component")
    };
    let mut events = 0;
    site.visit_static_listeners(|event| {
        assert_eq!(event.name, "save");
        assert_eq!(event.modifiers, &["once"]);
        let decl = event.handler_binding.unwrap().declaration().unwrap();
        assert_eq!(decl.span.start as usize, source.find("save=").unwrap());
        assert_eq!(decl.initializer, InitializerKind::Function);
        events += 1;
    })
    .unwrap();
    assert_eq!(events, 1);
    let block = observed.scripts()[0].block();
    let rows = oracle::oxc(block, SourceType::ts().with_module(true));
    assert_eq!(oracle::file(original_file(&observed).unwrap()), rows);
    assert_eq!(rows.references.iter().filter(|row| row.5).count(), 1);
}

#[test]
fn unsupported_parameter_scope_invocation_and_nested_forms_refuse_the_complete_file() {
    for init in [
        "async x=>x",
        "(x=1)=>x",
        "(...xs)=>xs",
        "({x})=>x",
        "<T>(x:T)=>x",
        "x=>()=>x",
        "x=>this",
        "()=>arguments",
        "(x:Missing)=>x",
        "()=>{if (1) return 1}",
    ] {
        let source = format!("const first=()=>0;const bad={init};");
        assert!(
            standalone(
                &Allocator::new(),
                &source,
                SourceType::ts().with_module(true),
                true
            )
            .is_none(),
            "{source}"
        );
    }
}

#[test]
#[ignore = "exhaustive original Vue and standalone JS/TS manifests, compared with OXC scoping"]
fn original_modules_and_component_files() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (key, vue) in [
        ("VIZE_EXPRESSION_QUERY_MANIFEST", true),
        ("VIZE_LEXICAL_MODULE_MANIFEST", false),
    ] {
        let list = std::fs::read_to_string(std::env::var_os(key).unwrap()).unwrap();
        let (
            mut total,
            mut spelling,
            mut valid,
            mut files,
            mut arrows,
            mut symbols,
            mut references,
            mut captures,
            mut listeners,
        ) = (0, 0, 0, 0, 0, 0, 0, 0, 0);
        for name in list.split_terminator('\0') {
            total += 1;
            let bytes = std::fs::read(root.join(name)).unwrap();
            let Ok(source) = std::str::from_utf8(&bytes) else {
                continue;
            };
            valid += 1;
            if !source.contains("=>") {
                continue;
            }
            spelling += 1;
            let arena = Allocator::new();
            let mut compare = |file: &FileArtifact<'_>, block, ty| {
                let count = count_arrows(file);
                if count == 0 {
                    return;
                }
                let actual = oracle::file(file);
                let expected = oracle::oxc(block, ty);
                assert_eq!(actual, expected, "{name}");
                files += 1;
                arrows += count;
                symbols += expected.declarations.len();
                references += expected.references.len();
                captures += expected.references.iter().filter(|r| r.5).count();
            };
            if vue {
                let obs = lower_sfc_with_lexical_initializers(&arena, source, options());
                let Ok(imports) = obs.setup_component_imports() else {
                    continue;
                };
                let file = original_file(&obs).unwrap();
                if count_arrows(file) == 0 {
                    continue;
                }
                let parsed = &obs.scripts()[0];
                let ty = parsed
                    .syntax()
                    .unwrap()
                    .admitted_program()
                    .unwrap()
                    .source_type();
                compare(file, parsed.block(), ty);
                let (_, events) = super::component_listeners::compare(&imports);
                listeners += events;
            } else {
                let ty = if name.ends_with(".ts") || name.ends_with(".mts") {
                    SourceType::ts()
                } else {
                    SourceType::mjs()
                }
                .with_module(true);
                let Some(file) = standalone(&arena, source, ty, true) else {
                    continue;
                };
                compare(&file, SourceRoot::new(source).unwrap().whole_block(), ty);
            }
        }
        eprintln!(
            "lexical_arrows vue={vue} total={total} valid={valid} candidates={spelling} admitted_with_arrows={files} arrows={arrows} symbols={symbols} refs={references} captures={captures} component_listeners={listeners}"
        );
        if std::env::var_os("VIZE_REQUIRE_REPOSITORY_ARROWS").is_some() {
            assert!(files > 0)
        }
    }
}
