//! Actual original Program admission, File mint, projection and checker positions.

#[path = "program_projection/checker.rs"]
mod checker;
#[path = "program_projection/support.rs"]
mod support;

use vize_l0::{Allocator, SourceRoot, Span, String, cstr};
use vize_l1::embed::syntax::{ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l2::file::{FileBuilder, FileIssueKind, Namespace};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l4::targets::ts::{
    MappingError, ProjectionError, SourceKind, project_program, project_program_no_links,
};

#[test]
fn admitted_whole_modules_preserve_exact_bytes_kind_owner_links_and_bindings() {
    for (source, lang, kind) in [
        ("", Lang::Js, SourceKind::JavaScript),
        (
            "#!/usr/bin/env node\nconst value = 1; // tail",
            Lang::Js,
            SourceKind::JavaScript,
        ),
        (
            "\u{feff}/* 😀 */ const 日本語 = 'é';\r\n日本語.length",
            Lang::Ts,
            SourceKind::TypeScript,
        ),
        (
            "import type {T} from './missing';\n",
            Lang::Ts,
            SourceKind::TypeScript,
        ),
        (
            "/*😀*/ const 日本語: /*🌸*/ number = 1; 日本語;",
            Lang::Ts,
            SourceKind::TypeScript,
        ),
    ] {
        let arena = Allocator::default();
        let file = support::file(&arena, source, lang).unwrap();
        assert!(file.is_complete(), "{source}: {:?}", file.issues());
        let projection = project_program(&file).unwrap();
        let plain = project_program_no_links(&file).unwrap();
        assert!(core::ptr::eq(projection.file(), &file));
        assert!(core::ptr::eq(
            projection.unit(),
            file.units().first().unwrap()
        ));
        assert_eq!(projection.source_kind(), kind);
        assert_eq!(
            projection.document().as_str(),
            cstr!("{source}\n;\nexport {{}};\n")
        );
        assert_eq!(plain.document().as_str(), projection.document().as_str());
        assert!(!plain.document().is_recording());
        assert!(plain.document().links().is_empty());
        assert_eq!(
            plain.map_span(Span::new(0, 0)),
            Err(MappingError::Unrecorded)
        );
        let [link] = projection.document().links() else {
            panic!("one complete source link")
        };
        assert_eq!(link.generated, support::whole(source).unwrap());
        assert_eq!(link.authored, support::whole(source).unwrap());
        assert!(link.name.is_none());
        assert_eq!(projection.map_span(link.generated), Ok(link.authored));
        let map: serde_json::Value =
            serde_json::from_str(&projection.document().source_map("Input.ts", source)).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(map["mappings"], "AAAA");
        if source.contains("日本語") {
            let binding = file
                .lookup(projection.unit().scope, "日本語", Namespace::Value)
                .unwrap();
            assert!(core::ptr::eq(binding.file(), projection.file()));
        }
    }
    assert_eq!(SourceKind::JavaScript.extension(), "mjs");
    assert_eq!(SourceKind::TypeScript.extension(), "ts");
}

#[test]
fn diagnostic_mapping_never_clamps_utf16_or_generated_boundaries() {
    let arena = Allocator::default();
    let source = "/*😀*/ const 日本語 = 'é';";
    let file = support::file(&arena, source, Lang::Js).unwrap();
    let projection = project_program(&file).unwrap();
    assert_eq!(projection.map_utf16(2, 2), Ok(Span::new(2, 6)));
    assert_eq!(
        projection.map_utf16(3, 0),
        Err(MappingError::InvalidUtf16Boundary)
    );
    assert_eq!(
        projection.map_utf16(u32::MAX, 1),
        Err(MappingError::InvalidUtf16Boundary)
    );
    let authored_end = u32::try_from(source.encode_utf16().count()).unwrap();
    let byte_end = u32::try_from(source.len()).unwrap();
    assert_eq!(
        projection.map_utf16(authored_end, 0),
        Ok(Span::new(byte_end, byte_end))
    );
    assert_eq!(
        projection.map_utf16(authored_end, 1),
        Err(MappingError::GeneratedOnly)
    );
    assert_eq!(
        projection.map_utf16(authored_end - 1, 2),
        Err(MappingError::CrossesBoundary)
    );
    assert_eq!(
        projection.map_span(Span::new(3, 3)),
        Err(MappingError::InvalidRange)
    );
    assert_eq!(
        projection.map_span(Span::new(7, 6)),
        Err(MappingError::InvalidRange)
    );
    assert_eq!(
        projection.map_span(Span::new(u32::MAX, u32::MAX)),
        Err(MappingError::InvalidRange)
    );
}

#[test]
fn neutral_empty_files_and_partial_or_multiple_real_units_cannot_be_projected() {
    let arena = Allocator::default();
    let empty = FileBuilder::new(&arena, "").unwrap().finish().unwrap();
    assert!(empty.is_complete());
    assert!(matches!(
        project_program(&empty),
        Err(ProjectionError::ProgramCount)
    ));
    let source = "const a=1; const b=2;";
    let split = source.find(" const").unwrap() + 1;
    let root = SourceRoot::new(source).unwrap();
    let first = root.block(source.get(..split).unwrap(), 0).unwrap();
    let second = root
        .block(source.get(split..).unwrap(), split as u32)
        .unwrap();
    let a = support::syntax(&arena, first, Lang::Js).unwrap();
    let b = support::syntax(&arena, second, Lang::Js).unwrap();
    let mut partial = FileProducer::new(&arena, source).unwrap();
    partial
        .program(
            ProgramInput::checked(a.admitted_program().unwrap(), first, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let partial = partial.finish().unwrap();
    assert!(partial.is_complete());
    assert!(matches!(
        project_program(&partial),
        Err(ProjectionError::PartialSource)
    ));
    let mut multi = FileProducer::new(&arena, source).unwrap();
    multi
        .program(
            ProgramInput::checked(a.admitted_program().unwrap(), first, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    multi
        .program(
            ProgramInput::checked(b.admitted_program().unwrap(), second, 1).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let multi = multi.finish().unwrap();
    assert!(multi.is_complete());
    assert!(matches!(
        project_program(&multi),
        Err(ProjectionError::ProgramCount)
    ));
}

#[test]
fn incomplete_profiles_and_supported_parser_recovery_are_never_promoted() {
    let arena = Allocator::default();
    for (source, lang) in [
        ("const value: number | string = 1;", Lang::Ts),
        ("class C {}", Lang::Js),
    ] {
        let file = support::file(&arena, source, lang).unwrap();
        assert!(!file.is_complete());
        assert!(matches!(
            project_program(&file),
            Err(ProjectionError::IncompleteFile)
        ));
    }
    let source = "const value=1;";
    let block = SourceRoot::new(source).unwrap().whole_block();
    for (options, complete, expected) in [
        (
            ProgramOptions {
                jsx: true,
                ..ProgramOptions::module(Lang::Ts)
            },
            true,
            ProjectionError::UnsupportedProfile,
        ),
        (
            ProgramOptions {
                goal: vize_l1::embed::syntax::ProgramGoal::Script,
                ..ProgramOptions::module(Lang::Js)
            },
            false,
            ProjectionError::IncompleteFile,
        ),
    ] {
        let syntax = parse_program_once(
            &arena,
            EmbedSource::authored(source, support::whole(source).unwrap()).unwrap(),
            options,
        );
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert_eq!(file.is_complete(), complete);
        assert!(matches!(project_program(&file), Err(actual) if actual == expected));
    }
    let source = "const value=/x/uv;";
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, support::whole(source).unwrap()).unwrap(),
        ProgramOptions::module(Lang::Js),
    );
    assert!(syntax.admitted_program().is_none());
    assert!(syntax.diagnostics().count() > 0);
}

#[test]
fn equal_copy_source_is_rejected_before_file_unit_mint() {
    let arena = Allocator::default();
    let source = String::from("const value=1;");
    let copy = source.clone();
    let block = SourceRoot::new(&source).unwrap().whole_block();
    let syntax = support::syntax(&arena, block, Lang::Js).unwrap();
    let foreign = SourceRoot::new(&copy).unwrap().whole_block();
    assert!(
        matches!(ProgramInput::checked(syntax.admitted_program().unwrap(), foreign, 0), Err(issue) if issue.kind == FileIssueKind::InvalidSource)
    );
    let file = support::file(&arena, &source, Lang::Js).unwrap();
    let projection = project_program(&file).unwrap();
    assert!(core::ptr::eq(projection.file(), &file));
    assert_eq!(
        projection.file().artifact().source().as_ptr(),
        source.as_ptr()
    );
}

#[test]
fn authentic_nested_units_and_mixed_template_nodes_remain_outside_program_projection() {
    use vize_l2::artifact::ComponentFactory;
    use vize_l2::file::{Declaration, TemplatePolicy, TemplateScope};
    #[derive(Clone, Copy)]
    struct NoBindings;
    impl TemplatePolicy for NoBindings {
        fn visible(self, _declaration: &Declaration) -> bool {
            false
        }
    }
    let arena = Allocator::default();
    let source = "const value=1;";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let syntax = support::syntax(&arena, block, Lang::Js).unwrap();
    let input = || ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap();
    let mut nested = FileProducer::new(&arena, source).unwrap();
    nested.program(input(), ProgramScope::Nested).unwrap();
    let nested = nested.finish().unwrap();
    assert!(nested.is_complete());
    assert!(matches!(
        project_program(&nested),
        Err(ProjectionError::NestedScope)
    ));
    let mut mixed = FileProducer::new(&arena, source).unwrap();
    mixed.program(input(), ProgramScope::Module).unwrap();
    mixed
        .template_region(TemplateScope::Root, NoBindings)
        .unwrap()
        .text(source, block.span())
        .unwrap();
    let mixed = mixed.finish().unwrap();
    assert!(mixed.is_complete());
    assert!(matches!(
        project_program(&mixed),
        Err(ProjectionError::TemplateOperations)
    ));
}
