use super::{
    Allocator, DeclarationKind, Namespace, Parser, SourceRoot, SourceType, finish, finish_block,
};
use oxc_ast::ast::{Statement, TSType};
use oxc_parser::ParseOptions;
use oxc_span::GetSpan;
use vize_l0::String;
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::ProgramInput;

#[test]
fn optional_identifier_parameter_and_annotation_keep_original_unicode_crlf_geometry() {
    let arena = Allocator::default();
    let source = "🌸é<script>/* original 😀 */\r\nfunction 表示(値?: /* 型🌸 */ number) { return 値; }</script>";
    let start = source.find("/*").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let observed =
        Parser::new(&arena, block.source(), SourceType::ts().with_module(true)).parse_observed();
    let program = observed.admitted().unwrap().program();
    let body = program.body.as_ptr();
    let comments = program.comments.as_ptr();
    let Statement::FunctionDeclaration(function) = &program.body[0] else {
        panic!("the original function declaration");
    };
    let parameter = &function.params.items[0];
    assert!(parameter.optional);
    let annotation = parameter.type_annotation.as_ref().unwrap();
    assert!(matches!(
        annotation.type_annotation,
        TSType::TSNumberKeyword(_)
    ));
    let authored = |span: oxc_span::Span| {
        block
            .span_of(&block.source()[span.start as usize..span.end as usize])
            .unwrap()
    };
    let whole = authored(parameter.span);
    assert_eq!(whole.slice(source), "値?: /* 型🌸 */ number");
    assert_eq!(whole.start as usize, source.find("値?").unwrap());
    let outer = authored(annotation.span);
    let inner = authored(annotation.type_annotation.span());
    assert_eq!(outer.slice(source), ": /* 型🌸 */ number");
    assert_eq!(inner.slice(source), "number");
    assert_eq!(outer.start as usize, source.find(": /*").unwrap());
    assert_eq!(inner.start as usize, source.find("number").unwrap());
    let file = finish_block(&arena, &observed, block).unwrap();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(file.units()[0].span, block.span());
    assert!(file.units()[0].profile.module);
    assert!(file.units()[0].profile.typescript);
    assert!(!file.units()[0].profile.jsx);
    let child = file.scopes()[1].id;
    let parameter = file.lookup(child, "値", Namespace::Value).unwrap();
    let declaration = parameter.declaration().unwrap();
    assert_eq!(declaration.kind, DeclarationKind::Parameter);
    assert_eq!(declaration.namespace, Namespace::Value);
    assert_eq!(declaration.unit, file.units()[0].id);
    assert_eq!(declaration.span.start as usize, source.find("値").unwrap());
    assert_eq!(declaration.span.slice(source), "値");
    assert_ne!(declaration.span, whole);
    assert!(file.lookup(child, "値?", Namespace::Value).is_none());
    assert_eq!(file.bindings().count(), 2);
    assert_eq!(
        file.references()[0].span.start as usize,
        source.rfind("値").unwrap()
    );
    assert_eq!(
        file.scopes()[1].span.slice(source),
        "function 表示(値?: /* 型🌸 */ number) { return 値; }"
    );
    let original = observed.admitted().unwrap();
    assert!(core::ptr::eq(original.source(), block.source()));
    assert_eq!(original.program().body.as_ptr(), body);
    assert_eq!(original.program().comments.as_ptr(), comments);
    assert_eq!(original.program().comments.len(), 2);
    assert!(observed.diagnostics().is_empty());
    assert!(core::ptr::eq(file.artifact().source(), source));
}

#[test]
fn optional_parameter_cannot_substitute_source_options_or_nonmodule_profiles() {
    let arena = Allocator::default();
    let source = String::from("function f(value?: number) { return value; }");
    let copy = source.clone();
    let block = SourceRoot::new(&source).unwrap().whole_block();
    let original =
        Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
    assert!(
        matches!(ProgramInput::checked(original.admitted().unwrap(), SourceRoot::new(&copy).unwrap().whole_block(), 7),
        Err(issue) if issue.kind == FileIssueKind::InvalidSource)
    );
    let changed = Parser::new(&arena, &source, SourceType::ts().with_module(true))
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse_observed();
    assert!(
        matches!(ProgramInput::checked(changed.admitted().unwrap(), block, 7),
        Err(issue) if issue.kind == FileIssueKind::InvalidProfile)
    );
    for profile in [
        SourceType::ts().with_script(true),
        SourceType::tsx().with_script(true),
    ] {
        let observed = Parser::new(&arena, &source, profile).parse_observed();
        assert!(observed.admitted().is_some());
        let file = finish(&arena, &observed).unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.bindings().count(), 0);
        let issues = file.issues();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, FileIssueKind::InvalidProfile);
        assert_eq!(issues[0].span, block.span());
    }
    let observed = Parser::new(&arena, &source, SourceType::mjs()).parse_observed();
    assert!(observed.admitted().is_none());
    assert!(!observed.diagnostics().is_empty());
    let ambient = "declare function f(value?: number): number;";
    let observed =
        Parser::new(&arena, ambient, SourceType::d_ts().with_module(true)).parse_observed();
    let file = finish(&arena, &observed).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.bindings().count(), 0);
    assert_eq!(file.issues().len(), 1);
    assert_eq!(file.issues()[0].kind, FileIssueKind::InvalidProfile);
}
