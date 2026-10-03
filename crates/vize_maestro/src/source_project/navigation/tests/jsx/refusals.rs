use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, location, uri,
};
use vize_l2::file::FileIssueKind::{UnresolvedReference, UnsupportedSyntax};

#[test]
fn real_document_language_selects_jsx_without_extension_or_source_guessing() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let source = "const UI=1;\nconst view=<UI/>;";
    // The shared fixture URI is .ts, including when its genuine host language
    // is javascriptreact. Plain JS/TS never retries parsing with JSX enabled.
    for language in ["javascript", "typescript"] {
        documents.open(uri(), source.into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(1, 13))),
            Err(NavigationRefusal::Syntax)
        );
    }
    for language in ["javascriptreact", "typescriptreact"] {
        documents.open(uri(), source.into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(1, 13))),
            Ok(Some(location((0, 6), (0, 8))))
        );
    }
    for language in ["vue", "jsx", "tsx", "unknown"] {
        documents.open(uri(), source.into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(1, 13))),
            Err(NavigationRefusal::Language)
        );
    }
}

#[test]
fn unsupported_and_unresolved_jsx_retains_complete_original_issue_vectors() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    for (expression, kind, authored) in [
        ("<ns:tag/>", UnsupportedSyntax, "ns:tag"),
        ("<this.Button/>", UnsupportedSyntax, "this"),
        ("<UI<Type>/>", UnsupportedSyntax, "<UI<Type>/>"),
        (
            "<div>{value as Type}</div>",
            UnsupportedSyntax,
            "value as Type",
        ),
        ("<Missing/>", UnresolvedReference, "Missing"),
    ] {
        let source = format!("const UI=1; const value=2; const view={expression};");
        documents.open(uri(), source.clone(), 1, "typescriptreact".into());
        let result = block_on(project.definition(&uri(), Position::new(0, 7)));
        let Err(NavigationRefusal::Producer(issues)) = result else {
            panic!("incomplete actual JSX File cannot expose partial positive bindings")
        };
        assert_eq!(
            issues
                .iter()
                .map(|issue| (
                    issue.unit.index(),
                    issue.kind,
                    source.get(issue.span.start as usize..issue.span.end as usize),
                ))
                .collect::<Vec<_>>(),
            [(0, kind, Some(authored))]
        );
        let original = Arc::clone(&project.summaries.lock().get(&uri()).unwrap().snapshot);
        assert_eq!(
            block_on(project.references(&uri(), Position::new(0, 7), true)),
            Err(NavigationRefusal::Producer(issues))
        );
        assert!(Arc::ptr_eq(
            &original,
            &project.summaries.lock().get(&uri()).unwrap().snapshot
        ));
    }
    documents.open(
        uri(),
        "const view=<div>;".into(),
        1,
        "typescriptreact".into(),
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 7))),
        Err(NavigationRefusal::Syntax)
    );
}
