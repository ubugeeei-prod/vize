use super::{
    DocumentStore, ModuleOperandRefusal, NativeNavigationProject, Position, SourceQueryProject,
    Url, block_on, range, uri, worker,
};

#[test]
fn module_link_original_import_reexport_order_and_occurrence_dedup_are_exact() {
    let documents = DocumentStore::new();
    let source = "export {a as x,b as y} from './first.ts';\nimport './same.ts';\nexport * from './third.ts';\nimport './same.ts';";
    documents.open(uri(), source.into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    let operands = block_on(worker.module_operands()).unwrap();
    assert_eq!(operands.len(), 4);
    assert_eq!(
        operands.decoded_requests().collect::<Vec<_>>(),
        ["./first.ts", "./same.ts", "./third.ts", "./same.ts"]
    );
    let targets = ["first.ts", "same.ts", "third.ts", "same.ts"]
        .map(|name| Url::parse(&vize_l0::cstr!("file:///observed/{name}")).unwrap());
    let links = operands.into_links(&targets).unwrap();
    assert_eq!(
        links.iter().map(|link| link.range).collect::<Vec<_>>(),
        [
            range((0, 28), (0, 40)),
            range((1, 7), (1, 18)),
            range((2, 14), (2, 26)),
            range((3, 7), (3, 18)),
        ]
    );
    assert_eq!(
        links
            .iter()
            .map(|link| link.target.as_ref().unwrap())
            .collect::<Vec<_>>(),
        targets.iter().collect::<Vec<_>>()
    );
    let before = block_on(worker.inspect(Position::new(1, 0))).unwrap();
    assert_eq!(before.statements, 4);
    assert_eq!(before.parses, 1);
    assert_eq!(block_on(worker.module_operands()).unwrap().len(), 4);
    assert_eq!(
        block_on(worker.inspect(Position::new(0, 0))).unwrap(),
        before
    );
    assert_eq!(worker.counts(), (1, 2));
}

#[test]
fn module_link_quoted_utf16_geometry_uses_authored_escapes_unicode_and_crlf() {
    let documents = DocumentStore::new();
    let source = "/*🦀*/ import './雪\\u002F🦀.ts';\r\nexport * from './雪\\\r\n🦀.ts';";
    documents.open(uri(), source.into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let operands = block_on(worker(&project).module_operands()).unwrap();
    assert_eq!(
        operands.decoded_requests().collect::<Vec<_>>(),
        ["./雪/🦀.ts", "./雪🦀.ts"]
    );
    let targets = [
        Url::parse("file:///one.ts").unwrap(),
        Url::parse("file:///two.ts").unwrap(),
    ];
    let links = operands.into_links(&targets).unwrap();
    assert_eq!(links[0].range, range((0, 14), (0, 30)));
    assert_eq!(links[1].range, range((1, 14), (2, 6)));
    assert_eq!(
        serde_json::to_value(&links).unwrap(),
        serde_json::json!([
            {"range":{"start":{"line":0,"character":14},"end":{"line":0,"character":30}},"target":"file:///one.ts"},
            {"range":{"start":{"line":1,"character":14},"end":{"line":2,"character":6}},"target":"file:///two.ts"}
        ])
    );
}

#[test]
fn module_link_ts_type_mixed_and_string_named_sources_are_original_operands() {
    let documents = DocumentStore::new();
    let source = "import type {T} from './types.ts'; export type {T as U,T as V} from './types.ts'; export {type T,V} from './mixed.ts'; export type * from './all.ts'; import {'remote/name' as local} from './actual.ts'; export {'original/name' as 'public/name'} from './remote.ts';";
    documents.open(uri(), source.into(), 1, "typescript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let operands = block_on(worker(&project).module_operands()).unwrap();
    assert_eq!(
        operands.decoded_requests().collect::<Vec<_>>(),
        [
            "./types.ts",
            "./types.ts",
            "./mixed.ts",
            "./all.ts",
            "./actual.ts",
            "./remote.ts"
        ]
    );
    assert_eq!(operands.len(), 6);
}

#[test]
fn module_link_valid_nonempty_program_without_sources_mints_empty_operands() {
    for source in [";", "const value=1;value;", "export default 1;"] {
        let documents = DocumentStore::new();
        documents.open(uri(), source.into(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let operands = block_on(worker(&project).module_operands()).unwrap();
        assert!(operands.is_empty());
        assert_eq!(operands.len(), 0);
        assert_eq!(operands.decoded_requests().count(), 0);
        assert_eq!(operands.into_links(&[]), Ok(Vec::new()));
    }
}

#[test]
fn module_link_decoded_request_is_metadata_without_guessed_target_policy() {
    let documents = DocumentStore::new();
    documents.open(uri(), "import ''; import 'package'; import '../parent'; import './extensionless'; import './a.ts?query';".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let operands = block_on(worker(&project).module_operands()).unwrap();
    assert_eq!(
        operands.decoded_requests().collect::<Vec<_>>(),
        [
            "",
            "package",
            "../parent",
            "./extensionless",
            "./a.ts?query"
        ]
    );
}

#[test]
fn module_link_target_cardinality_refuses_before_any_partial_response() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "import './a.ts'; import './b.ts';".into(),
        1,
        "javascript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    for count in [0, 1, 3] {
        let operands = block_on(worker.module_operands()).unwrap();
        let targets = vec![Url::parse("file:///target.ts").unwrap(); count];
        assert_eq!(
            operands.into_links(&targets),
            Err(ModuleOperandRefusal::TargetCardinality {
                operands: 2,
                targets: count
            })
        );
    }
}
