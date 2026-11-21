//! Qualify selected diagnostic edits against the original versioned editor packet.
#![expect(
    clippy::unwrap_used,
    clippy::disallowed_types,
    clippy::panic,
    reason = "integration fixtures require real host documents and compare complete wire packets"
)]
use serde::Deserialize;
use std::{fs, path::Path};
use tower_lsp::lsp_types::{DocumentChanges, OneOf, TextDocumentIdentifier, Url, WorkspaceEdit};
use vize_l0::{
    SourceRoot, Span,
    line_index::{LineBreaks, utf16_len, utf16_offset},
};
use vize_l1::edit::{EditError, VersionedSource};
use vize_maestro::{
    document::DocumentStore,
    source_project::{SnapshotRefusal, SourceEditRefusal, SourceSnapshotCache},
};
use vize_patina::{JsxLang, LintPreset, LintResult, Linter};

mod profile;
mod terminators;

fn request(uri: &Url) -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: uri.clone() }
}

fn open(store: &DocumentStore, path: &str, source: &str) -> Url {
    let uri = Url::parse(path).unwrap();
    store.open(uri.clone(), source.into(), 7, "vue".into());
    uri
}

fn lint(linter: &Linter, entry: &str, source: &str, name: &str) -> LintResult {
    match entry {
        "sfc" => linter.lint_sfc(source, name),
        "html" => linter.lint_standalone_html(source, name),
        "jsx" => linter.lint_jsx(source, name, JsxLang::Jsx),
        "tsx" => linter.lint_jsx(source, name, JsxLang::Tsx),
        _ => panic!("unsupported fixture entry {entry}"),
    }
}

fn apply_wire(source: &str, edits: &WorkspaceEdit, uri: &Url, version: i32) -> String {
    let Some(DocumentChanges::Edits(changes)) = &edits.document_changes else {
        panic!("one versioned document change expected");
    };
    let [document] = changes.as_slice() else {
        panic!("one original document expected");
    };
    assert_eq!(&document.text_document.uri, uri);
    assert_eq!(document.text_document.version, Some(version));
    let starts: Vec<_> = LineBreaks::Lsp.line_starts(source).collect();
    // Editors clamp columns beyond line content to the start of its terminator.
    let offset = |position: tower_lsp::lsp_types::Position| {
        let line = position.line as usize;
        let Some(&from) = starts.get(line) else {
            panic!("wire line {line} outside the document");
        };
        let to = starts.get(line + 1).copied().unwrap_or(source.len());
        let text = source.get(from..to).unwrap().trim_end_matches(['\r', '\n']);
        from + utf16_offset(text, position.character.min(utf16_len(text) as u32)).unwrap()
    };
    let mut result = source.to_owned();
    for edit in document.edits.iter().rev() {
        let OneOf::Left(edit) = edit else {
            panic!("plain text edit expected");
        };
        let start = edit.range.start;
        let end = edit.range.end;
        let lo = offset(start);
        let hi = offset(end);
        result.replace_range(lo..hi, &edit.new_text);
    }
    result
}

fn compare(source: &str, diagnostics: &LintResult, profile_id: Option<usize>) -> (String, usize) {
    let store = DocumentStore::new();
    let authored_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(diagnostics.filename.as_str());
    let uri = Url::from_file_path(authored_path).unwrap();
    let language = if diagnostics.filename.ends_with(".jsx") {
        "javascriptreact"
    } else if diagnostics.filename.ends_with(".tsx") {
        "typescriptreact"
    } else if diagnostics.filename.ends_with(".html") {
        "html"
    } else {
        "vue"
    };
    store.open(uri.clone(), source.into(), 7, language.into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&store, &uri)
        .unwrap();
    let frame = snapshot.versioned_source().unwrap();
    let candidates = diagnostics
        .diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .flat_map(|fix| &fix.edits)
        .map(|e| (Span::new(e.start, e.end), e.new_text.as_str()));
    let Some(selected) = frame.select_edits(candidates).unwrap() else {
        return (source.to_owned(), 0);
    };
    if let Some(index) = profile_id {
        profile::paired(
            &vize_l0::cstr!("original={index}"),
            &store,
            &snapshot,
            &selected,
        );
    }
    let set = selected.as_edit_set();
    let old = snapshot.workspace_edit(&store, set).unwrap();
    let new = snapshot
        .workspace_edit_for_fixes(&store, request(&uri), selected.as_edit_set())
        .unwrap();
    assert_eq!(new, old);
    assert_eq!(
        serde_json::to_vec(&new).unwrap(),
        serde_json::to_vec(&old).unwrap()
    );
    let output = apply_wire(source, &new, &uri, snapshot.version());
    assert_eq!(output, snapshot.apply_edits(&store, set).unwrap().as_str());
    assert_eq!(store.text(&uri).unwrap(), source);
    (output, set.edits().len())
}

#[derive(Deserialize)]
struct Case {
    id: String,
    entry: String,
    filename: String,
    source: String,
    fixed: String,
}

#[test]
fn original_cli_and_binding_fix_history_publishes_identical_editor_packets_and_bytes() {
    for (rule, fixture) in [
        (
            "vue/no-boolean-attr-value",
            include_str!("../../../vize_patina/tests/fixtures/issue-7905/cases.json"),
        ),
        (
            "vue/v-slot-style",
            include_str!("../../../vize_patina/tests/fixtures/issue-7905-slot-style/cases.json"),
        ),
    ] {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![rule.into()]));
        for case in serde_json::from_str::<Vec<Case>>(fixture).unwrap() {
            let result = lint(&linter, &case.entry, &case.source, &case.filename);
            assert_eq!(
                compare(&case.source, &result, None).0,
                case.fixed,
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn each_published_fix_pass_matches_the_original_complete_document_on_its_actual_revision() {
    use tower_lsp::lsp_types::TextDocumentContentChangeEvent;
    const BEFORE: &str = include_str!(
        "../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.vue.txt"
    );
    const AFTER: &str = include_str!(
        "../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.fixed.vue.txt"
    );
    let store = DocumentStore::new();
    let uri = open(&store, "file:///workspace/TitleBox.vue", BEFORE);
    let cache = SourceSnapshotCache::default();
    let linter = Linter::with_preset(LintPreset::Opinionated);
    for _ in 0..10 {
        let snapshot = cache.capture(&store, &uri).unwrap();
        let source = snapshot.source();
        let result = linter.lint_sfc(source, "TitleBox.vue");
        let candidates = result
            .diagnostics
            .iter()
            .filter_map(|d| d.fix.as_ref())
            .flat_map(|fix| &fix.edits)
            .map(|e| (Span::new(e.start, e.end), e.new_text.as_str()));
        let Some(selected) = snapshot
            .versioned_source()
            .unwrap()
            .select_edits(candidates)
            .unwrap()
        else {
            break;
        };
        let packet = snapshot
            .workspace_edit_for_fixes(&store, request(&uri), selected.as_edit_set())
            .unwrap();
        assert_eq!(
            packet,
            snapshot
                .workspace_edit(&store, selected.as_edit_set())
                .unwrap()
        );
        let next = apply_wire(source, &packet, &uri, snapshot.version());
        assert_eq!(
            next,
            snapshot
                .apply_edits(&store, selected.as_edit_set())
                .unwrap()
                .as_str()
        );
        if next == source {
            break;
        }
        assert!(store.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: next,
            }],
            snapshot.version() + 1
        ));
        assert_eq!(
            snapshot.workspace_edit_for_fixes(&store, request(&uri), selected.as_edit_set()),
            Err(SourceEditRefusal::Host(SnapshotRefusal::Superseded))
        );
    }
    assert_eq!(store.text(&uri).unwrap(), AFTER);
}

#[test]
fn batch_order_and_foreign_stale_and_closed_documents_keep_the_original_outcomes() {
    let source = "α\r\n雪😀z";
    let store = DocumentStore::new();
    let uri = open(&store, "file:///workspace/first.vue", source);
    let other = open(&store, "file:///workspace/second.vue", source);
    let cache = SourceSnapshotCache::default();
    let snapshot = cache.capture(&store, &uri).unwrap();
    let foreign = cache.capture(&store, &other).unwrap();
    let frame = snapshot.versioned_source().unwrap();
    let selected = frame
        .select_edits([
            (Span::new(7, 11), "🌿"),
            (Span::new(4, 4), "一"),
            (Span::new(4, 4), "二"),
            (Span::new(4, 7), "三"),
            (Span::new(8, 9), "invalid"),
        ])
        .unwrap()
        .unwrap();
    let set = selected.as_edit_set();
    let wire = snapshot
        .workspace_edit_for_fixes(&store, request(&uri), selected.as_edit_set())
        .unwrap();
    assert_eq!(wire, snapshot.workspace_edit(&store, set).unwrap());
    assert_eq!(
        apply_wire(source, &wire, &uri, snapshot.version()),
        "α\r\n一二三🌿z"
    );
    assert_eq!(
        foreign.workspace_edit_for_fixes(&store, request(&other), selected.as_edit_set()),
        Err(SourceEditRefusal::Edit(EditError::SnapshotMismatch))
    );
    assert_eq!(
        snapshot.workspace_edit_for_fixes(&store, request(&other), set),
        Err(SourceEditRefusal::Edit(EditError::SnapshotMismatch))
    );
    let independent = SourceSnapshotCache::default()
        .capture(&store, &uri)
        .unwrap();
    assert_eq!(
        independent.workspace_edit_for_fixes(&store, request(&uri), set),
        Ok(wire)
    );
    let copied = source.to_owned();
    let detached = VersionedSource::new(
        frame.key(),
        frame.version(),
        SourceRoot::new(&copied).unwrap(),
    )
    .select_edits([(Span::new(0, 0), "!"), (Span::new(4, 4), "!")])
    .unwrap()
    .unwrap();
    assert_eq!(
        snapshot.workspace_edit_for_fixes(&store, request(&uri), detached.as_edit_set()),
        Err(SourceEditRefusal::Edit(EditError::SourceMismatch))
    );
    let old_version = VersionedSource::new(frame.key(), 6, frame.root())
        .select_edits([(Span::new(0, 0), "!"), (Span::new(4, 4), "!")])
        .unwrap()
        .unwrap();
    assert_eq!(
        snapshot.workspace_edit_for_fixes(&store, request(&uri), old_version.as_edit_set()),
        Err(SourceEditRefusal::Edit(EditError::StaleVersion))
    );
    store.close(&uri);
    assert_eq!(
        snapshot.workspace_edit_for_fixes(&store, request(&uri), selected.as_edit_set()),
        Err(SourceEditRefusal::Host(SnapshotRefusal::MissingDocument))
    );
    store.open(uri.clone(), source.into(), 7, "vue".into());
    assert_eq!(
        snapshot.workspace_edit_for_fixes(&store, request(&uri), selected.as_edit_set()),
        Err(SourceEditRefusal::Host(SnapshotRefusal::Superseded))
    );
}

#[test]
#[ignore = "requires a NUL-delimited list of original source files"]
fn original_lint_fixes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let paths = fs::read(std::env::var_os("VIZE_VERSIONED_FIX_INPUTS").unwrap()).unwrap();
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let timed = std::env::var_os("VIZE_VERSIONED_FIX_TIMINGS").is_some();
    let (mut sources, mut invalid, mut fixed, mut edits, mut bytes) = (0, 0, 0, 0, 0);
    for path in paths.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let name = std::str::from_utf8(path).unwrap().trim_end_matches(".txt");
        let entry = if name.ends_with(".vue") {
            "sfc"
        } else if name.ends_with(".html") {
            "html"
        } else if name.ends_with(".jsx") {
            "jsx"
        } else if name.ends_with(".tsx") {
            "tsx"
        } else {
            continue;
        };
        let file = fs::read(root.join(std::str::from_utf8(path).unwrap())).unwrap();
        sources += 1;
        let Ok(text) = std::str::from_utf8(&file) else {
            invalid += 1;
            continue;
        };
        let problems = lint(&linter, entry, text, name);
        let (result, selected) = compare(text, &problems, timed.then_some(sources));
        if selected > 0 {
            fixed += 1;
            edits += selected;
            bytes += result.len();
        }
    }
    assert!(fixed > 0 && edits >= fixed);
    eprintln!(
        "original sources={sources} invalid={invalid} fixed={fixed} applied edits={edits} exact output bytes={bytes}"
    );
}
