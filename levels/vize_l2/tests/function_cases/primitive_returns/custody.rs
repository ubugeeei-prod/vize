use super::*;
use oxc_ast::ast::{Statement, TSType};
use oxc_parser::ParseOptions;
use oxc_span::GetSpan;
use vize_l0::{Span, String};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::ProgramInput;

#[test]
fn original_nonzero_unicode_crlf_return_annotation_keeps_its_live_ast_and_authored_sites() {
    let arena = Allocator::default();
    let source = "é😀\r\n<script>/* 雪😀 */\r\nfunction 表示(値:number): /* 型🌸 */ number {return 値;}</script>";
    let start = source.find("/*").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let observed =
        Parser::new(&arena, block.source(), SourceType::ts().with_module(true)).parse_observed();
    let program = observed.admitted().unwrap().program();
    let Statement::FunctionDeclaration(function) = &program.body[0] else {
        panic!("original function")
    };
    let annotation = function.return_type.as_ref().unwrap();
    let file = finish_block(&arena, &observed, block).unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(program.comments.len(), 2);
    assert!(core::ptr::eq(
        observed.admitted().unwrap().program(),
        program
    ));
    let Statement::FunctionDeclaration(current) = &observed.admitted().unwrap().program().body[0]
    else {
        panic!("same function")
    };
    assert!(core::ptr::eq(&**current, &**function));
    assert!(core::ptr::eq(
        &**current.return_type.as_ref().unwrap(),
        &**annotation
    ));
    let original = Span::new(
        start as u32 + annotation.span.start,
        start as u32 + annotation.span.end,
    );
    assert_eq!(original.slice(source), ": /* 型🌸 */ number");
    assert!(matches!(
        annotation.type_annotation,
        TSType::TSNumberKeyword(_)
    ));
    let inner = annotation.type_annotation.span();
    let inner = Span::new(start as u32 + inner.start, start as u32 + inner.end);
    assert_eq!(inner.slice(source), "number");
    assert_eq!(inner.start as usize, source.find("*/ number").unwrap() + 3);
    let parameter = file
        .lookup(file.scopes()[1].id, "値", Namespace::Value)
        .unwrap();
    let declaration = parameter.declaration().unwrap();
    assert_eq!(
        declaration.span.start as usize,
        source.find("値:number").unwrap()
    );
    assert_eq!(declaration.span.slice(source), "値");
    assert_eq!(
        file.references()[0].span.start as usize,
        source.find("return 値").unwrap() + 7
    );
    assert_eq!(
        file.references()[0].target,
        ReferenceTarget::Resolved(parameter.id())
    );
    assert_eq!(
        file.scopes()[1].span.slice(source),
        "function 表示(値:number): /* 型🌸 */ number {return 値;}"
    );
    assert_eq!(file.units()[0].span, block.span());
    assert_eq!(file.units()[0].id.index(), 7);
    assert!(core::ptr::eq(file.artifact().source(), source));
}

#[test]
fn original_annotated_function_never_accepts_equal_foreign_source_or_nondefault_parser() {
    let arena = Allocator::default();
    let source = String::from("function f(value:number):number{return value;}");
    let foreign = source.clone();
    let observed =
        Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
    assert!(matches!(
        ProgramInput::checked(observed.admitted().unwrap(), SourceRoot::new(&foreign).unwrap().whole_block(), 7),
        Err(issue) if issue.span == Span::new(0, source.len() as u32) && issue.kind == FileIssueKind::InvalidSource
    ));
    let changed = Parser::new(&arena, &source, SourceType::ts().with_module(true))
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse_observed();
    assert!(matches!(
        ProgramInput::checked(changed.admitted().unwrap(), SourceRoot::new(&source).unwrap().whole_block(), 7),
        Err(issue) if issue.span == Span::new(0, source.len() as u32) && issue.kind == FileIssueKind::InvalidProfile
    ));
    let original = finish(&arena, &observed).unwrap();
    assert!(original.is_complete());
    assert_eq!(original.references().len(), 1);
}

#[test]
fn actual_script_definition_and_javascript_profiles_never_promote_keyword_returns() {
    let arena = Allocator::default();
    let source = "function f(value:number):number{return value;}";
    for profile in [
        SourceType::ts().with_script(true),
        SourceType::tsx().with_script(true),
    ] {
        let observed = Parser::new(&arena, source, profile).parse_observed();
        assert!(
            observed.admitted().is_some(),
            "{:?}",
            observed.diagnostics()
        );
        let file = finish(&arena, &observed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.bindings().count(), 0);
        assert_eq!(
            file.issues(),
            &[vize_l2::file::FileIssue {
                unit: file.units()[0].id,
                span: Span::new(0, source.len() as u32),
                kind: FileIssueKind::InvalidProfile,
            }]
        );
    }
    let ambient = "declare function f(value:number):number;";
    let observed =
        Parser::new(&arena, ambient, SourceType::d_ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.bindings().count(), 0);
    assert_eq!(
        file.issues(),
        &[vize_l2::file::FileIssue {
            unit: file.units()[0].id,
            span: Span::new(0, ambient.len() as u32),
            kind: FileIssueKind::InvalidProfile,
        }]
    );
    let observed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    assert!(observed.admitted().is_none());
    assert!(!observed.diagnostics().is_empty());
}
