//! The P4-13 corpus lane after the scanner's deletion: every committed Musea
//! fixture's full fingerprint, pinned.
//!
//! Slice 1 compared the S0/S1 reader with the retired hand scanner over this
//! corpus (209 of 217 identical, 8 ledgered divergences pinned in
//! `divergence_tests.rs`); slice 2 deleted the scanner and froze the S0/S1
//! fingerprints here, so the corpus stays a TS-1 regression suite of the
//! reader it validated.
//!
//! A fingerprint is the `Debug` rendering of the whole parse result, the
//! file-absolute offset of every borrowed slice (LSP consumers do pointer
//! arithmetic on them), every downstream artifact built from the descriptor
//! (Storybook CSF, Vue, component docs, catalog entry, palette, owned JSON)
//! and the status warnings. `FINGERPRINTS.tsv` pins each one by its FNV-1a
//! 64 digest — exact equality of the digested bytes, the TS-11 shape.
//!
//! Scope proof: the corpus and manifest counts are pinned, so an empty or
//! half-read fixture directory fails. `VIZE_MUSEA_GOLDEN_WRITE=1` rewrites
//! the manifest (review the diff like a snapshot);
//! `VIZE_MUSEA_GOLDEN_SHOW=<fixture>` prints one fingerprint.

use std::path::{Path, PathBuf};

use super::{art_status_warnings, parse_art};
use crate::docs::{CatalogEntry, DocOptions, generate_component_doc};
use crate::palette::{PaletteOptions, generate_palette};
use crate::types::{ArtParseOptions, ArtParseResult};
use crate::{transform_to_csf, transform_to_vue};
use vize_s0::{Allocator, String, append};

/// Committed corpus size (`tests/fixtures/differential`, excluding the
/// manifests). Changing the corpus means changing this number.
const COMMITTED_FIXTURES: usize = 217;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/differential")
}

fn manifest_path() -> PathBuf {
    fixture_dir().join("FINGERPRINTS.tsv")
}

/// The committed fixtures (`*.vue.txt`: many are malformed by design, so
/// they stay out of the repository's Vue lint and format lanes), sorted.
fn corpus() -> std::vec::Vec<(String, String)> {
    let mut paths: std::vec::Vec<PathBuf> = std::fs::read_dir(fixture_dir())
        .expect("readable corpus directory")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            name.strip_suffix(".vue.txt").is_some()
        })
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let source = std::fs::read_to_string(&path).expect("UTF-8 fixture");
            (String::from(name.as_ref()), String::from(source))
        })
        .collect()
}

/// File-absolute offset of `slice` when it borrows `source`, or `arena`.
fn place(out: &mut String, label: &str, slice: &str, source: &str) {
    let base = source.as_ptr() as usize;
    let ptr = slice.as_ptr() as usize;
    if ptr >= base && ptr + slice.len() <= base + source.len() {
        append!(*out, "{label}@{}+{}\n", ptr - base, slice.len());
    } else {
        append!(*out, "{label}@arena\n");
    }
}

fn describe(out: &mut String, result: &ArtParseResult<'_>, source: &str) {
    append!(*out, "{result:#?}\n");
    let Ok(desc) = result else {
        return;
    };
    let meta = &desc.metadata;
    place(out, "title", meta.title, source);
    for (label, value) in [
        ("description", meta.description),
        ("component", meta.component),
        ("category", meta.category),
    ] {
        if let Some(value) = value {
            place(out, label, value, source);
        }
    }
    for tag in meta.tags.iter() {
        place(out, "tag", tag, source);
    }
    for variant in &desc.variants {
        place(out, "variant.name", variant.name, source);
        place(out, "variant.template", variant.template, source);
    }
    for script in [&desc.script_setup, &desc.script].into_iter().flatten() {
        place(out, "script.content", script.content, source);
        if let Some(lang) = script.lang {
            place(out, "script.lang", lang, source);
        }
    }
    for style in desc.styles.iter() {
        place(out, "style.content", style.content, source);
        if let Some(lang) = style.lang {
            place(out, "style.lang", lang, source);
        }
    }
    let docs = DocOptions {
        include_source: true,
        include_templates: true,
        include_metadata: true,
        include_toc: true,
        toc_threshold: 1,
        ..DocOptions::default()
    };
    append!(*out, "csf: {:#?}\n", transform_to_csf(desc));
    append!(*out, "vue: {:#?}\n", transform_to_vue(desc));
    append!(*out, "doc: {:#?}\n", generate_component_doc(desc, &docs));
    append!(
        *out,
        "catalog: {:#?}\n",
        CatalogEntry::from_descriptor(desc, "/gallery")
    );
    append!(
        *out,
        "palette: {:#?}\n",
        generate_palette(desc, &PaletteOptions::default())
    );
}

/// One fixture's full fingerprint.
fn fingerprint(name: &str, source: &str) -> String {
    let allocator = Allocator::new();
    let options = ArtParseOptions {
        filename: String::from(name),
    };
    let result = parse_art(&allocator, source, options);
    let mut out = String::default();
    describe(&mut out, &result, source);
    let owned = result
        .map(|desc| serde_json::to_string(&desc.into_owned()).expect("owned JSON"))
        .unwrap_or_default();
    append!(out, "owned: {owned}\n");
    append!(
        out,
        "warnings: {:?}\n",
        art_status_warnings(&allocator, source, name).as_slice()
    );
    out
}

/// FNV-1a 64: stable by definition, so the manifest survives toolchain
/// upgrades (std's hashers promise no such thing).
fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn current_manifest() -> String {
    let mut out = String::from("fixture\tfnv1a64\n");
    for (name, source) in corpus() {
        let digest = fnv1a64(fingerprint(&name, &source).as_bytes());
        append!(out, "{name}\t{digest:016x}\n");
    }
    out
}

#[test]
fn committed_musea_fixtures_match_their_fingerprints() {
    assert_eq!(
        corpus().len(),
        COMMITTED_FIXTURES,
        "scope proof: corpus size"
    );
    let current = current_manifest();
    if std::env::var_os("VIZE_MUSEA_GOLDEN_WRITE").is_some() {
        std::fs::write(manifest_path(), current.as_bytes()).expect("writable manifest");
        return;
    }
    let committed = std::fs::read_to_string(manifest_path()).expect("committed manifest");
    assert_eq!(
        committed.lines().count(),
        COMMITTED_FIXTURES + 1,
        "scope proof: manifest rows"
    );
    assert_eq!(current.as_str(), committed.as_str());
}

/// The in-repo gallery (`examples/vite-musea`, inline `<art>` blocks in real
/// components) parses to exactly the variants it declares.
#[test]
fn example_gallery_components_keep_their_variants() {
    let gallery = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/vite-musea/src");
    let mut rows = std::vec::Vec::new();
    let mut paths: std::vec::Vec<PathBuf> = std::fs::read_dir(gallery.join("components"))
        .expect("gallery components")
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    paths.sort();
    for path in paths {
        let source = std::fs::read_to_string(&path).expect("UTF-8 component");
        let allocator = Allocator::new();
        let desc = parse_art(&allocator, &source, ArtParseOptions::default()).expect("art");
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let variants: std::vec::Vec<&str> = desc.variants.iter().map(|v| v.name).collect();
        rows.push((String::from(name.as_ref()), variants.join(",")));
    }
    let rows: std::vec::Vec<(&str, &str)> = rows
        .iter()
        .map(|(name, variants)| (name.as_str(), variants.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            (
                "Alert.vue",
                "Default,Success,Warning,Error,With Close Button"
            ),
            ("Avatar.vue", "Default,With Image,Sizes,Group"),
            ("Badge.vue", "Default,Variants,Sizes"),
            (
                "Button.vue",
                "Default,Primary,Secondary,Disabled,Sizes,All Variants"
            ),
            ("Card.vue", "Default,With Image,Outlined,Elevated"),
            ("Input.vue", "Default,With Value,Search,With Error,Disabled"),
        ]
    );
}

/// Print one fixture's fingerprint (triage aid for a failing run):
/// `VIZE_MUSEA_GOLDEN_SHOW=<fixture> cargo test -p vize_musea show_`.
#[test]
fn show_fixture_fingerprint() {
    let Some(name) = std::env::var_os("VIZE_MUSEA_GOLDEN_SHOW") else {
        return;
    };
    let name = name.to_string_lossy();
    let source = std::fs::read_to_string(fixture_dir().join(name.as_ref())).expect("fixture");
    std::eprintln!("{}", fingerprint(&name, &source));
}
