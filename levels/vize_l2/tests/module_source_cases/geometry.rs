use super::{lower, moved, observe};
use oxc_ast::ast::{Statement, StringLiteral};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::file::Namespace;
use vize_l2::lang::js::{ModuleSourceKind, ProgramInput};

#[test]
fn moved_original_js_owners_keep_real_operands_rows_and_authored_geometry() {
    let arena = Allocator::default();
    let source = "🦀\r\n<script>/* retained */ import '日本語\\u002Fx'; import {thing as local} from \"dep\"; export {thing as a, another as b} from 'remote\\\r\npath'; export * as all from \"all\"; export {local}; export default 1;</script>tail";
    let start = source.find("/* retained */").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let original = observe(&arena, block, SourceType::mjs());
    let file = lower(&arena, &original, block, 3);
    let body = original.admitted().unwrap().program();
    let originals: [&StringLiteral<'_>; 4] = [
        match &body.body[0] {
            Statement::ImportDeclaration(row) => &row.source,
            _ => panic!("side effect import"),
        },
        match &body.body[1] {
            Statement::ImportDeclaration(row) => &row.source,
            _ => panic!("named import"),
        },
        match &body.body[2] {
            Statement::ExportNamedDeclaration(row) => row.source.as_ref().unwrap(),
            _ => panic!("named reexport"),
        },
        match &body.body[3] {
            Statement::ExportAllDeclaration(row) => &row.source,
            _ => panic!("all reexport"),
        },
    ];
    let original_addresses = originals.map(core::ptr::from_ref);
    let original = moved(original);
    let file = moved(file);
    assert_eq!(original.comments().len(), 1);
    assert_eq!(file.imports().len(), 2);
    assert_eq!(file.exports().len(), 5);
    let input = ProgramInput::checked(original.admitted().unwrap(), block, 3).unwrap();
    let view = file.original_module_sources(input).unwrap();
    let expected = [
        (
            "'日本語\\u002Fx'",
            "日本語\\u002Fx",
            "日本語/x",
            ModuleSourceKind::Import,
        ),
        ("\"dep\"", "dep", "dep", ModuleSourceKind::Import),
        (
            "'remote\\\r\npath'",
            "remote\\\r\npath",
            "remotepath",
            ModuleSourceKind::NamedReexport,
        ),
        ("\"all\"", "all", "all", ModuleSourceKind::AllReexport),
    ];
    let mut count = 0;
    view.for_each(|row| {
        let (raw, authored, decoded, kind) = expected[count];
        assert_eq!(
            core::ptr::from_ref(row.literal()),
            original_addresses[count]
        );
        assert!(core::ptr::eq(row.file(), &file));
        assert_eq!(row.raw(), raw);
        assert_eq!(row.authored_content(), authored);
        assert_eq!(row.decoded_request(), decoded);
        assert_eq!(row.copied_request(), decoded);
        assert_eq!(row.kind(), kind);
        assert_eq!(row.namespace(), Namespace::Value);
        let position = source.find(raw).unwrap() as u32;
        assert_eq!(
            row.quoted_span(),
            Span::new(position, position + raw.len() as u32)
        );
        assert_eq!(
            row.content_span(),
            Span::new(position + 1, position + raw.len() as u32 - 1)
        );
        assert!(core::ptr::eq(
            row.raw(),
            source
                .get(position as usize..position as usize + raw.len())
                .unwrap()
        ));
        assert!(row.declaration_span().start <= row.quoted_span().start);
        assert!(row.quoted_span().end <= row.declaration_span().end);
        if count < 2 {
            assert!(core::ptr::eq(row.import().unwrap(), &file.imports()[count]));
            assert!(row.export().is_none());
        } else {
            let index = if count == 2 { 0 } else { 2 };
            assert!(core::ptr::eq(row.export().unwrap(), &file.exports()[index]));
            assert!(row.import().is_none());
        }
        count += 1;
    })
    .unwrap();
    assert_eq!(
        count, 4,
        "two named specifiers share one original source operand"
    );
}

#[test]
fn original_ts_declaration_namespaces_are_separate_from_specifier_namespaces() {
    let arena = Allocator::default();
    let source = "import type {T} from 'types'; export type {T as U, T as V} from 'types'; export {type T, V} from 'mixed'; export type * from 'all';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::ts().with_module(true));
    let file = lower(&arena, &original, block, 0);
    assert_eq!(file.exports()[2].namespace, Namespace::Type);
    assert_eq!(file.exports()[3].namespace, Namespace::Value);
    let view = file
        .original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        )
        .unwrap();
    let expected = [
        Namespace::Type,
        Namespace::Type,
        Namespace::Value,
        Namespace::Type,
    ];
    let mut count = 0;
    view.for_each(|row| {
        assert_eq!(row.namespace(), expected[count]);
        count += 1;
    })
    .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn empty_authored_requests_and_unicode_escape_values_keep_real_token_bounds() {
    let arena = Allocator::default();
    let source = "import ''; import '\\u{1F980}'; import '\\uFFFD';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let file = lower(&arena, &original, block, 0);
    let view = file
        .original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        )
        .unwrap();
    let expected = [("", ""), ("\\u{1F980}", "🦀"), ("\\uFFFD", "�")];
    let mut count = 0;
    view.for_each(|row| {
        assert_eq!(row.authored_content(), expected[count].0);
        assert_eq!(row.decoded_request(), expected[count].1);
        assert_eq!(row.content_span().len() as usize, expected[count].0.len());
        assert_eq!(row.quoted_span().len(), row.content_span().len() + 2);
        count += 1;
    })
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn all_five_original_line_continuations_keep_authored_bytes_and_decoded_value() {
    for ending in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        let arena = Allocator::default();
        let source = format!("import '雪\\{ending}🦀';");
        let block = SourceRoot::new(&source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        let file = lower(&arena, &original, block, 0);
        let mut count = 0;
        file.original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        )
        .unwrap()
        .for_each(|row| {
            assert_eq!(row.decoded_request(), "雪🦀");
            assert_eq!(row.authored_content(), format!("雪\\{ending}🦀"));
            assert_eq!(
                row.content_span().len() as usize,
                row.authored_content().len()
            );
            assert_eq!(row.quoted_span().len(), row.content_span().len() + 2);
            assert!(source.is_char_boundary(row.content_span().start as usize));
            assert!(source.is_char_boundary(row.content_span().end as usize));
            count += 1;
        })
        .unwrap();
        assert_eq!(count, 1);
    }
}

#[test]
fn quoted_imported_and_exported_names_never_become_module_source_operands() {
    let arena = Allocator::default();
    let source = "import {'remote/name' as local} from 'actual'; export {'original/name' as 'public/name'} from 'remote';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let file = lower(&arena, &original, block, 0);
    assert_eq!(
        file.bindings()
            .next()
            .unwrap()
            .declaration()
            .unwrap()
            .imported_name
            .as_deref(),
        Some("remote/name")
    );
    assert_eq!(file.exports()[0].name, "public/name");
    let expected = ["actual", "remote"];
    let mut count = 0;
    file.original_module_sources(
        ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
    )
    .unwrap()
    .for_each(|row| {
        assert_eq!(row.decoded_request(), expected[count]);
        count += 1;
    })
    .unwrap();
    assert_eq!(count, 2);
}
