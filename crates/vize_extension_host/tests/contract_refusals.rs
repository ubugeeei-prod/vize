//! Every refusal the host can issue, with its exact message: capability
//! negotiation, `lang` routing, and acceptance of a guest's answer.

#![expect(
    clippy::expect_used,
    clippy::unreachable,
    reason = "tests assert by panicking"
)]

use vize_extension_host::vue::VueDialect;
use vize_extension_host::{
    Capability, ContractError, Diagnostic, GuestError, InputDialectGuest, LoweredBlock, Session,
    Severity, SourceBlock, Span, Stage,
};
use vize_s0::String;

/// A guest that offers `features` at `protocol_version` and answers every
/// block with the in-tree Vue answer passed through `tamper`.
#[derive(Debug)]
struct Stub {
    protocol_version: u32,
    features: &'static [&'static str],
    tamper: fn(&mut LoweredBlock),
}

impl Stub {
    fn offering(protocol_version: u32, features: &'static [&'static str]) -> Self {
        Self {
            protocol_version,
            features,
            tamper: |_| {},
        }
    }

    fn tampering(tamper: fn(&mut LoweredBlock)) -> Self {
        Self {
            tamper,
            ..Self::offering(1, &["lang:html", "s1-page@1", "s2-page@1"])
        }
    }
}

impl InputDialectGuest for Stub {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        Ok(Capability {
            protocol_version: self.protocol_version,
            features: self.features.iter().copied().map(String::from).collect(),
        })
    }

    fn lower_block(&mut self, block: &SourceBlock) -> Result<LoweredBlock, GuestError> {
        let mut lowered = VueDialect::default().lower_block(block)?;
        (self.tamper)(&mut lowered);
        Ok(lowered)
    }
}

fn handshake(stub: Stub) -> String {
    let error = Session::open(stub).expect_err("the offer must be refused");
    vize_s0::cstr!("{error}")
}

#[test]
fn capability_offers_are_refused_exactly() {
    assert_eq!(
        handshake(Stub::offering(2, &["s1-page@1", "s2-page@1"])),
        "handshake refused: protocol version mismatch: the host speaks 1, the guest offered 2"
    );
    assert_eq!(
        handshake(Stub::offering(0, &[])),
        "handshake refused: protocol version mismatch: the host speaks 1, the guest offered 0"
    );
    assert_eq!(
        handshake(Stub::offering(1, &["s2-page@1", "s1-page@1"])),
        "handshake refused: features must be sorted and unique: feature 1 \"s1-page@1\" does not sort after \"s2-page@1\""
    );
    assert_eq!(
        handshake(Stub::offering(1, &["s1-page@1", "s1-page@1", "s2-page@1"])),
        "handshake refused: features must be sorted and unique: feature 1 \"s1-page@1\" does not sort after \"s1-page@1\""
    );
    assert_eq!(
        handshake(Stub::offering(1, &["s1-page@1"])),
        "handshake refused: the guest does not offer required feature \"s2-page@1\""
    );
    assert_eq!(
        handshake(Stub::offering(1, &["lang:svelte", "s2-page@1"])),
        "handshake refused: the guest does not offer required feature \"s1-page@1\""
    );
}

#[test]
fn unknown_features_are_additive() {
    let session = Session::open(Stub::offering(
        1,
        &[
            "cache-key@2",
            "lang:svelte",
            "s1-page@1",
            "s2-page@1",
            "zzz",
        ],
    ))
    .expect("unknown features never refuse");
    assert_eq!(session.negotiated().langs, [String::from("svelte")]);
    assert_eq!(
        session.negotiated().ignored,
        [String::from("cache-key@2"), String::from("zzz")]
    );
}

const SOURCE: &str = "<p :title=\"t\">{{ msg }}</p>";

fn answer(stub: Stub, lang: Option<&str>) -> Result<(), String> {
    let block = SourceBlock {
        source: String::from(SOURCE),
        base: 100,
        lang: lang.map(String::from),
    };
    let mut session = Session::open(stub).expect("the offer negotiates");
    session
        .lower_block(&block)
        .map(|_| ())
        .map_err(|error| vize_s0::cstr!("{error}"))
}

#[test]
fn the_untampered_answer_is_accepted() {
    assert_eq!(answer(Stub::tampering(|_| {}), None), Ok(()));
    assert_eq!(answer(Stub::tampering(|_| {}), Some("html")), Ok(()));
}

#[test]
fn undeclared_langs_never_reach_the_guest() {
    let stub = Stub {
        tamper: |_| panic!("the guest must not be called"),
        ..Stub::tampering(|_| {})
    };
    assert_eq!(
        answer(stub, Some("pug")),
        Err(String::from(
            "the guest does not declare feature \"lang:pug\""
        ))
    );
}

fn refused(tamper: fn(&mut LoweredBlock)) -> String {
    answer(Stub::tampering(tamper), None).expect_err("the answer must be refused")
}

#[test]
fn answers_are_refused_exactly() {
    assert_eq!(
        refused(|lowered| lowered.surface.schema_version = 2),
        "answer refused: s1-page schema version 2 is unreadable: this host reads version 1"
    );
    assert_eq!(
        refused(|lowered| lowered.semantic.schema_version = 0),
        "answer refused: s2-page schema version 0 is unreadable: this host reads version 1"
    );
    assert_eq!(
        refused(|lowered| lowered.surface.text = String::from("[s1]\nbytes=x\n")),
        "answer refused: s1-page: folio parse error at line 2: invalid integer `x`"
    );
    assert_eq!(
        refused(|lowered| lowered.semantic.text.push('\n')),
        "answer refused: s2-page is not canonical: it differs from its reprint at byte 156"
    );
    assert_eq!(
        refused(|lowered| {
            lowered.surface.text = lowered.surface.text.replace("bytes=27", "bytes=027").into();
        }),
        "answer refused: s1-page is not canonical: it differs from its reprint at byte 11"
    );
    assert_eq!(
        refused(|lowered| {
            lowered.surface.text = lowered
                .surface
                .text
                .replace("gt 13:13:14", "gt 13:13:13")
                .into();
        }),
        "answer refused: s1-page does not tile the block: token 7 (`open`) starts at 14, expected 13"
    );
    assert_eq!(
        refused(|lowered| lowered.diagnostics.push(diagnostic(Span {
            start: 99,
            end: 101
        }))),
        "answer refused: diagnostic 0 span 99:101 lies outside the block 100:127"
    );
    assert_eq!(
        refused(|lowered| lowered.diagnostics.push(diagnostic(Span {
            start: 120,
            end: 128
        }))),
        "answer refused: diagnostic 0 span 120:128 lies outside the block 100:127"
    );
    assert_eq!(
        refused(|lowered| lowered.diagnostics.push(diagnostic(Span {
            start: 110,
            end: 109
        }))),
        "answer refused: diagnostic 0 span 110:109 lies outside the block 100:127"
    );
}

fn diagnostic(span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        stage: Stage::Surface,
        span,
        message: String::from("probe"),
        parts: Vec::new(),
        witness: None,
    }
}

#[test]
fn guest_failures_surface_verbatim() {
    #[derive(Debug)]
    struct Failing;
    impl InputDialectGuest for Failing {
        fn get_capability(&mut self) -> Result<Capability, GuestError> {
            Err(GuestError::Trap(String::from(
                "wasm `unreachable` instruction executed",
            )))
        }
        fn lower_block(&mut self, _: &SourceBlock) -> Result<LoweredBlock, GuestError> {
            unreachable!("never negotiated")
        }
    }
    let error = Session::open(Failing).expect_err("the call fails");
    assert_eq!(
        error,
        ContractError::Guest(GuestError::Trap(String::from(
            "wasm `unreachable` instruction executed"
        )))
    );
    assert_eq!(
        vize_s0::cstr!("{error}"),
        "guest trapped: wasm `unreachable` instruction executed"
    );
}
