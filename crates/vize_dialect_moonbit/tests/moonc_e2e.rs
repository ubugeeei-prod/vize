//! The live toolchain (the `moonc` feature): the pinned native `moonc`
//! type-checks every fixture's projection and answers exactly the
//! committed JSON lines, which map to exactly the committed rendering.
//!
//! CI installs the toolchain at `.moonbit-version` first
//! (`.github/actions/setup-moonbit`); a different `moonc` fails the
//! version assertion before any diagnostic is compared.

mod support;

use vize_dialect_moonbit::host::{CheckUnit, HostError, MooncHost};
use vize_dialect_moonbit::native::{ARGUMENT_LIMIT, NativeMoonc};
use vize_dialect_moonbit::render::render;
use vize_s0::Allocator;

use support::{FIXTURES, golden, pinned_toolchain, read};

fn host() -> NativeMoonc {
    let host = NativeMoonc::discover().expect("MOON_HOME points at a MoonBit installation");
    assert_eq!(
        host.toolchain(),
        pinned_toolchain(),
        "moonc is not the pinned toolchain"
    );
    host
}

#[test]
fn pinned_moonc_checks_every_fixture_exactly() {
    let mut host = host();
    for name in FIXTURES {
        let source = read(name, ".vue");
        let allocator = Allocator::new();
        let file_name = [name, ".vue"].concat();
        let checked = vize_dialect_moonbit::check(&allocator, &source, &file_name, &mut host)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(checked.toolchain, pinned_toolchain());
        let lines: Vec<_> = checked
            .diagnostics
            .iter()
            .map(|mapped| mapped.diagnostic.clone())
            .collect();
        let raw = host
            .check(&CheckUnit {
                package: vize_dialect_moonbit::projection::PACKAGE,
                file_name: &checked.projection.file_name,
                source: &checked.projection.text,
            })
            .unwrap();
        let mut jsonl = raw.lines.join("\n");
        if !jsonl.is_empty() {
            jsonl.push('\n');
        }
        golden(name, ".moonc.jsonl", &jsonl);
        assert_eq!(
            lines.len(),
            raw.lines.len(),
            "{name}: the two runs disagree"
        );
        let rendered = render(
            &source,
            &file_name,
            &checked.projection,
            &checked.diagnostics,
        );
        golden(name, ".diagnostics", &rendered);
    }
}

#[test]
fn the_clean_fixture_has_no_diagnostic_at_all() {
    let source = read("todo", ".vue");
    let allocator = Allocator::new();
    let checked =
        vize_dialect_moonbit::check(&allocator, &source, "todo.vue", &mut host()).unwrap();
    assert_eq!(checked.diagnostics, []);
}

#[test]
fn an_oversized_projection_is_refused_before_spawning() {
    let text = "x".repeat(ARGUMENT_LIMIT + 1);
    let unit = CheckUnit {
        package: "vize/sfc",
        file_name: "big.mbt",
        source: &text,
    };
    assert_eq!(
        host().check(&unit).unwrap_err(),
        HostError::TooLarge {
            bytes: ARGUMENT_LIMIT + 1,
            limit: ARGUMENT_LIMIT
        }
    );
}
