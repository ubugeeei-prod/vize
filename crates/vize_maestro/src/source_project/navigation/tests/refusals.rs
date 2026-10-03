use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, location, uri,
};
use vize_l0::Span;
use vize_l2::file::FileIssueKind::{UnresolvedReference, UnsupportedSyntax};

#[test]
fn actual_document_language_controls_the_profile_despite_uri_or_source() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    for language in ["vue", "javascriptreact", "typescriptreact", "unknown"] {
        documents.open(uri(), "const value=1;value;".into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(0, 15))),
            Err(NavigationRefusal::Language)
        );
    }
    documents.open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 15))),
        Ok(Some(location((0, 6), (0, 11))))
    );
}

#[test]
fn recovered_program_and_incomplete_file_never_fall_back_to_legacy_queries() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    documents.open(
        uri(),
        "const value = /x/uv; value;".into(),
        1,
        "javascript".into(),
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 7))),
        Err(NavigationRefusal::Syntax)
    );
    for (source, expected) in [
        (
            "const value:number=1;value;",
            vec![(0, Span::new(6, 20), UnsupportedSyntax)],
        ),
        (
            "const {value}=other;value;",
            vec![
                (0, Span::new(6, 19), UnsupportedSyntax),
                (0, Span::new(20, 25), UnresolvedReference),
            ],
        ),
        (
            "const value=1;missing;",
            vec![(0, Span::new(14, 21), UnresolvedReference)],
        ),
    ] {
        documents.open(uri(), source.into(), 1, "typescript".into());
        let result = block_on(project.definition(&uri(), Position::new(0, 7)));
        let Err(NavigationRefusal::Producer(issues)) = result else {
            panic!("native File must refuse original unsupported observation")
        };
        assert_eq!(
            issues
                .iter()
                .map(|issue| (issue.unit.index(), issue.span, issue.kind))
                .collect::<Vec<_>>(),
            expected
        );
        let snapshot = Arc::clone(&project.summaries.lock().get(&uri()).unwrap().snapshot);
        assert_eq!(
            block_on(project.references(&uri(), Position::new(0, 7), false)),
            Err(NavigationRefusal::Producer(issues))
        );
        assert!(Arc::ptr_eq(
            &snapshot,
            &project.summaries.lock().get(&uri()).unwrap().snapshot
        ));
    }
}
