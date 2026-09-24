// Dumps the compiler's actual output for every `sfc/patches.pkl` case.
// Run with: cargo run -p vize_test_runner --bin dump_patches

use std::{path::PathBuf, process::ExitCode};
use vize_test_runner::{CompilerMode, compile, load_fixture};

fn main() -> ExitCode {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // The crate lives at `tests/vize_test_runner`, so its parent is `tests/`.
    let tests_dir = manifest_dir.parent().unwrap_or(&manifest_dir);
    let fixture_path = tests_dir.join("fixtures").join("sfc/patches.pkl");

    let fixture = match load_fixture(&fixture_path) {
        Ok(fixture) => fixture,
        Err(error) => {
            eprintln!(
                "dump_patches: failed to load {}: {error}",
                fixture_path.display()
            );
            return ExitCode::FAILURE;
        }
    };

    for case in &fixture.cases {
        let actual = compile(&case.input, CompilerMode::Sfc, &case.options);
        println!("=== TEST: {} ===", case.name);
        println!("{}", actual);
        println!("=== END ===\n");
    }
    ExitCode::SUCCESS
}
