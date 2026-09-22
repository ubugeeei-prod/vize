//! Type-check a MoonBit Vue SFC with the pinned `moonc` (the P6-4a
//! showcase):
//!
//! ```sh
//! cargo run -p vize_dialect_moonbit --features moonc --example moonbit_check -- \
//!   crates/vize_dialect_moonbit/tests/fixtures/todo-typos.vue [--projection]
//! ```
//!
//! Prints `moonc`'s diagnostics at their authored template positions;
//! `--projection` prints the virtual MoonBit file instead. Exits 1 when
//! any diagnostic is an error.

use std::io::Write as _;
use std::path::Path;

use vize_dialect_moonbit::diagnostic::Level;
use vize_dialect_moonbit::native::NativeMoonc;
use vize_dialect_moonbit::render::render;
use vize_s0::Allocator;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: moonbit_check <file.vue> [--projection]");
        std::process::exit(2);
    };
    let projection_only = args.next().as_deref() == Some("--projection");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| fail(&error));
    let file_name = Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("App.vue");
    let mut host = NativeMoonc::discover().unwrap_or_else(|error| fail(&error));
    let allocator = Allocator::new();
    let checked = vize_dialect_moonbit::check(&allocator, &source, file_name, &mut host)
        .unwrap_or_else(|error| fail(&error));
    let mut out = std::io::stdout().lock();
    if projection_only {
        let _ = out.write_all(checked.projection.text.as_bytes());
        return;
    }
    let rendered = render(
        &source,
        file_name,
        &checked.projection,
        &checked.diagnostics,
    );
    let _ = out.write_all(rendered.as_bytes());
    let errors = checked
        .diagnostics
        .iter()
        .filter(|mapped| mapped.diagnostic.level == Level::Error)
        .count();
    let _ = writeln!(
        out,
        "{file_name}: {errors} error(s), {} expression(s) checked by moonc {}",
        checked.projection.positions.len(),
        checked.toolchain
    );
    if errors > 0 {
        std::process::exit(1);
    }
}

fn fail(error: &dyn std::fmt::Display) -> ! {
    eprintln!("moonbit_check: {error}");
    std::process::exit(2);
}
