//! Actual element syntax retains one original source/File/worker for both reads.
use super::{Arc, DocumentStore, NativeNavigationProject, Position, SourceQueryProject};
use super::{NavigationRefusal, SnapshotRefusal, SourceSnapshotCache};
use super::{Url, block_on, range, uri, worker};
use tower_lsp::lsp_types::{DocumentLink, Location};

const SOURCE: &str = "/*😀*/import Widget from './child.ts';\r\nconst local=1;\r\nconst tree=<Widget value={local}/>;\r\nexport {alpha,beta} from './second.vue';";

#[test]
fn module_link_real_jsx_and_tsx_elements_reuse_the_same_original_program_worker() {
    for language in ["javascriptreact", "typescriptreact"] {
        let documents = DocumentStore::new();
        documents.open(uri(), SOURCE.into(), 1, language.into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let original = worker(&project);
        let before = block_on(original.inspect(Position::new(0, 0))).unwrap();
        let targets = [
            Url::parse("file:///observed/child.ts").unwrap(),
            Url::parse("file:///observed/second.vue").unwrap(),
        ];
        let expected = vec![
            DocumentLink {
                range: range((0, 25), (0, 37)),
                target: Some(targets[0].clone()),
                tooltip: None,
                data: None,
            },
            DocumentLink {
                range: range((3, 25), (3, 39)),
                target: Some(targets[1].clone()),
                tooltip: None,
                data: None,
            },
        ];
        for _ in 0..2 {
            let operands = block_on(original.module_operands()).unwrap();
            assert_eq!(
                operands.decoded_requests().collect::<Vec<_>>(),
                ["./child.ts", "./second.vue"]
            );
            assert_eq!(operands.into_links(&targets), Ok(expected.clone()));
        }
        assert_eq!(
            block_on(original.definition(Position::new(2, 12))),
            Ok(Some(Location::new(uri(), range((0, 13), (0, 19)))))
        );
        assert_eq!(
            block_on(original.inspect(Position::new(0, 0))).unwrap(),
            before
        );
        assert!(Arc::ptr_eq(&original, &worker(&project)));
        assert_eq!(original.counts(), (1, 3));
    }
}

#[test]
fn module_link_real_jsx_and_tsx_refuse_equal_foreign_source_owners() {
    for language in ["javascriptreact", "typescriptreact"] {
        let documents = DocumentStore::new();
        let foreign = DocumentStore::new();
        for store in [&documents, &foreign] {
            store.open(uri(), SOURCE.into(), 1, language.into());
        }
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let original = worker(&project);
        let snapshot = SourceSnapshotCache::default()
            .capture(&foreign, &uri())
            .unwrap();
        assert!(!original.belongs_to(&snapshot));
        assert!(matches!(
            project.worker(snapshot),
            Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
        ));
        assert_eq!(
            block_on(original.module_operands())
                .unwrap()
                .decoded_requests()
                .collect::<Vec<_>>(),
            ["./child.ts", "./second.vue"]
        );
        assert!(Arc::ptr_eq(&original, &worker(&project)));
        assert_eq!(original.counts(), (1, 1));
    }
}
