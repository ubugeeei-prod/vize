//! JSX profiles borrow the original completed File, never an inferred reparse.

use oxc_ast::ast::{Expression, Statement};
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span};
use vize_l1::embed::syntax::{NativeSyntax, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang, SourceError};
use vize_l2::file::Namespace;
use vize_l2::lang::js::{FileProducer, ModuleSourceKind, ProgramInput, ProgramScope};

fn syntax<'a>(
    arena: &'a Allocator,
    block: SourceBlock<'a>,
    lang: Lang,
) -> Result<NativeSyntax<'a>, SourceError> {
    Ok(parse_program_once(
        arena,
        EmbedSource::authored(block.root_source(), block.span())?,
        ProgramOptions {
            jsx: true,
            ..ProgramOptions::module(lang)
        },
    ))
}

#[test]
fn actual_jsx_and_tsx_elements_keep_original_module_literals_and_reference_rows() {
    for lang in [Lang::Js, Lang::Ts] {
        let arena = Allocator::default();
        let source = "import Widget from './部品\\u002ejsx'; const value = 1; const view = <><Widget label=\"&amp;\">{value}</Widget></>; export {thing as first, another as second} from './out.tsx';";
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = syntax(&arena, block, lang).unwrap();
        assert_eq!(original.diagnostics().count(), 0);
        let admitted = original.admitted_program().unwrap();
        assert!(admitted.source_type().is_jsx());
        assert_eq!(admitted.source_type().is_typescript(), lang == Lang::Ts);
        let program = admitted.program();
        let Statement::ImportDeclaration(import) = &program.body[0] else {
            panic!("original import")
        };
        let Statement::VariableDeclaration(view) = &program.body[2] else {
            panic!("original view")
        };
        assert!(matches!(
            view.declarations[0].init,
            Some(Expression::JSXFragment(_))
        ));
        let Statement::ExportNamedDeclaration(export) = &program.body[3] else {
            panic!("original export")
        };
        let literals = [&import.source, export.source.as_ref().unwrap()];
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(admitted, block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert!(file.is_complete());
        assert_eq!(file.references().len(), 2);
        assert_eq!(file.references()[0].name.as_str(), "Widget");
        assert_eq!(file.references()[1].name.as_str(), "value");
        let view = file
            .original_module_sources(
                ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
            )
            .unwrap();
        for _ in 0..2 {
            let mut visited = 0;
            view.for_each(|row| {
                assert!(core::ptr::eq(row.literal(), literals[visited]));
                assert!(core::ptr::eq(row.file(), &file));
                assert_eq!(row.namespace(), Namespace::Value);
                if visited == 0 {
                    assert_eq!(row.raw(), "'./部品\\u002ejsx'");
                    assert_eq!(row.authored_content(), "./部品\\u002ejsx");
                    assert_eq!(row.decoded_request(), "./部品.jsx");
                    assert!(core::ptr::eq(row.import().unwrap(), &file.imports()[0]));
                } else {
                    assert_eq!(row.kind(), ModuleSourceKind::NamedReexport);
                    assert!(core::ptr::eq(row.export().unwrap(), &file.exports()[0]));
                }
                assert_eq!(row.copied_request(), row.decoded_request());
                visited += 1;
            })
            .unwrap();
            assert_eq!(
                visited, 2,
                "named reexport specifiers share one actual literal"
            );
        }
        assert!(core::ptr::eq(program, original.program().unwrap()));
        assert_eq!(
            file.references().len(),
            2,
            "reads preserve the original sole walk"
        );
    }
}

#[test]
fn original_tsx_declaration_namespaces_remain_separate_from_jsx_component_reads() {
    let arena = Allocator::default();
    let source = "import type {Model} from './types.ts'; import Widget from './widget.tsx'; const view = <Widget/>; export type {Model as PublicModel} from './public.ts';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = syntax(&arena, block, Lang::Ts).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    let expected = [
        (ModuleSourceKind::Import, Namespace::Type, "./types.ts"),
        (ModuleSourceKind::Import, Namespace::Value, "./widget.tsx"),
        (
            ModuleSourceKind::NamedReexport,
            Namespace::Type,
            "./public.ts",
        ),
    ];
    let mut visited = 0;
    file.original_module_sources(
        ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
    )
    .unwrap()
    .for_each(|row| {
        assert_eq!(
            (row.kind(), row.namespace(), row.decoded_request()),
            expected[visited]
        );
        visited += 1;
    })
    .unwrap();
    assert_eq!(visited, 3);
    assert_eq!(file.references().len(), 1);
    let reference = file
        .reference_at_offset(source.find("<Widget").unwrap() as u32 + 1)
        .unwrap()
        .unwrap();
    assert_eq!(
        reference
            .binding()
            .unwrap()
            .declaration()
            .unwrap()
            .namespace,
        Namespace::Value
    );
}

#[test]
fn nonzero_original_jsx_source_keeps_authored_crlf_unicode_and_escaped_spans() {
    let arena = Allocator::default();
    let source = "🦀\r\n<script>/* kept */ import Widget from './日\\\r\n本.jsx'; const view = <Widget/>;</script>tail";
    let start = source.find("/* kept */").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let original = syntax(&arena, block, Lang::Js).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(original.admitted_program().unwrap(), block, 7).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert_eq!(original.comments().count(), 1);
    let raw = "'./日\\\r\n本.jsx'";
    let position = source.find(raw).unwrap() as u32;
    let mut visited = 0;
    file.original_module_sources(
        ProgramInput::checked(original.admitted_program().unwrap(), block, 7).unwrap(),
    )
    .unwrap()
    .for_each(|row| {
        assert_eq!(
            row.quoted_span(),
            Span::new(position, position + raw.len() as u32)
        );
        assert_eq!(
            row.content_span(),
            Span::new(position + 1, position + raw.len() as u32 - 1)
        );
        assert_eq!(row.raw(), raw);
        assert!(core::ptr::eq(
            row.raw(),
            source
                .get(position as usize..position as usize + raw.len())
                .unwrap()
        ));
        assert_eq!(row.decoded_request(), "./日本.jsx");
        assert!(row.declaration_span().start >= block.span().start);
        assert!(row.declaration_span().end <= block.span().end);
        visited += 1;
    })
    .unwrap();
    assert_eq!(visited, 1);
}

mod refusals;
