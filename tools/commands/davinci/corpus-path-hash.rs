#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```
//!
//! Pins the typechecker path normalization: machine directories collapse,
//! relative paths and diagnostic text do not.

#[path = "../../support/davinci/corpus_paths.rs"]
mod corpus_paths;

use std::process::ExitCode;

fn main() -> ExitCode {
    let first = corpus_paths::normalize_machine_paths(
        "Building Corsa virtual project for 2 files under /private/tmp/vize-int/tests/_fixtures/_git/vue-lottie...\n",
    );
    let second = corpus_paths::normalize_machine_paths(
        "Building Corsa virtual project for 2 files under /Users/ubugeeei/Source/github.com/ubugeeei-prod/vize/tests/_fixtures/_git/vue-lottie...\n",
    );
    if first != second || !first.contains("<abs>") || !first.ends_with("...\n") {
        eprintln!("banner paths did not collapse:\n{first}\n{second}");
        return ExitCode::from(1);
    }
    if first.contains("/private/tmp") || first.contains("/Users/") {
        eprintln!("absolute path survived normalization: {first}");
        return ExitCode::from(1);
    }

    let relative = corpus_paths::normalize_machine_paths(
        "{\n  \"file\": \"src/App.vue\",\n  \"diagnostics\": [\"error:1:1 [TS2307] Cannot find module './missing'.\"]\n}\n",
    );
    if !relative.contains("src/App.vue")
        || !relative.contains("Cannot find module './missing'.")
        || relative.contains("<abs>")
    {
        eprintln!("relative diagnostic text changed: {relative}");
        return ExitCode::from(1);
    }

    let clean = corpus_paths::normalize_machine_paths(
        "error:1:1 [TS2322] Type 'number' is not assignable to type 'string'. (/tmp/proj/src/App.vue)\n",
    );
    let extra = corpus_paths::normalize_machine_paths(
        "error:1:1 [TS2322] Type 'number' is not assignable to type 'string'. (/tmp/proj/src/App.vue)\nerror:2:1 [TS2304] Cannot find name 'nope'.\n",
    );
    if clean == extra || !clean.contains("<abs>") {
        eprintln!("an added diagnostic collapsed into path noise");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
