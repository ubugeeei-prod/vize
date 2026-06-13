//! JSX/TSX ecosystem reference smoke (the ecosystem-CI portion of #1489 / #1491).
//!
//! A manual, network-gated coverage run over upstream JSX reference suites
//! (`@vue/babel-plugin-jsx`, `vue-jsx-vapor`), each pinned by full commit SHA in
//! `tests/ecosystem/testbeds.json`. Real-world component-library testbeds are
//! owned by the Vize-wide app e2e fixture registry under `tests/_fixtures/`.
//!
//! It is `#[ignore]`d so PR CI stays fast and offline; run it explicitly:
//!
//! ```text
//! cargo test -p vize_atelier_jsx --test ecosystem_smoke -- --ignored --nocapture
//! ```
//!
//! The smoke shallow-clones each pinned entry, walks the configured roots for
//! `.jsx`/`.tsx` files, and feeds every file through [`lower_source`]. It reports
//! how many files lower cleanly, lower with diagnostics, or panic. This is a
//! robustness/coverage signal over real-world JSX — not a byte-for-byte parity
//! gate. A panic is always a Vize bug (the compiler must surface a diagnostic,
//! never unwind), so the run fails if any input panics.

// Std-only manual test harness: the manifest deserializes into plain structs, so
// std `String` (what `serde` derives into) is intentional here rather than the
// workspace `vize_carton::String`.
#![allow(clippy::disallowed_types)]

use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_carton::Bump;

#[derive(Deserialize)]
struct Manifest {
    references: Vec<Entry>,
    testbeds: Vec<Entry>,
}

#[derive(Deserialize, Clone)]
struct Entry {
    id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    repository: String,
    revision: String,
    roots: Vec<String>,
    extensions: Vec<String>,
}

#[derive(Default)]
struct Coverage {
    files: usize,
    clean: usize,
    diagnostics: usize,
    panicked: usize,
}

fn load_manifest() -> Manifest {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ecosystem/testbeds.json");
    let raw = fs::read_to_string(path).expect("read testbeds.json");
    serde_json::from_str(&raw).expect("parse testbeds.json")
}

/// Shallow-checkout `revision` of `repository` into `dest`. Returns false (with a
/// printed note) when the network/git is unavailable so an offline manual run
/// degrades gracefully instead of looking like a code failure.
fn shallow_checkout(repository: &str, revision: &str, dest: &Path) -> bool {
    let _ = fs::create_dir_all(dest);
    let git = |args: &[&str]| -> bool {
        Command::new("git")
            .arg("-C")
            .arg(dest)
            .args(args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    git(&["init", "-q"])
        && git(&["remote", "add", "origin", repository])
        && git(&["fetch", "--depth", "1", "-q", "origin", revision])
        && git(&["checkout", "-q", "FETCH_HEAD"])
}

fn collect_files(root: &Path, extensions: &[String], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip vendored/transient dirs that never hold authored components.
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == "node_modules" || name == ".git" || name == "dist" {
                continue;
            }
            collect_files(&path, extensions, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            // Match `.tsx`/`.jsx` against the manifest's dotted extensions without
            // allocating a `format!` string per file.
            if extensions
                .iter()
                .any(|wanted| wanted.strip_prefix('.').is_some_and(|bare| bare == ext))
            {
                out.push(path);
            }
        }
    }
}

fn lang_for(path: &Path) -> JsxLang {
    match path.extension().and_then(|e| e.to_str()) {
        Some("tsx") => JsxLang::Tsx,
        _ => JsxLang::Jsx,
    }
}

fn measure(entry: &Entry, checkout: &Path) -> Coverage {
    let mut files = Vec::new();
    for root in &entry.roots {
        collect_files(&checkout.join(root), &entry.extensions, &mut files);
    }

    let mut cov = Coverage {
        files: files.len(),
        ..Coverage::default()
    };
    for path in &files {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        let lang = lang_for(path);
        let outcome = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let bump = Bump::new();
            lower_source(&bump, &source, lang).has_errors()
        }));
        match outcome {
            Ok(false) => cov.clean += 1,
            Ok(true) => cov.diagnostics += 1,
            Err(_) => {
                cov.panicked += 1;
                eprintln!("  PANIC lowering {}", path.display());
            }
        }
    }
    cov
}

#[test]
#[ignore = "network-gated ecosystem coverage smoke; run with --ignored"]
fn jsx_ecosystem_coverage_smoke() {
    // Quiet panic hook so caught per-file panics don't spam the report; we count
    // and surface them ourselves.
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let manifest = load_manifest();
    let base = std::env::temp_dir().join("vize-jsx-ecosystem");
    let _ = fs::remove_dir_all(&base);

    let mut cloned = 0usize;
    let mut total_files = 0usize;
    let mut total_panicked = 0usize;

    eprintln!("\nJSX ecosystem coverage smoke (lower_source over pinned references)\n");
    eprintln!(
        "{:<22} {:>7} {:>7} {:>11} {:>9}",
        "testbed", "files", "clean", "w/diag", "PANIC"
    );

    for entry in manifest.references.iter().chain(manifest.testbeds.iter()) {
        let dest = base.join(&entry.id);
        if !shallow_checkout(&entry.repository, &entry.revision, &dest) {
            eprintln!(
                "{:<22} (clone failed — offline? skipped)",
                entry.display_name
            );
            continue;
        }
        cloned += 1;
        let cov = measure(entry, &dest);
        total_files += cov.files;
        total_panicked += cov.panicked;
        eprintln!(
            "{:<22} {:>7} {:>7} {:>11} {:>9}",
            entry.display_name, cov.files, cov.clean, cov.diagnostics, cov.panicked
        );
    }

    let _ = fs::remove_dir_all(&base);
    panic::set_hook(previous_hook);

    if cloned == 0 {
        eprintln!("\nNo testbeds could be cloned (network/git unavailable). Smoke skipped.");
        return;
    }

    // A successful clone with zero matched files means the manifest roots drifted.
    assert!(
        total_files > 0,
        "cloned {cloned} testbeds but matched no .jsx/.tsx files — manifest roots are stale"
    );
    // The compiler must never unwind on real-world input; surface diagnostics instead.
    assert_eq!(
        total_panicked, 0,
        "Vize JSX lowering panicked on {total_panicked} real-world file(s) — that is a compiler bug"
    );
}
