mod lifecycle;
mod refusals;

use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};

#[test]
fn actual_react_profiles_reuse_original_component_and_utf16_container_targets() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    for language in ["javascriptreact", "typescriptreact"] {
        documents.open(
            uri(),
            "/*😀*/ import UI from 'dep';\r\nconst café=1;\r\nconst view=<UI.Button value={café}><div>{café}</div><UI.Button/></UI.Button>;".into(),
            1,
            language.into(),
        );
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(2, 13))),
            Ok(Some(location((0, 14), (0, 16))))
        );
        let original = cached(&project);
        assert_eq!(
            block_on(project.references(&uri(), Position::new(2, 13), true)),
            Ok(vec![
                location((0, 14), (0, 16)),
                location((2, 12), (2, 14)),
                location((2, 53), (2, 55)),
            ])
        );
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(2, 30))),
            Ok(Some(location((1, 6), (1, 10))))
        );
        assert_eq!(
            block_on(project.references(&uri(), Position::new(2, 42), false)),
            Ok(vec![location((2, 29), (2, 33)), location((2, 41), (2, 45)),])
        );
        // Static properties, attributes, intrinsic tags and closing names have
        // no authored binding occurrence in the actual provider's File facts.
        for character in [16, 23, 37, 67] {
            assert_eq!(
                block_on(project.definition(&uri(), Position::new(2, character))),
                Ok(None)
            );
        }
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(0, 3))),
            Err(NavigationRefusal::Position)
        );
        assert!(Arc::ptr_eq(&original, &cached(&project)));
    }
}

#[test]
fn original_jsx_function_scope_keeps_shadowed_container_bindings_distinct() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "const value=1;\nfunction render(value){return <div>{value}</div>;}\nconst view=<div>{value}</div>;".into(),
        1,
        "javascriptreact".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 37))),
        Ok(Some(location((1, 16), (1, 21))))
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 7), true)),
        Ok(vec![location((0, 6), (0, 11)), location((2, 17), (2, 22))])
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 17), false)),
        Ok(vec![location((1, 36), (1, 41))])
    );
}

#[test]
fn original_jsx_fragments_and_spreads_project_only_runtime_read_occurrences() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "const value=1; const props=2;\nconst view=<><div value={value} {...props}>{value}{...props}</div></>;".into(),
        1,
        "typescriptreact".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 26), false)),
        Ok(vec![location((1, 25), (1, 30)), location((1, 44), (1, 49))])
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 37), true)),
        Ok(vec![
            location((0, 21), (0, 26)),
            location((1, 36), (1, 41)),
            location((1, 54), (1, 59)),
        ])
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 19))),
        Ok(None)
    );
}
