//! Croquis folio fixture harness.
//!
//! Every `tests/fixtures/croquis/*.vue` input is analyzed with the
//! inspector's croquis recipe, rendered through the live
//! `Croquis::to_vir()` renderer, parsed as a [`CroquisFolio`], and printed
//! canonically. The committed `*.folio` next to each input is that
//! canonical text; `davinci-opt --roundtrip` must be identity on it.
//!
//! Regenerate the committed folios after a deliberate renderer change:
//!
//! ```text
//! UPDATE_FOLIO_FIXTURES=1 cargo test -p vize_davinci --test croquis_folio
//! ```

use std::path::{Path, PathBuf};

use vize_atelier_core::Allocator;
use vize_atelier_sfc::{
    SfcParseOptions, croquis::SfcCroquisOptions, croquis::analyze_sfc_descriptor, parse_sfc,
};
use vize_davinci::folio::{Folio, FolioMode, croquis::CroquisFolio};
use vize_s0::String;

/// Render the croquis VIR dump for an SFC source, mirroring the curator
/// inspector recipe (`crates/vize_curator/src/inspector/payload.rs`).
fn vir_of(source: &str) -> String {
    let descriptor =
        parse_sfc(source, SfcParseOptions::default()).expect("fixture must be a parseable SFC");
    let template_allocator = Allocator::default();
    let template_root = descriptor.template.as_ref().map(|template| {
        let (root, _parse_errors) =
            vize_atelier_core::parser::parse(&template_allocator, template.content.as_ref());
        root
    });
    let mut croquis_options = SfcCroquisOptions::for_lint();
    croquis_options
        .analyzer_options
        .collect_template_expressions = true;
    let croquis = analyze_sfc_descriptor(&descriptor, template_root.as_ref(), croquis_options);
    croquis.to_vir()
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("croquis")
}

fn fixture_inputs() -> Vec<PathBuf> {
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(fixture_dir())
        .expect("fixture directory must exist")
        .map(|entry| entry.expect("readable fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vue"))
        .collect();
    inputs.sort();
    inputs
}

#[test]
fn committed_folios_are_canonical_renderer_output() {
    let inputs = fixture_inputs();
    assert!(
        inputs.len() >= 10,
        "the acceptance gate needs at least 10 croquis folio fixtures, found {}",
        inputs.len()
    );

    let update = std::env::var_os("UPDATE_FOLIO_FIXTURES").is_some();
    for input in &inputs {
        let source = std::fs::read_to_string(input).expect("readable fixture source");
        let vir = vir_of(&source);

        // The parser must accept everything the live renderer emits.
        let folio = match CroquisFolio::parse(vir.as_str()) {
            Ok(folio) => folio,
            Err(error) => panic!(
                "{}: renderer output did not parse: {error}\n--- renderer output ---\n{vir}",
                input.display()
            ),
        };
        let canonical = folio.print_to_string(FolioMode::Full);

        let folio_path = input.with_extension("folio");
        if update {
            std::fs::write(&folio_path, canonical.as_str()).expect("writable folio fixture");
        }
        let committed = std::fs::read_to_string(&folio_path)
            .unwrap_or_else(|_| panic!("{}: missing committed folio", folio_path.display()));
        assert_eq!(
            committed,
            canonical.as_str(),
            "{}: committed folio is stale; rerun with UPDATE_FOLIO_FIXTURES=1",
            folio_path.display()
        );

        // Round-trip laws on the canonical text (the davinci-opt gate).
        let reparsed = CroquisFolio::parse(canonical.as_str()).expect("canonical text parses");
        assert_eq!(
            reparsed.print_to_string(FolioMode::Full).as_str(),
            canonical.as_str(),
            "{}: print(parse(t)) must be identity on canonical text",
            folio_path.display()
        );
        assert_eq!(
            reparsed,
            folio,
            "{}: parse(print(v)) must be structural identity",
            folio_path.display()
        );
    }
}
