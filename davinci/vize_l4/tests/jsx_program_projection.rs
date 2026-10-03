//! Genuine original JSX/TSX owners lend complete checker bytes and exact maps.

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, cstr};
use vize_l2::lang::js::{JsxFile, JsxFileError, JsxFileProducer};
use vize_l4::targets::ts::{
    MappingError, ProjectionError, SourceKind, project_jsx_program, project_jsx_program_no_links,
    project_program, project_program_no_links,
};

type LawResult = Result<(), &'static str>;

fn lower<'a>(
    arena: &'a Allocator,
    block: SourceBlock<'a>,
    profile: SourceType,
) -> Result<JsxFile<'a>, &'static str> {
    let observation = Parser::new(arena, block.source(), profile).parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, observation, block, 17).map_err(|_| "admission")?;
    producer.walk().map_err(|_| "walk")?;
    producer.finish().map_err(|_| "complete owner")
}

#[test]
fn whole_original_jsx_and_tsx_keep_kind_owner_comments_full_source_and_maps() -> LawResult {
    for (source, profile, kind) in [
        (
            "/*😀*/ import C from './dep'; const 日本語 = 'é'; export function render() { return <C title={日本語}>hello &amp; <span>{日本語}</span></C>; } // tail",
            SourceType::jsx(),
            SourceKind::Jsx,
        ),
        (
            "/*🌸*/ const 日本語: number = 1;\r\nexport function render() { return <div>{日本語}</div>; }",
            SourceType::tsx().with_module(true),
            SourceKind::Tsx,
        ),
        (
            "#!/usr/bin/env node\n'use client'; const view = <><div/>kept &amp; text</>;",
            SourceType::jsx(),
            SourceKind::Jsx,
        ),
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
        let owner = lower(&arena, block, profile)?;
        let original_body = owner
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr();
        let comment_count = owner.observation().comments().len();
        // Moving the genuine owner preserves its original observation and File.
        let mut moved = vec![owner];
        moved.reserve(256);
        let owner = moved.pop().ok_or("moved owner")?;
        let projection = project_jsx_program(&owner).map_err(|_| "projection")?;
        let plain = project_jsx_program_no_links(&owner).map_err(|_| "plain")?;
        assert!(core::ptr::eq(projection.file(), owner.file()));
        assert!(core::ptr::eq(
            projection.unit(),
            owner.file().units().first().ok_or("unit")?
        ));
        assert!(projection.unit().profile.jsx);
        assert_eq!(projection.source_kind(), kind);
        assert_eq!(
            projection.document().as_str(),
            cstr!("{source}\n;\nexport {{}};\n")
        );
        assert_eq!(plain.document().as_str(), projection.document().as_str());
        assert_eq!(plain.source_kind(), kind);
        assert!(!plain.document().is_recording());
        assert!(plain.document().links().is_empty());
        assert_eq!(
            plain.map_span(Span::new(0, 0)),
            Err(MappingError::Unrecorded)
        );
        assert_eq!(plain.map_utf16(0, 0), Err(MappingError::Unrecorded));
        let end = u32::try_from(source.len()).map_err(|_| "length")?;
        let [link] = projection.document().links() else {
            return Err("one complete authored link");
        };
        assert_eq!(link.generated, Span::new(0, end));
        assert_eq!(link.authored, Span::new(0, end));
        assert!(link.name.is_none());
        assert_eq!(projection.map_span(link.generated), Ok(link.authored));
        let filename = cstr!("Original.雪.{}", kind.extension());
        let map: serde_json::Value =
            serde_json::from_str(&projection.document().source_map(&filename, source))
                .map_err(|_| "map")?;
        assert_eq!(
            map,
            serde_json::json!({
                "version": 3,
                "file": filename.as_str(),
                "sources": [filename.as_str()],
                "sourcesContent": [source],
                "names": [],
                "mappings": "AAAA",
            })
        );
        assert_eq!(
            owner
                .observation()
                .admitted()
                .ok_or("retained")?
                .program()
                .body
                .as_ptr(),
            original_body
        );
        assert_eq!(owner.observation().comments().len(), comment_count);
        assert!(matches!(
            project_program(owner.file()),
            Err(ProjectionError::UnsupportedProfile)
        ));
        assert!(matches!(
            project_program_no_links(owner.file()),
            Err(ProjectionError::UnsupportedProfile)
        ));
    }
    assert_eq!(SourceKind::Jsx.extension(), "jsx");
    assert_eq!(SourceKind::Tsx.extension(), "tsx");
    assert_eq!(SourceKind::JavaScript.extension(), "mjs");
    assert_eq!(SourceKind::TypeScript.extension(), "ts");
    Ok(())
}

#[test]
fn original_jsx_utf16_ranges_refuse_surrogates_overflow_and_generated_suffix() -> LawResult {
    let arena = Allocator::default();
    let source = "/*😀*/ const 日本語 = 'é';\r\nconst view = <div>{日本語}</div>;";
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let owner = lower(&arena, block, SourceType::jsx())?;
    let projection = project_jsx_program(&owner).map_err(|_| "projection")?;
    assert_eq!(projection.map_utf16(2, 2), Ok(Span::new(2, 6)));
    assert_eq!(
        projection.map_utf16(3, 0),
        Err(MappingError::InvalidUtf16Boundary)
    );
    assert_eq!(
        projection.map_utf16(u32::MAX, 1),
        Err(MappingError::InvalidUtf16Boundary)
    );
    let start = source.rfind("日本語").ok_or("binding read")?;
    let start_utf16 = u32::try_from(source.get(..start).ok_or("prefix")?.encode_utf16().count())
        .map_err(|_| "offset")?;
    let byte_start = u32::try_from(start).map_err(|_| "byte offset")?;
    assert_eq!(
        projection.map_utf16(start_utf16, 3),
        Ok(Span::new(byte_start, byte_start + 9))
    );
    let authored_end = u32::try_from(source.encode_utf16().count()).map_err(|_| "utf16 length")?;
    let byte_end = u32::try_from(source.len()).map_err(|_| "length")?;
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
    Ok(())
}

#[test]
fn genuine_partial_jsx_owners_cannot_project_unparsed_root_bytes() -> LawResult {
    let arena = Allocator::default();
    let source = "const prefix = 0;\nconst view = <div/>;";
    let root = SourceRoot::new(source).map_err(|_| "source")?;
    let offset = source.find("const view").ok_or("block offset")?;
    let block = root
        .block(
            source.get(offset..).ok_or("block source")?,
            u32::try_from(offset).map_err(|_| "block offset")?,
        )
        .map_err(|_| "block")?;
    let owner = lower(&arena, block, SourceType::jsx())?;
    assert!(owner.file().is_complete());
    assert_eq!(owner.file().artifact().source(), source);
    assert!(matches!(
        project_jsx_program(&owner),
        Err(ProjectionError::PartialSource)
    ));
    assert!(matches!(
        project_jsx_program_no_links(&owner),
        Err(ProjectionError::PartialSource)
    ));
    Ok(())
}

#[test]
fn rejected_original_profiles_holes_and_incomplete_files_never_supply_jsx_owners() -> LawResult {
    let arena = Allocator::default();
    for (source, profile, expected) in [
        (
            "const value = 1;",
            SourceType::mjs(),
            JsxFileError::InvalidProfile,
        ),
        (
            "const view = <div/>;",
            SourceType::jsx().with_script(true),
            JsxFileError::InvalidProfile,
        ),
        (
            "/*kept*/ const view = <div>;",
            SourceType::jsx(),
            JsxFileError::SyntaxAdmission,
        ),
    ] {
        let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
        let observation = Parser::new(&arena, source, profile).parse_observed();
        let rejected = match JsxFileProducer::new(&arena, observation, block, 0) {
            Err(rejected) => rejected,
            Ok(_) => return Err("rejected profile minted producer"),
        };
        assert_eq!(rejected.error(), expected);
    }
    let source = "/*kept*/ const view = <Missing/>;";
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let observation = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
    let mut producer =
        JsxFileProducer::new(&arena, observation, block, 0).map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    let rejected = match producer.finish() {
        Err(rejected) => rejected,
        Ok(_) => return Err("incomplete File minted owner"),
    };
    assert_eq!(rejected.error(), JsxFileError::IncompleteFile);
    assert_eq!(rejected.observation().comments().len(), 1);
    let file = rejected.file().ok_or("retained File")?;
    assert!(!file.is_complete());
    assert!(matches!(
        project_program(file),
        Err(ProjectionError::IncompleteFile)
    ));
    Ok(())
}
