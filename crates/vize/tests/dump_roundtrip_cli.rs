//! Source-built CLI checks: real level APIs, bytes, usage and failure exits.

#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std format")]

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use vize_davinci::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Span};
use vize_l2::dump::Page as L2Page;
use vize_l3::dump::Page as L3Page;
use vize_l3::op::{Op, OpId, OpKind, Phase, Program, Region, RegionId};

const L2_FULL: &str = include_str!("../../vize_l2/tests/fixtures/reference.folio");

// Complete bodies captured from the actual source-built CLI. The only transport
// mapping is the one authored temporary input path; all error text stays exact.
fn expected_path_error(template: &str, path: &Path) -> Vec<u8> {
    assert_eq!(template.matches("@INPUT_PATH@").count(), 1);
    template
        .replace("@INPUT_PATH@", path.to_str().unwrap())
        .into_bytes()
}

fn invoke(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(args)
        .output()
        .unwrap()
}

fn roundtrip(level: &str, path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["dump", "--level", level, "--roundtrip"])
        .arg(path)
        .output()
        .unwrap()
}

fn assert_success(level: &str, bytes: &[u8]) {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("input.dump");
    fs::write(&path, bytes).unwrap();
    let output = roundtrip(level, &path);
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(output.stderr, b"");
    assert_eq!(
        output.stdout,
        format!(
            "dump: {level} roundtrip OK: {} ({} bytes)\n",
            path.display(),
            bytes.len()
        )
        .as_bytes(),
    );
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

fn l3_page() -> L3Page {
    let allocator = Allocator::default();
    let mut program = Program::new(&allocator, Phase::Built);
    program.push_region(Region::root(Span::new(0, 12)));
    program.push_op(Op::new(
        OpId::new(0),
        OpKind::SetText,
        RegionId::ROOT,
        Span::new(0, 12),
    ));
    L3Page::of(&program)
}

#[test]
fn l1_preserves_unicode_crlf_empty_and_recoverable_source() {
    for input in [
        "<div>日本語 🌈 {{ value }}</div>\r\n",
        "",
        "<div title=\"unterminated",
    ] {
        assert_success("l1", input.as_bytes());
    }
    let allocator = Allocator::default();
    let (_, errors) = vize_l1::parse(&allocator, "<div title=\"unterminated");
    assert!(
        !errors.is_empty(),
        "fidelity success does not mean syntax is valid"
    );
}

#[test]
fn l2_committed_full_fixture_and_l3_live_page_roundtrip() {
    assert_success("l2", L2_FULL.as_bytes());
    assert_success("l3", l3_page().print_to_string(DumpMode::Full).as_bytes());
}

#[test]
fn noncanonical_input_reports_the_first_differing_line_without_rewriting() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("noncanonical.dump");
    let input = L2_FULL.replacen("ops=13", "ops=999", 1);
    fs::write(&path, &input).unwrap();
    let output = roundtrip("l2", &path);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        format!("dump: {}: l2 roundtrip mismatch starting at line 2 (input {} bytes, printed {} bytes)\n", path.display(), input.len(), L2_FULL.len()).as_bytes(),
    );
    assert_eq!(fs::read(&path).unwrap(), input.as_bytes());
}

#[test]
fn missing_canonical_newline_is_a_byte_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("newline.dump");
    let canonical = l3_page().print_to_string(DumpMode::Full);
    let input = canonical.strip_suffix('\n').unwrap();
    fs::write(&path, input).unwrap();
    let output = roundtrip("l3", &path);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        format!("dump: {}: l3 roundtrip mismatch starting at line {} (input {} bytes, printed {} bytes)\n", path.display(), input.split('\n').count() + 1, input.len(), canonical.len()).as_bytes(),
    );
    assert_eq!(fs::read(&path).unwrap(), input.as_bytes());
}

#[test]
fn malformed_wrong_level_and_elided_l2_display_are_failures() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("invalid.dump");
    let display = L2Page::parse(L2_FULL)
        .unwrap()
        .print_to_string(DumpMode::Display);
    for (level, input, expected) in [
        (
            "l2",
            "not a dump",
            include_str!("fixtures/dump_cli/malformed-v2.stderr"),
        ),
        (
            "l3",
            L2_FULL,
            include_str!("fixtures/dump_cli/wrong-level-v2.stderr"),
        ),
        (
            "l2",
            display.as_str(),
            include_str!("fixtures/dump_cli/elided.stderr"),
        ),
    ] {
        fs::write(&path, input).unwrap();
        let output = roundtrip(level, &path);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, expected_path_error(expected, &path));
        assert_eq!(fs::read(&path).unwrap(), input.as_bytes());
    }
}

#[test]
fn unreadable_and_non_utf8_files_fail_without_success_output() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("input.dump");
    for (bytes, expected) in [
        (None, include_str!("fixtures/dump_cli/missing-file.stderr")),
        (
            Some([0xff]),
            include_str!("fixtures/dump_cli/non-utf8.stderr"),
        ),
    ] {
        if let Some(bytes) = bytes {
            fs::write(&path, bytes).unwrap();
        }
        let output = roundtrip("l1", &path);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, expected_path_error(expected, &path));
    }
}

#[test]
fn invalid_options_and_incomplete_modes_are_rejected() {
    for (args, expected) in [
        (
            vec!["dump"],
            include_bytes!("fixtures/dump_cli/required-both.stderr").as_slice(),
        ),
        (
            vec!["dump", "--roundtrip", "missing"],
            include_bytes!("fixtures/dump_cli/required-level.stderr").as_slice(),
        ),
        (
            vec!["dump", "--level", "l1"],
            include_bytes!("fixtures/dump_cli/required-roundtrip.stderr").as_slice(),
        ),
        (
            vec!["dump", "--level", "l0", "--roundtrip", "missing"],
            include_bytes!("fixtures/dump_cli/l0.stderr").as_slice(),
        ),
        (
            vec!["dump", "--level", "l4", "--roundtrip", "missing"],
            include_bytes!("fixtures/dump_cli/l4.stderr").as_slice(),
        ),
        (
            vec![
                "dump",
                "--level",
                "l2",
                "--roundtrip",
                "missing",
                "--pipeline",
                "l2()",
            ],
            include_bytes!("fixtures/dump_cli/pipeline.stderr").as_slice(),
        ),
        (
            vec!["dump", "--all-levels", "--json"],
            include_bytes!("fixtures/dump_cli/all-levels.stderr").as_slice(),
        ),
    ] {
        let output = invoke(&args);
        assert_eq!(output.status.code(), Some(2), "{:?}", output);
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, expected);
    }
}

#[test]
fn help_describes_both_dump_modes_and_exits_zero() {
    let output = invoke(&["dump", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    assert_eq!(
        output.stdout,
        include_bytes!("fixtures/dump_cli/help.stdout")
    );
}
