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
    for language in ["jsx", "tsx", "unknown"] {
        documents.open(uri(), "const value=1;value;".into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(0, 15))),
            Err(NavigationRefusal::Language)
        );
    }
    documents.open(uri(), "const value=1;value;".into(), 1, "vue".into());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 15))),
        Err(NavigationRefusal::Configuration)
    );
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
            "const value:number|string=1;value;",
            vec![(0, Span::new(6, 27), UnsupportedSyntax)],
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
        let snapshot = Arc::clone(&project.workers.lock().get(&uri()).unwrap().snapshot);
        assert_eq!(
            block_on(project.references(&uri(), Position::new(0, 7), false)),
            Err(NavigationRefusal::Producer(issues))
        );
        assert!(Arc::ptr_eq(
            &snapshot,
            &project.workers.lock().get(&uri()).unwrap().snapshot
        ));
    }
}

#[test]
fn primitive_keyword_annotation_keeps_original_ts_binding_and_query_coordinates() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    documents.open(
        uri(),
        "const value:number=1;value;".into(),
        1,
        "typescript".into(),
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 22))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    let original = super::cached(&project);
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 7), true)),
        Ok(vec![location((0, 6), (0, 11)), location((0, 21), (0, 26))])
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 13))),
        Ok(None)
    );
    assert!(Arc::ptr_eq(&original, &super::cached(&project)));
}
