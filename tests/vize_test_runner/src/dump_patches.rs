// Temporary script to dump actual outputs for patches tests
// Run with: cargo run -p vize_test_runner --bin dump_patches

use std::path::PathBuf;
use vize_test_runner::{compile, load_fixture, CompilerMode};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest_dir.parent().unwrap().join("fixtures");
    let fixture_path = fixtures_dir.join("sfc/patches.toml");

    let fixture = load_fixture(&fixture_path).expect("Failed to load fixture");

    for case in &fixture.cases {
        let actual = compile(&case.input, CompilerMode::Sfc, &case.options);
        println!("=== TEST: {} ===", case.name);
        println!("{}", actual);
        println!("=== END ===\n");
    }
}
