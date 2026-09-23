//! Shared test support: the golden cases, their committed files, and the
//! guest builder.

#![allow(dead_code)]

pub mod expression;
pub mod output;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use vize_extension_host::{
    Diagnostic, LoweredBlock, PartKind, Severity, SourceBlock, Stage, Witness,
};
use vize_s0::{String, cstr};

/// Set to rewrite the committed golden pages from the in-tree Vue dialect.
pub const BLESS_ENV: &str = "VIZE_EXTENSION_GOLDEN_BLESS";

/// One golden exchange: a committed block at a file offset.
pub struct Case {
    pub name: &'static str,
    pub base: u32,
    pub lang: Option<&'static str>,
}

/// Every golden case; the echo guest (`tests/guests/echo`) embeds the same
/// four, and `wit_golden` proves it knows no other.
pub const CASES: &[Case] = &[
    Case {
        name: "attributes",
        base: 10,
        lang: None,
    },
    Case {
        name: "control-flow",
        base: 0,
        lang: Some("html"),
    },
    Case {
        name: "recovery",
        base: 64,
        lang: None,
    },
    Case {
        name: "unicode",
        base: 1_000,
        lang: Some("html"),
    },
];

pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden")
}

fn read(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    String::from_utf8(bytes).expect("golden files are UTF-8")
}

impl Case {
    pub fn block(&self) -> SourceBlock {
        SourceBlock {
            source: read(&golden_dir().join(self.file("block").as_str())),
            base: self.base,
            lang: self.lang.map(String::from),
        }
    }

    pub fn file(&self, extension: &str) -> String {
        cstr!("{}.{extension}", self.name)
    }

    /// The committed goldens for this case: `(s1 page, s2 page, diagnostics)`.
    pub fn committed(&self) -> [String; 3] {
        ["s1.folio", "s2.folio", "diagnostics"]
            .map(|ext| read(&golden_dir().join(self.file(ext).as_str())))
    }
}

/// The golden diagnostics spelling the echo guest reads back.
pub fn diagnostics_text(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::default();
    for diagnostic in diagnostics {
        assert_eq!(diagnostic.message.find('\n'), None, "line-atomic messages");
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        };
        let stage = match diagnostic.stage {
            Stage::Source => "source",
            Stage::Surface => "surface",
            Stage::Semantic => "semantic",
            Stage::Lowered => "lowered",
            Stage::Emit => "emit",
        };
        let span = diagnostic.span;
        let _ = writeln!(
            out,
            "diagnostic {severity} {stage} {}:{} {}",
            span.start, span.end, diagnostic.message
        );
        for part in &diagnostic.parts {
            let kind = match part.kind {
                PartKind::Primary => "primary",
                PartKind::Secondary => "secondary",
                PartKind::Help => "help",
                PartKind::Suggestion => "suggestion",
            };
            let _ = writeln!(
                out,
                "part {kind} {}:{} {}",
                part.span.start, part.span.end, part.message
            );
        }
        if let Some(Witness::LegacyExempt(producer)) = &diagnostic.witness {
            let _ = writeln!(out, "witness legacy-exempt {producer}");
        }
    }
    out
}

/// A lowered block's three golden texts.
pub fn golden_texts(lowered: &LoweredBlock) -> [String; 3] {
    [
        lowered.surface.text.clone(),
        lowered.semantic.text.clone(),
        diagnostics_text(&lowered.diagnostics),
    ]
}

/// Build the echo guest for `wasm32-wasip2` with `features`, into its own
/// target directory, and return the component path.
pub fn build_echo_guest(features: &[&str]) -> PathBuf {
    build_guest("echo", "vize_contract_echo_guest", features)
}

/// Build the guest crate in `tests/guests/<dir>` for `wasm32-wasip2` with
/// `features`, into its own target directory, and return the component.
pub fn build_guest(dir: &str, artifact: &str, features: &[&str]) -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/guests")
        .join(dir)
        .join("Cargo.toml");
    let variant = if features.is_empty() {
        cstr!("default")
    } else {
        cstr!("{}", features.join("+"))
    };
    let target_root = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"),
        PathBuf::from,
    );
    let target_dir = target_root
        .join("contract-guests")
        .join(dir)
        .join(variant.as_str());
    let mut command = Command::new(env!("CARGO"));
    command
        .args([
            "build",
            "--release",
            "--locked",
            "--target",
            "wasm32-wasip2",
        ])
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target_dir);
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
    for leaked in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_TARGET_DIR",
    ] {
        command.env_remove(leaked);
    }
    let status = command.status().expect("cargo runs");
    assert!(status.success(), "building the echo guest failed: {status}");
    target_dir
        .join("wasm32-wasip2/release")
        .join(cstr!("{artifact}.wasm").as_str())
}
