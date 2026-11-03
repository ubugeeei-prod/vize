use super::*;
use oxc_ast::ast::{Declaration, Statement};
use oxc_parser::ParseOptions;
use vize_l0::{Span, String};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{JsxFileProducer, ProgramInput};

#[test]
fn original_named_export_nonzero_unicode_crlf_keeps_exact_declaration_and_name_scope() {
    let arena = Allocator::default();
    let source = "é😀\r\n<script>/* 雪😀 */\r\nexport function 表示(値:number): /* 型🌸 */ number {return 値;}</script>";
    let start = source.find("/*").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let parsed =
        Parser::new(&arena, block.source(), SourceType::ts().with_module(true)).parse_observed();
    let program = parsed.admitted().unwrap().program();
    let Statement::ExportNamedDeclaration(export) = &program.body[0] else {
        panic!("original named export")
    };
    let Some(Declaration::FunctionDeclaration(function)) = &export.declaration else {
        panic!("actual function")
    };
    let file = Box::new(finish_block(&arena, &parsed, block).unwrap());
    assert!(file.is_complete(), "{:?}", file.issues());
    assert_eq!(program.comments.len(), 2);
    assert_eq!(file.units()[0].span, block.span());
    assert_eq!(file.units()[0].id.index(), 7);
    let root = file.units()[0].scope;
    let declared = file.lookup(root, "表示", Namespace::Value).unwrap();
    let parameter = file
        .lookup(file.scopes()[1].id, "値", Namespace::Value)
        .unwrap();
    let row = &file.exports()[0];
    assert_eq!(row.local, Some(declared.id()));
    assert_eq!(row.span, declared.declaration().unwrap().span);
    assert_eq!(row.span.slice(source), "表示");
    assert_eq!(
        row.span.start,
        start as u32 + function.id.as_ref().unwrap().span.start
    );
    assert_eq!(row.unit, parameter.declaration().unwrap().unit);
    assert_eq!(file.scopes()[1].parent, Some(root));
    assert_eq!(
        file.scopes()[1].span.slice(source),
        "function 表示(値:number): /* 型🌸 */ number {return 値;}"
    );
    assert_eq!(parameter.declaration().unwrap().span.slice(source), "値");
    assert_eq!(
        file.references()[0].target,
        ReferenceTarget::Resolved(parameter.id())
    );
    assert_eq!(file.references()[0].span.slice(source), "値");
    for _ in 0..2 {
        let at = row.span.start;
        let queried = file.binding_at_offset(at).unwrap().unwrap();
        assert_eq!(queried.id(), declared.id());
        assert!(core::ptr::eq(queried.file(), file.as_ref()));
        let scope = file.scope_at_offset(at).unwrap().unwrap();
        assert_eq!(scope.scope().id, root);
        assert!(core::ptr::eq(scope.file(), file.as_ref()));
    }
    let Statement::ExportNamedDeclaration(current) = &parsed.admitted().unwrap().program().body[0]
    else {
        panic!("same export")
    };
    let Some(Declaration::FunctionDeclaration(current_function)) = &current.declaration else {
        panic!("same function")
    };
    assert!(core::ptr::eq(&**current, &**export));
    assert!(core::ptr::eq(&**current_function, &**function));
    assert!(core::ptr::eq(file.artifact().source(), source));
    drop(file);
    assert_eq!(program.comments.len(), 2);
    assert!(core::ptr::eq(parsed.admitted().unwrap().program(), program));
}

#[test]
fn genuine_named_tsx_owner_move_and_requery_keep_actual_export_and_original_fragment() {
    let arena = Allocator::default();
    let source = "/* original 😀 */ export function f(value:number):number{return value;}const view=<></>;view;";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let parsed = Parser::new(&arena, source, SourceType::tsx().with_module(true)).parse_observed();
    let body = parsed.admitted().unwrap().program().body.as_ptr();
    let mut producer = JsxFileProducer::new(&arena, parsed, block, 7)
        .unwrap_or_else(|r| panic!("original TSX: {:?}", r.error()));
    producer.walk().unwrap();
    let owner = Box::new(
        producer
            .finish()
            .unwrap_or_else(|r| panic!("complete TSX: {:?}", r.error())),
    );
    let file = owner.file();
    assert!(file.is_complete());
    assert!(file.units()[0].profile.jsx);
    let function = file
        .lookup(file.units()[0].scope, "f", Namespace::Value)
        .unwrap();
    let parameter = file
        .lookup(file.scopes()[1].id, "value", Namespace::Value)
        .unwrap();
    assert_eq!(file.exports()[0].local, Some(function.id()));
    assert_eq!(
        file.references()[0].target,
        ReferenceTarget::Resolved(parameter.id())
    );
    assert!(owner.nodes().any(|node| node.source() == Some("<></>")));
    for _ in 0..2 {
        let binding = file
            .binding_at_offset(file.references()[0].span.start)
            .unwrap()
            .unwrap();
        assert_eq!(binding.id(), parameter.id());
        assert!(core::ptr::eq(binding.file(), file));
    }
    assert_eq!(owner.observation().comments().len(), 1);
    assert_eq!(
        owner
            .observation()
            .admitted()
            .unwrap()
            .program()
            .body
            .as_ptr(),
        body
    );
    assert!(core::ptr::eq(file.artifact().source(), source));
}

#[test]
fn equal_source_foreign_arenas_and_nondefault_profiles_never_replace_named_export_owner() {
    let arena = Allocator::default();
    let other = Allocator::default();
    let source = String::from("export function f(value:number):number{return value;}");
    let foreign = source.clone();
    let a = Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
    let b = Parser::new(&other, &source, SourceType::ts().with_module(true)).parse_observed();
    assert!(!core::ptr::eq(
        a.admitted().unwrap().program().body.as_ptr(),
        b.admitted().unwrap().program().body.as_ptr()
    ));
    let left = finish(&arena, &a).unwrap();
    let right = finish(&other, &b).unwrap();
    let x = left
        .binding_at_offset(left.exports()[0].span.start)
        .unwrap()
        .unwrap();
    let y = right
        .binding_at_offset(right.exports()[0].span.start)
        .unwrap()
        .unwrap();
    assert_eq!(x.id(), y.id());
    assert!(core::ptr::eq(x.file(), &left));
    assert!(core::ptr::eq(y.file(), &right));
    assert!(!core::ptr::eq(x.file(), y.file()));
    assert!(
        matches!(ProgramInput::checked(a.admitted().unwrap(), SourceRoot::new(&foreign).unwrap().whole_block(), 7), Err(issue) if issue.kind==FileIssueKind::InvalidSource && issue.span==Span::new(0,source.len() as u32))
    );
    let changed = Parser::new(&arena, &source, SourceType::ts().with_module(true))
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse_observed();
    assert!(
        matches!(ProgramInput::checked(changed.admitted().unwrap(), SourceRoot::new(&source).unwrap().whole_block(), 7), Err(issue) if issue.kind==FileIssueKind::InvalidProfile && issue.span==Span::new(0,source.len() as u32))
    );
    assert!(left.is_complete() && right.is_complete());
}
