//! `ReproFolio` round-trip laws and exact parse rejections (P2-13).
//!
//! The `[repro]` page is hand-written (its artifact section is verbatim and
//! terminal - a semantic decision the derive refuses to make), so it carries
//! its own TS-16-shaped suite: byte-exact `print(parse(t)) == t` for
//! canonical text, structural `parse(print(v)) == v` over normalized values,
//! normalization-by-first-print for scrambled input, and every rejection
//! asserted on the exact `FolioError`.

use vize_davinci::folio::repro::{ReproFolio, failure_text};
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_s0::{FxHashMap, String};

const CANONICAL: &str = "[repro]\n\
pipeline=template(transform,codegen)\n\
failed-stage=template\n\
failed-pass=transform\n\
reason=injected davinci panic in pass `transform`\n\
artifact-stage=source\n\
\n\
[repro.config]\n\
inject-panic=transform\n\
mode=dom\n\
\n\
[repro.artifact]\n\
<template><div>b</div></template>\n";

fn canonical_value() -> ReproFolio {
    let mut config: FxHashMap<String, String> = FxHashMap::default();
    config.insert(String::from("mode"), String::from("dom"));
    config.insert(String::from("inject-panic"), String::from("transform"));
    ReproFolio {
        pipeline: String::from("template(transform,codegen)"),
        failed_stage: String::from("template"),
        failed_pass: String::from("transform"),
        reason: String::from("injected davinci panic in pass `transform`"),
        artifact_stage: String::from("source"),
        config,
        artifact: String::from("<template><div>b</div></template>\n"),
    }
}

#[test]
fn canonical_text_round_trips_byte_exactly() {
    let folio = ReproFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(folio, canonical_value());
}

#[test]
fn display_prints_the_full_text() {
    // A fact about this page, not a law: nothing on it is a span or a
    // default, so Display has nothing to elide.
    let folio = ReproFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(
        folio.print_to_string(FolioMode::Display),
        folio.print_to_string(FolioMode::Full)
    );
}

#[test]
fn values_round_trip_structurally_after_normalize() {
    let mut value = canonical_value();
    value.artifact = String::from("<template><div>b</div></template>");
    value.normalize();
    let reparsed = ReproFolio::parse(value.print_to_string(FolioMode::Full).as_str())
        .expect("printed text parses");
    assert_eq!(reparsed, value);
}

#[test]
fn scrambled_scalars_and_config_normalize_on_first_print() {
    let scrambled = "[repro]\n\
reason=injected davinci panic in pass `transform`\n\
artifact-stage=source\n\
pipeline=template(transform,codegen)\n\
failed-pass=transform\n\
failed-stage=template\n\
\n\
[repro.config]\n\
mode=dom\n\
inject-panic=transform\n\
\n\
[repro.artifact]\n\
<template><div>b</div></template>\n";
    let folio = ReproFolio::parse(scrambled).expect("scrambled text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
}

#[test]
fn the_artifact_section_is_verbatim_and_terminal() {
    // Folio-looking lines, blank lines, and header-looking lines inside the
    // artifact are content, not structure.
    let artifact = "[repro]\n\npipeline=not-a-field\n\n[weird]\nlast line\n";
    let mut value = canonical_value();
    value.config = FxHashMap::default();
    value.artifact = String::from(artifact);
    let printed = value.print_to_string(FolioMode::Full);
    let reparsed = ReproFolio::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(reparsed.artifact.as_str(), artifact);
    assert_eq!(reparsed, value);
    assert_eq!(reparsed.print_to_string(FolioMode::Full), printed);
}

#[test]
fn an_empty_failed_pass_prints_and_parses() {
    let mut value = canonical_value();
    value.failed_pass = String::default();
    value.config = FxHashMap::default();
    let printed = value.print_to_string(FolioMode::Full);
    let reparsed = ReproFolio::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(reparsed, value);
}

#[test]
fn failure_text_marks_an_unattributable_pass_with_a_question_mark() {
    assert_eq!(
        failure_text("template", "transform", "boom").as_str(),
        "template.transform: boom"
    );
    assert_eq!(
        failure_text("template", "", "boom").as_str(),
        "template.?: boom"
    );
    let folio = ReproFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(
        folio.failure().as_str(),
        "template.transform: injected davinci panic in pass `transform`"
    );
}

#[test]
fn rejections_carry_the_exact_error() {
    let missing_reason = "[repro]\n\
pipeline=template(transform,codegen)\n\
failed-stage=template\n\
failed-pass=transform\n\
artifact-stage=source\n";
    for (input, expected) in [
        (
            "x\n",
            FolioError::new(1, String::from("content before the [repro] header")),
        ),
        (
            "",
            FolioError::new(0, String::from("missing [repro] header")),
        ),
        (
            "[vir]\n",
            FolioError::new(1, String::from("first section must be [repro]")),
        ),
        (
            "[repro]\nbogus=1\n",
            FolioError::new(2, String::from("unknown field `bogus`")),
        ),
        (
            "[repro]\npipeline=s2()\npipeline=s2()\n",
            FolioError::new(3, String::from("duplicate field `pipeline`")),
        ),
        (
            "[repro]\nno-equals-here\n",
            FolioError::new(2, String::from("field line is missing `=`")),
        ),
        (
            missing_reason,
            FolioError::new(0, String::from("missing field `reason`")),
        ),
        (
            "[repro]\n\n[repro.bogus]\n",
            FolioError::new(3, String::from("unknown section [repro.bogus]")),
        ),
        (
            "[repro]\n\n[repro.config]\na=1\n\n[repro.config]\n",
            FolioError::new(6, String::from("duplicate section [repro.config]")),
        ),
        (
            "[repro]\n\n[repro.config]\na=1\na=2\n",
            FolioError::new(5, String::from("duplicate map key `a`")),
        ),
    ] {
        let error = ReproFolio::parse(input).expect_err("malformed input is rejected");
        assert_eq!(error, expected, "input: {input:?}");
    }
}

#[test]
fn an_unparseable_pipeline_is_rejected_with_the_grammar_error() {
    let input = "[repro]\n\
pipeline=Nope\n\
failed-stage=template\n\
failed-pass=\n\
reason=boom\n\
artifact-stage=source\n";
    let error = ReproFolio::parse(input).expect_err("invalid pipeline is rejected");
    assert_eq!(
        error,
        FolioError::new(
            0,
            String::from("invalid pipeline `Nope`: unexpected character `N` at offset 0")
        )
    );
}
