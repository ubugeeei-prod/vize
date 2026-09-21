//! P4-2: the α/β split — `import(export(t)) == t` for every α group, the
//! TS-16 `Full`-mode byte round trip of every versioned α page, and the
//! schema-doc check that fails on an undocumented group.
//!
//! `cargo test -p vize_davinci --test fact_alpha`

mod groups;

use groups::{Lengths, Longest, REGISTRY, Summary, Words, Words2};
use vize_davinci::fact::{
    ALPHA_GROUPS, AlphaDesc, AlphaDocError, AlphaDocument, AlphaExport, FactGroup, FactManager,
    FactTable, check_alpha_schema_doc, ids,
};
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_davinci::pass::AnalysisId;
use vize_s0::String;

const SCHEMA_DOC: &str = include_str!("../../../../davinci-road/plan/fact-alpha-schemas.md");

const ARTIFACTS: [&Words; 3] = [&["alpha", "be", "gamma", "δelta"], &["one"], &[]];

/// Export, print (`Full`), parse, import: the table survives exactly and
/// the page text survives byte-for-byte.
fn round_trip<G: AlphaExport>(table: &FactTable<G>) -> String {
    let document = AlphaDocument::<G>::export(table);
    let text = document.print_to_string(FolioMode::Full);
    let parsed = AlphaDocument::<G>::parse(&text).expect("canonical α text parses");
    assert_eq!(parsed.print_to_string(FolioMode::Full), text);
    assert!(parsed == document, "parse(print(v)) == v for {}", G::NAME);
    let imported = parsed.import();
    assert!(&imported == table, "import(export(t)) == t for {}", G::NAME);
    text
}

#[test]
fn every_alpha_group_round_trips_exactly() {
    for words in ARTIFACTS {
        let mut manager = FactManager::new(&REGISTRY);
        let view = manager.prepare::<Summary>(words).unwrap();
        round_trip::<Lengths>(view.get::<Lengths>().unwrap());
        round_trip::<Words2>(view.get::<Words2>().unwrap());
        round_trip::<Longest>(view.get::<Longest>().unwrap());
    }
}

#[test]
fn an_alpha_page_prints_its_group_and_schema_version() {
    let mut manager = FactManager::new(&REGISTRY);
    // `gamma` ties `alpha` for longest; `max_by_key` keeps the last.
    let view = manager
        .prepare::<Summary>(&["alpha", "be", "gamma"][..])
        .unwrap();
    assert_eq!(
        round_trip::<Words2>(view.get::<Words2>().unwrap()),
        "[fact-alpha]\ngroup=words\nschema_version=3\n\n\
         [words-alpha]\n\n\
         [words-alpha.words]\n0=alpha\n1=be\n2=gamma\n\n"
    );
    assert_eq!(
        round_trip::<Longest>(view.get::<Longest>().unwrap()),
        "[fact-alpha]\ngroup=longest\nschema_version=1\n\n\
         [longest-alpha]\npresent=true\nindex=2\n\n"
    );
}

#[test]
fn a_page_under_another_version_or_group_is_refused_exactly() {
    let text = "[fact-alpha]\ngroup=words\nschema_version=2\n\n[words-alpha]\n\n";
    assert_eq!(
        AlphaDocument::<Words2>::parse(text).map(|_| ()),
        Err(FolioError::new(
            3,
            String::from("expected `schema_version=3`, found `schema_version=2`")
        ))
    );
    let text = "[fact-alpha]\ngroup=lengths\nschema_version=3\n\n[words-alpha]\n\n";
    assert_eq!(
        AlphaDocument::<Words2>::parse(text).map(|_| ()),
        Err(FolioError::new(
            2,
            String::from("expected `group=words`, found `group=lengths`")
        ))
    );
    assert_eq!(
        AlphaDocument::<Words2>::parse("[fact-alpha]\n").map(|_| ()),
        Err(FolioError::new(
            2,
            String::from("expected `group=words`, found ``")
        ))
    );
}

#[test]
fn a_body_error_is_reported_at_its_document_line() {
    let text = "[fact-alpha]\ngroup=longest\nschema_version=1\n\n[longest-alpha]\npresent=maybe\n";
    assert_eq!(
        AlphaDocument::<Longest>::parse(text).map(|_| ()),
        Err(FolioError::new(6, String::from("invalid bool `maybe`")))
    );
}

#[test]
fn the_schema_doc_covers_every_registered_group() {
    assert_eq!(check_alpha_schema_doc(SCHEMA_DOC, ALPHA_GROUPS), Ok(()));
    // The planned P4-3a rows parse as the ids and versions the code will use.
    let planned = [
        AlphaDesc {
            id: ids::BINDINGS,
            name: "bindings",
            schema: 1,
        },
        AlphaDesc {
            id: ids::UNDEFINED_REFS,
            name: "undefined-refs",
            schema: 1,
        },
    ];
    assert_eq!(check_alpha_schema_doc(SCHEMA_DOC, &planned), Ok(()));
}

#[test]
fn the_schema_doc_check_fails_on_an_undocumented_group() {
    let injected = [Words2::ALPHA_DESC];
    assert_eq!(
        check_alpha_schema_doc(SCHEMA_DOC, &injected),
        Err(AlphaDocError::Undocumented { group: "words" })
    );
    let wrong_version = [AlphaDesc {
        id: ids::BINDINGS,
        name: "bindings",
        schema: 2,
    }];
    assert_eq!(
        check_alpha_schema_doc(SCHEMA_DOC, &wrong_version),
        Err(AlphaDocError::Mismatch {
            group: "bindings",
            documented: String::from("id=1 schema=1"),
        })
    );
    let wrong_id = [AlphaDesc {
        id: AnalysisId::new(9),
        name: "undefined-refs",
        schema: 1,
    }];
    assert_eq!(
        check_alpha_schema_doc(SCHEMA_DOC, &wrong_id),
        Err(AlphaDocError::Mismatch {
            group: "undefined-refs",
            documented: String::from("id=2 schema=1"),
        })
    );
}

/// α values are owned: every fixture page is `'static` (the trait bound),
/// and so is the document around it.
#[test]
fn alpha_documents_are_owned() {
    fn owned<T: 'static>() {}
    owned::<AlphaDocument<Lengths>>();
    owned::<AlphaDocument<Words2>>();
    owned::<AlphaDocument<Longest>>();
    assert_eq!(
        (Lengths::ALPHA_DESC.schema, Words2::ALPHA_DESC.schema),
        (1, 3)
    );
    assert_eq!(Longest::ALPHA_DESC.id, Longest::ID);
}
