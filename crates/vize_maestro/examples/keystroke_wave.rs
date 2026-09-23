//! Keystroke cost of the hover, completion and definition wave (P5-6a).
//!
//! ```text
//! cargo run --release -p vize_maestro --example keystroke_wave -- <dir|file.vue>... [--keystrokes N]
//! ```
//!
//! For every `.vue` file with a template interpolation: `N` keystrokes
//! (default 20), each appending or removing one trailing space so that
//! consecutive buffers differ and every keystroke is a new revision. After
//! each keystroke the document store and virtual documents are updated (the
//! `didChange` work, untimed), then one hover, one completion and one
//! definition request run at the first interpolation, each timed from its
//! context's construction to its response — the first request after a
//! keystroke pays for the parse, the others read the resident memo. Prints
//! nearest-rank p50/p95 and the maximum per feature, and of the whole wave per
//! keystroke, in microseconds.

// A measurement tool: owned std strings and formatting keep it short.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use tower_lsp::lsp_types::Url;
use vize_maestro::ide::{CompletionService, DefinitionService, HoverService, IdeContext};
use vize_maestro::server::ServerState;

fn main() {
    let mut keystrokes = 20usize;
    let mut roots = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--keystrokes" {
            keystrokes = args
                .next()
                .and_then(|n| n.parse().ok())
                .expect("--keystrokes N");
        } else {
            roots.push(PathBuf::from(arg));
        }
    }
    let mut files = Vec::new();
    for root in &roots {
        collect_vue(root, &mut files);
    }
    files.sort();

    let mut samples: [Vec<u128>; 4] = Default::default();
    let (mut measured, mut skipped) = (0usize, 0usize);
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            skipped += 1;
            continue;
        };
        let Some(offset) = source.find("{{").map(|at| at + 3) else {
            skipped += 1;
            continue;
        };
        measured += 1;
        let state = ServerState::new();
        let uri = Url::from_file_path(std::fs::canonicalize(file).unwrap()).unwrap();
        for k in 0..keystrokes {
            let mut text = source.clone();
            text.push_str(if k % 2 == 0 { " " } else { "  " });
            state
                .documents
                .open(uri.clone(), text.clone(), k as i32 + 1, "vue".to_string());
            state.update_virtual_docs(&uri, &text);
            let wave = [
                time(|| {
                    let ctx = IdeContext::new(&state, &uri, offset).expect("document is open");
                    HoverService::hover(&ctx).is_some()
                }),
                time(|| {
                    let ctx = IdeContext::new(&state, &uri, offset).expect("document is open");
                    CompletionService::complete(&ctx).is_some()
                }),
                time(|| {
                    let ctx = IdeContext::new(&state, &uri, offset).expect("document is open");
                    DefinitionService::definition(&ctx).is_some()
                }),
            ];
            for (feature, micros) in wave.iter().enumerate() {
                samples[feature].push(*micros);
            }
            samples[3].push(wave.iter().sum());
        }
    }

    println!("files {measured} measured, {skipped} skipped; {keystrokes} keystrokes each");
    println!("feature     p50_us  p95_us  max_us  samples");
    for (name, values) in ["hover", "completion", "definition", "wave"]
        .iter()
        .zip(&mut samples)
    {
        values.sort_unstable();
        println!(
            "{name:<10} {:>7} {:>7} {:>7} {:>8}",
            rank(values, 50),
            rank(values, 95),
            values.last().copied().unwrap_or(0),
            values.len()
        );
    }
}

/// Microseconds `request` took; its answer is kept alive past the clock.
fn time(request: impl FnOnce() -> bool) -> u128 {
    let start = Instant::now();
    let answered = request();
    let micros = start.elapsed().as_micros();
    std::hint::black_box(answered);
    micros
}

/// Nearest-rank percentile of sorted `values`.
fn rank(values: &[u128], percentile: usize) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let index = (values.len() * percentile).div_ceil(100).max(1) - 1;
    values[index]
}

fn collect_vue(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "vue") {
            files.push(path.to_path_buf());
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let child = entry.path();
        if child.file_name().is_some_and(|name| name == "node_modules") {
            continue;
        }
        collect_vue(&child, files);
    }
}
