#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, Location, NumberOrString, Position, Range,
    TextDocumentContentChangeEvent, Url,
};

use super::super::DocumentStore;
use crate::ide::diagnostics::NativeFamily;

fn uri() -> Url {
    Url::parse("file:///carried.vue").unwrap()
}

fn range(start: (u32, u32), end: (u32, u32)) -> Range {
    Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1))
}

fn diagnostic(range: Range, message: &str) -> Diagnostic {
    Diagnostic {
        range,
        message: message.into(),
        ..Default::default()
    }
}

fn edit(range: Range, text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: Some(range),
        range_length: None,
        text: text.into(),
    }
}

fn whole(text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
    }
}

/// Carry `diagnostics` at version 1, apply each version's changes, and
/// return the moved ranges by message.
fn carried(
    text: &str,
    diagnostics: Vec<Diagnostic>,
    versions: Vec<Vec<TextDocumentContentChangeEvent>>,
) -> Vec<(String, Range)> {
    let store = DocumentStore::new();
    store.open(uri(), text.into(), 1, "vue".into());
    let family = NativeFamily {
        diagnostics,
        owns_script_syntax: true,
    };
    assert!(store.carry_native(&uri(), 1, family));
    let edited = !versions.is_empty();
    let mut version = 1;
    for changes in versions {
        version += 1;
        assert!(store.apply_changes(&uri(), changes, version));
    }
    let family = store.carried_native(&uri(), version).unwrap();
    // A moved family no longer describes the current script syntax (#11626).
    assert_eq!(family.owns_script_syntax, !edited);
    family
        .diagnostics
        .into_iter()
        .map(|diagnostic| (diagnostic.message, diagnostic.range))
        .collect()
}

#[test]
fn ranges_move_through_inserts_and_deletes_around_them() {
    let text = "const a = 1;\nconst broken: number = 'x';\nconst c = 3;\n";
    let broken = range((1, 6), (1, 12));
    let moved = carried(
        text,
        vec![diagnostic(broken, "broken")],
        vec![
            // A line above moves it down.
            vec![edit(range((0, 0), (0, 0)), "// note\n")],
            // An indent before it on its line moves it right.
            vec![edit(range((2, 0), (2, 0)), "  ")],
            // Edits after it on its line and below it leave it.
            vec![edit(range((2, 29), (2, 29)), " // x")],
            vec![edit(range((3, 0), (4, 0)), "")],
            // A deletion ending where it starts pulls it left.
            vec![edit(range((2, 0), (2, 2)), "")],
        ],
    );
    assert_eq!(moved, vec![("broken".into(), range((2, 6), (2, 12)))]);
}

#[test]
fn an_edit_overlapping_a_range_drops_only_that_diagnostic() {
    let text = "const a: number = 'x';\nconst b: number = 'y';\n";
    let moved = carried(
        text,
        vec![
            diagnostic(range((0, 6), (0, 7)), "a"),
            diagnostic(range((1, 6), (1, 7)), "b"),
            diagnostic(range((0, 0), (0, 0)), "file"),
        ],
        vec![vec![edit(range((0, 6), (0, 7)), "renamed")]],
    );
    assert_eq!(
        moved,
        vec![
            ("b".into(), range((1, 6), (1, 7))),
            ("file".into(), range((0, 0), (0, 0))),
        ]
    );
}

#[test]
fn several_changes_in_one_notification_apply_in_order() {
    let text = "a\nbroken\nc\n";
    let moved = carried(
        text,
        vec![diagnostic(range((1, 0), (1, 6)), "broken")],
        vec![vec![
            edit(range((2, 0), (2, 1)), "cc"),
            edit(range((0, 0), (0, 0)), "x\ny\n"),
            edit(range((3, 0), (3, 0)), "  "),
        ]],
    );
    assert_eq!(moved, vec![("broken".into(), range((3, 2), (3, 8)))]);
}

#[test]
fn a_whole_text_replacement_moves_ranges_outside_the_changed_bytes() {
    let text = "head\nconst broken = 1;\ntail;\nlast\n";
    let replaced = "head\r\nconst broken = 1;\ntail😀;\nlast\n";
    let moved = carried(
        text,
        vec![
            diagnostic(range((0, 0), (0, 4)), "head"),
            diagnostic(range((1, 6), (1, 12)), "crossed"),
            diagnostic(range((2, 0), (2, 4)), "tail"),
            diagnostic(range((3, 0), (3, 4)), "last"),
        ],
        vec![vec![whole(replaced)]],
    );
    // Only the window between the first and last changed byte is dropped.
    assert_eq!(
        moved,
        vec![
            ("head".into(), range((0, 0), (0, 4))),
            ("last".into(), range((3, 0), (3, 4))),
        ]
    );
}

#[test]
fn utf16_columns_follow_surrogate_pairs_and_crlf_lines() {
    let text = "<p>😀</p>\r\nconst broken = 1;\r\n";
    let moved = carried(
        text,
        vec![diagnostic(range((1, 6), (1, 12)), "broken")],
        vec![
            vec![edit(range((1, 0), (1, 0)), "/*🧭*/")],
            vec![edit(range((0, 3), (0, 5)), "東京🧭")],
        ],
    );
    assert_eq!(moved, vec![("broken".into(), range((1, 12), (1, 18)))]);
}

#[test]
fn carried_families_belong_to_one_open_version() {
    let store = DocumentStore::new();
    store.open(uri(), "const a = 1;\n".into(), 1, "vue".into());
    let family = |message| NativeFamily {
        diagnostics: vec![diagnostic(range((0, 6), (0, 7)), message)],
        owns_script_syntax: false,
    };
    assert!(!store.carry_native(&uri(), 2, family("stale")));
    assert!(store.carried_native(&uri(), 1).is_none());
    assert!(store.carry_native(&uri(), 1, family("current")));
    assert!(store.carried_native(&uri(), 2).is_none());
    assert_eq!(
        store.carried_native(&uri(), 1).unwrap().diagnostics,
        vec![diagnostic(range((0, 6), (0, 7)), "current")]
    );

    // Related locations in this document would not move with it.
    let mut related = diagnostic(range((0, 0), (0, 1)), "related");
    related.related_information = Some(vec![DiagnosticRelatedInformation {
        location: Location::new(uri(), range((0, 6), (0, 7))),
        message: "here".into(),
    }]);
    let family = NativeFamily {
        diagnostics: vec![related, diagnostic(range((0, 6), (0, 7)), "kept")],
        owns_script_syntax: false,
    };
    assert!(store.carry_native(&uri(), 1, family));
    assert_eq!(
        store.carried_native(&uri(), 1).unwrap().diagnostics,
        vec![diagnostic(range((0, 6), (0, 7)), "kept")]
    );

    store.close(&uri());
    store.open(uri(), "const a = 1;\n".into(), 1, "vue".into());
    assert!(store.carried_native(&uri(), 1).is_none());
}

/// #11626: a family computed on an earlier text cannot own the current text's
/// script syntax, so the prompt set keeps the script parser's fresh errors.
#[test]
fn carried_native_family_does_not_own_the_edited_script_syntax() {
    let store = DocumentStore::new();
    store.open(uri(), "const a: number = 1;\n".into(), 1, "vue".into());
    let coded = |range, message: &str, code| Diagnostic {
        code: Some(NumberOrString::Number(code)),
        ..diagnostic(range, message)
    };
    let family = NativeFamily {
        diagnostics: vec![
            coded(range((0, 6), (0, 7)), "type", 2322),
            coded(range((0, 18), (0, 19)), "syntax", 1109),
        ],
        owns_script_syntax: true,
    };
    assert!(store.carry_native(&uri(), 1, family));
    assert_eq!(
        store.carried_native(&uri(), 1).unwrap().owns_script_syntax,
        true,
        "the family owns the text it was computed on"
    );
    assert!(store.apply_changes(&uri(), vec![edit(range((1, 0), (1, 0)), "const = ;\n")], 2));
    let carried = store.carried_native(&uri(), 2).unwrap();
    assert_eq!(carried.owns_script_syntax, false);
    // Its syntax error described the old text; the type error stays.
    assert_eq!(
        carried.diagnostics,
        vec![coded(range((0, 6), (0, 7)), "type", 2322)]
    );
}
