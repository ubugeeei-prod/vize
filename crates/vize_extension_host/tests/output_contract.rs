//! Host acceptance of the output-target document, without a guest.

use vize_davinci::folio::{Folio, FolioMode};
use vize_extension_host::contract::{Page, Span};
use vize_extension_host::output::{
    EmitDocument, EmitError, EmitLink, EmitRequest, Emitted, accept_emitted,
};
use vize_s0::String;

fn request() -> EmitRequest {
    EmitRequest {
        s2: Page {
            schema_version: 1,
            text: String::from("template\n"),
        },
        s3: Page {
            schema_version: 1,
            text: String::from("ops\n"),
        },
    }
}

fn document() -> EmitDocument {
    EmitDocument {
        text: String::from("return \"ok\";\n"),
        links: vec![EmitLink {
            generated: Span { start: 0, end: 12 },
            authored: Span { start: 0, end: 2 },
            name: Some(String::from("ret")),
            segment: true,
        }],
    }
}

fn page_of(document: &EmitDocument) -> Page {
    Page {
        schema_version: 1,
        text: document.print_to_string(FolioMode::Full),
    }
}

#[test]
fn the_committed_document_is_canonical() {
    let text = include_str!("fixtures/output/probe.emit.folio");
    let parsed = EmitDocument::parse(text).expect("parses");
    assert_eq!(parsed.print_to_string(FolioMode::Full), text);
    let accepted = accept_emitted(
        &request(),
        Emitted {
            document: Page {
                schema_version: 1,
                text: String::from(text),
            },
            diagnostics: Vec::new(),
        },
    )
    .expect("accepts");
    assert_eq!(accepted.document, parsed);
}

#[test]
fn a_wrong_schema_is_refused() {
    let mut emitted = Emitted {
        document: page_of(&document()),
        diagnostics: Vec::new(),
    };
    emitted.document.schema_version = 2;
    let error = accept_emitted(&request(), emitted).expect_err("schema");
    assert_eq!(
        error.to_string(),
        "emit-document-page schema version 2 is unreadable: this host reads version 1"
    );
}

#[test]
fn a_non_canonical_page_is_refused() {
    let mut text = page_of(&document());
    text.text = text.text.replacen("\n\n", "\n", 1).into();
    let error = accept_emitted(
        &request(),
        Emitted {
            document: text,
            diagnostics: Vec::new(),
        },
    )
    .expect_err("canonical");
    assert!(
        error.to_string().starts_with(
            "emit-document-page is not canonical: it differs from its reprint at byte "
        ),
        "{error}"
    );
}

#[test]
fn a_generated_range_past_the_text_is_refused() {
    let mut document = document();
    document.links[0].generated.end = 99;
    let error = accept_emitted(
        &request(),
        Emitted {
            document: page_of(&document),
            diagnostics: Vec::new(),
        },
    )
    .expect_err("range");
    assert_eq!(
        error,
        EmitError::GeneratedRange {
            index: 0,
            span: Span { start: 0, end: 99 },
        }
    );
}

#[test]
fn a_diagnostic_outside_the_authored_links_is_refused() {
    use vize_extension_host::{Diagnostic, Severity, Stage};
    let error = accept_emitted(
        &request(),
        Emitted {
            document: page_of(&document()),
            diagnostics: vec![Diagnostic {
                severity: Severity::Error,
                stage: Stage::Emit,
                span: Span { start: 9, end: 10 },
                message: String::from("no"),
                parts: Vec::new(),
                witness: None,
            }],
        },
    )
    .expect_err("diagnostic");
    assert_eq!(
        error.to_string(),
        "diagnostic 0 span 9:10 lies outside every authored link"
    );
}
