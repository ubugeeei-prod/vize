use super::{
    Allocator, FileIssueKind, Parser, PositionQueryError, SourceRoot, SourceType, cstr, lower,
};

#[test]
fn self_contained_arrays_preserve_original_ts_and_tsx_profiles() {
    let arena = Allocator::default();
    for profile in [
        SourceType::ts().with_module(true),
        SourceType::tsx().with_module(true),
    ] {
        for element in [
            "bigint",
            "boolean",
            "null",
            "number",
            "string",
            "symbol",
            "undefined",
            "{}",
            "{ id:number; name:string; enabled:boolean }",
        ] {
            let source = cstr!("const items: {element}[] = [];items;");
            let observed = Parser::new(&arena, &source, profile).parse_observed();
            assert!(observed.diagnostics().is_empty(), "{source}");
            let block = SourceRoot::new(&source).unwrap().whole_block();
            let file = lower(&arena, &source, block, &observed);
            assert!(file.is_complete(), "{source}");
            assert_eq!(file.units()[0].profile.jsx, profile.is_jsx());
            assert!(file.units()[0].profile.typescript);
            assert!(file.units()[0].profile.module);
            assert_eq!(file.bindings().count(), 1);
            assert_eq!(file.references().len(), 1);
        }
    }
}

#[test]
fn named_recursive_and_nonflat_type_children_keep_whole_file_incomplete() {
    let arena = Allocator::default();
    for element in [
        "Name",
        "Imported.Name",
        "(number)",
        "number[]",
        "number | string",
        "number & string",
        "1",
        "any",
        "unknown",
        "never",
        "void",
        "object",
        "typeof value",
        "keyof Name",
        "[number]",
        "{ id:Name }",
        "{ id:number[] }",
        "{ id:{nested:number} }",
        "{ id?:number }",
        "{ readonly id:number }",
        "{ 'id':number }",
        "{ [value]:number }",
        "{ [name:string]:number }",
        "{ [K in string]:number }",
        "{ id() : number }",
        "{ () : number }",
        "{ new () : number }",
        "{ id }",
    ] {
        let source = cstr!("const value=1;const items: {element}[] = [];value;");
        let observed =
            Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
        assert!(observed.diagnostics().is_empty(), "{source}");
        let file = lower(
            &arena,
            &source,
            SourceRoot::new(&source).unwrap().whole_block(),
            &observed,
        );
        assert!(!file.is_complete(), "{source}");
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax),
            "{source}"
        );
        assert!(matches!(
            file.binding_at_offset(source.rfind("value").unwrap() as u32),
            Err(PositionQueryError::IncompleteFile)
        ));
    }
}

#[test]
fn context_and_unsupported_siblings_cannot_borrow_array_completeness() {
    let arena = Allocator::default();
    for source in [
        "export const items:number[]=[];",
        "declare const items:number[];",
        "let items!:number[];",
        "function local(){const items:number[]=[];}",
        "function local(items:number[]){}",
        "function local():number[]{return [];}",
        "const items:Array<number>=[];",
        "const items:readonly number[]=[];",
        "import type {Item} from 'types';const items:Item[]=[];",
        "const items:number[]=[];const other:Name=1;",
        "const other:Name=1;const items:number[]=[];",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(observed.diagnostics().is_empty(), "{source}");
        let file = lower(
            &arena,
            source,
            SourceRoot::new(source).unwrap().whole_block(),
            &observed,
        );
        assert!(!file.is_complete(), "{source}");
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax)
        );
    }
}
