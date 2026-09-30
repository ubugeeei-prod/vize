//! Published schema1 acceptance keeps its old bytes and refusal envelopes.

use vize_davinci::dump::{Dump, Error, Mode};
use vize_extension_host::accept::{AcceptError, accept};
use vize_extension_host::{LoweredBlock, Page, SourceBlock};
use vize_l0::{String, cstr};
use vize_l2::dump::Page as CurrentPage;

const OLD: &str = "[disegno]\nops=0\n\n";
const CURRENT: &str = "[l2-dump-v2]\nops=0\n\n";

fn block() -> SourceBlock {
    SourceBlock {
        source: String::default(),
        base: 0,
        lang: None,
    }
}

fn answer() -> LoweredBlock {
    LoweredBlock {
        surface: Page {
            schema_version: 1,
            text: String::from("[s1]\nbytes=0\n\n"),
        },
        semantic: Page {
            schema_version: 1,
            text: String::from(OLD),
        },
        diagnostics: vec![],
    }
}

#[test]
fn historical_wire_returns_the_same_current_ir_without_rewriting_bytes() {
    let raw = answer();
    let accepted = accept(&block(), raw.clone()).unwrap();
    assert_eq!(accepted.lowered, raw);
    assert_eq!(accepted.semantic, CurrentPage::default());
    assert_eq!(
        accepted.semantic.print_to_string(Mode::Full).as_str(),
        CURRENT
    );
    assert_eq!(
        serde_json::to_string(&accepted.lowered).unwrap(),
        r#"{"surface":{"schema-version":1,"text":"[s1]\nbytes=0\n\n"},"semantic":{"schema-version":1,"text":"[disegno]\nops=0\n\n"},"diagnostics":[]}"#
    );
}

#[test]
fn schema_refusal_precedes_parsing_and_current_text_is_not_a_v1_alias() {
    let mut raw = answer();
    raw.semantic.schema_version = 2;
    raw.semantic.text = String::from("not a page");
    assert_eq!(
        accept(&block(), raw),
        Err(AcceptError::UnreadableSchema {
            page: "s2-page",
            found: 2,
            reads: 1,
        })
    );
    let mut raw = answer();
    raw.semantic.text = String::from(CURRENT);
    let error = accept(&block(), raw).unwrap_err();
    assert_eq!(
        error,
        AcceptError::Parse {
            page: "s2-page",
            error: Error::new(1, cstr!("first section must be [disegno]")),
        }
    );
    assert_eq!(
        cstr!("{error}"),
        "s2-page: folio parse error at line 1: first section must be [disegno]"
    );
}

#[test]
fn canonical_byte_offset_and_line_zero_error_keep_the_old_contract() {
    let mut raw = answer();
    raw.semantic.text.push('\n');
    assert_eq!(
        accept(&block(), raw),
        Err(AcceptError::NotCanonical {
            page: "s2-page",
            at: OLD.len(),
        })
    );
    let mut raw = answer();
    raw.semantic.text = String::default();
    let error = accept(&block(), raw).unwrap_err();
    assert_eq!(
        error,
        AcceptError::Parse {
            page: "s2-page",
            error: Error::new(0, cstr!("missing [disegno] header")),
        }
    );
    assert_eq!(
        cstr!("{error}"),
        "s2-page: folio parse error: missing [disegno] header"
    );
}
