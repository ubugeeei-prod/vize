//! Genuine JSX/TSX module queries through the existing original-owner worker.
mod lifecycle;

use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};
use vize_l0::Span;
use vize_l2::file::FileIssueKind;

const SOURCE: &str = "import View from 'dep';\nimport UI from 'ui';\nconst value=1;\nconst props={};\nconst tree=<View prop={value} {...props}><UI.Button>{value}</UI.Button><div value='value'/></View>;\nvalue;";

#[test]
fn jsx_and_tsx_use_real_opening_roots_containers_and_spreads_only() {
    for language in ["javascriptreact", "typescriptreact"] {
        let documents = DocumentStore::new();
        documents.open(uri(), SOURCE.into(), 1, language.into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        for (at, declaration) in [
            (13, location((0, 7), (0, 11))),
            (43, location((1, 7), (1, 9))),
            (24, location((2, 6), (2, 11))),
            (54, location((2, 6), (2, 11))),
            (35, location((3, 6), (3, 11))),
        ] {
            assert_eq!(
                block_on(project.definition(&uri(), Position::new(4, at))),
                Ok(Some(declaration))
            );
        }
        assert_eq!(
            block_on(project.references(&uri(), Position::new(2, 7), true)),
            Ok(vec![
                location((2, 6), (2, 11)),
                location((4, 23), (4, 28)),
                location((4, 53), (4, 58)),
                location((5, 0), (5, 5)),
            ])
        );
        assert_eq!(
            block_on(project.references(&uri(), Position::new(0, 8), false)),
            Ok(vec![location((4, 12), (4, 16))])
        );
        assert_eq!(
            block_on(project.references(&uri(), Position::new(1, 8), false)),
            Ok(vec![location((4, 42), (4, 44))])
        );
        // Static properties, intrinsic/attribute names and closing names are
        // genuine syntax leaves, never invented binding references.
        for at in [46, 62, 65, 73, 77, 84, 94] {
            assert_eq!(
                block_on(project.definition(&uri(), Position::new(4, at))),
                Ok(None)
            );
            assert_eq!(
                block_on(project.references(&uri(), Position::new(4, at), true)),
                Ok(vec![])
            );
        }
    }
}

#[test]
fn original_tsx_types_unicode_and_shadowed_member_roots_keep_utf16_targets() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "/*😀*/ const café:number=1;\r\nfunction render(café) { return <café.部品 title={café}>{café}</café.部品>; }\r\nconst tree=<div>{café}</div>;\r\ncafé;".into(),
        1,
        "typescriptreact".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 33))),
        Ok(Some(location((1, 16), (1, 20))))
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 17), true)),
        Ok(vec![
            location((1, 16), (1, 20)),
            location((1, 32), (1, 36)),
            location((1, 47), (1, 51)),
            location((1, 54), (1, 58)),
        ])
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(2, 18))),
        Ok(Some(location((0, 13), (0, 17))))
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(3, 1), true)),
        Ok(vec![
            location((0, 13), (0, 17)),
            location((2, 17), (2, 21)),
            location((3, 0), (3, 4)),
        ])
    );
    for at in [38, 62, 67] {
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(1, at))),
            Ok(None)
        );
    }
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 20))),
        Ok(None)
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 3))),
        Err(NavigationRefusal::Position)
    );
}

#[test]
fn document_language_selects_jsx_and_explicit_tsx_module_without_source_retry() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    // The .ts URI cannot supply JSX permission to a plain JS/TS Document.
    for language in ["javascript", "typescript"] {
        documents.open(uri(), SOURCE.into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(4, 24))),
            Err(NavigationRefusal::Syntax)
        );
    }
    for language in ["javascriptreact", "typescriptreact"] {
        documents.open(uri(), SOURCE.into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(4, 24))),
            Ok(Some(location((2, 6), (2, 11))))
        );
        assert!(matches!(
            cached(&project).profile(),
            super::super::profile::Profile::Program(options)
                if options.jsx && options.source_type().is_module()
                    && !options.source_type().is_unambiguous()
                    && options.source_type().is_typescript() == (language == "typescriptreact")
        ));
    }
    documents.open(
        uri(),
        "const value:number=1;const tree=<div>{value}</div>;".into(),
        1,
        "javascriptreact".into(),
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 7))),
        Err(NavigationRefusal::Syntax)
    );
}

#[test]
fn original_jsx_syntax_and_unresolved_or_unsupported_tsx_keep_sticky_refusals() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    documents.open(
        uri(),
        "const tree=<div>".into(),
        1,
        "javascriptreact".into(),
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 7))),
        Err(NavigationRefusal::Syntax)
    );
    for (source, span, kind) in [
        (
            "const tree=<Missing/>;",
            Span::new(12, 19),
            FileIssueKind::UnresolvedReference,
        ),
        (
            "const value:number|string=1;const tree=<div>{value}</div>;",
            Span::new(6, 27),
            FileIssueKind::UnsupportedSyntax,
        ),
    ] {
        documents.open(uri(), source.into(), 1, "typescriptreact".into());
        let Err(NavigationRefusal::Producer(issues)) =
            block_on(project.definition(&uri(), Position::new(0, 7)))
        else {
            panic!("original incomplete File must refuse")
        };
        assert_eq!(issues.len(), 1);
        assert_eq!(
            (issues[0].unit.index(), issues[0].span, issues[0].kind),
            (0, span, kind)
        );
        assert_eq!(
            block_on(project.references(&uri(), Position::new(0, 7), true)),
            Err(NavigationRefusal::Producer(issues))
        );
    }
}

#[test]
fn repeated_jsx_requests_keep_one_actual_program_and_file_after_profile_capture() {
    let documents = DocumentStore::new();
    documents.open(uri(), SOURCE.into(), 1, "javascriptreact".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    block_on(project.definition(&uri(), Position::new(4, 24))).unwrap();
    let worker = cached(&project);
    let before = block_on(worker.inspect(Position::new(4, 24))).unwrap();
    assert_eq!(before.statements, 6);
    assert_eq!(before.parses, 1);
    assert_eq!(before.declaration, Some(Span::new(51, 56)));
    block_on(project.references(&uri(), Position::new(2, 7), false)).unwrap();
    let after = block_on(worker.inspect(Position::new(2, 7))).unwrap();
    assert_eq!(before, after);
    assert_eq!(worker.counts(), (1, 2));
    assert!(Arc::ptr_eq(&worker, &cached(&project)));
}
