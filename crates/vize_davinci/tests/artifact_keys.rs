//! TS-43 — stage artifact key stability (Davinci P5-1a).
//!
//! Every block of an SFC is keyed at each stage that has an artifact for
//! it: S0 for every block, the S1 surface page and the S2 page for the
//! template. Three properties, all by exact equality:
//!
//! 1. **Golden keys, two platforms.** `fixtures/keys/base.keys` is the full
//!    key listing of `fixtures/keys/base.vue`. The Linux and macOS lanes of
//!    `.github/workflows/davinci-incremental.yml` both print the listing and
//!    compare it with the golden, so a platform-dependent byte anywhere in a
//!    recipe fails on one of them.
//! 2. **Edit locality.** Each edit case names exactly the `(block, stage)`
//!    keys it changes: zero for an edit above a block, a reorder, or a
//!    whitespace/header-order edit outside every artifact; exactly the
//!    edited block's keys otherwise.
//! 3. **Offset independence.** Identical block content keys identically at
//!    any offset, and a page lowered with file-absolute spans keys exactly
//!    like the same block lowered as its own root.
//!
//! Regenerate the golden after a deliberate recipe change (which must bump
//! the stage's `vize_davinci::key::schema` version in the same change):
//!
//! ```text
//! UPDATE_KEY_GOLDENS=1 cargo test -p vize_davinci --test artifact_keys
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_davinci::key::{ArtifactKey, source_block_key};
use vize_s0::{Allocator, SourceRoot, String};
use vize_s1_to_s2::key::SurfacePage;
use vize_s2::folio::S2Folio;

type Keys = BTreeMap<String, ArtifactKey>;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("keys")
        .join(name)
}

fn base() -> String {
    String::from(std::fs::read_to_string(fixture("base.vue")).expect("base fixture"))
}

/// The block's exact slice of `source` and its file-absolute start.
fn block_slice(source: &str, start: usize, end: usize) -> (&str, u32) {
    (
        &source[start..end],
        u32::try_from(start).expect("u32 source"),
    )
}

fn attr_pairs<'a>(
    attrs: impl Iterator<Item = (&'a str, &'a str)>,
) -> Vec<(&'a str, Option<&'a str>)> {
    attrs.map(|(name, value)| (name, Some(value))).collect()
}

fn label(block: &str, stage: &str) -> String {
    let mut out = String::default();
    write!(out, "{block} {stage}").expect("string write");
    out
}

/// S1 and S2 keys of a template block, parsed from its slice of the file
/// and lowered with file-absolute spans (the production shape).
fn template_keys(source: &str, block: &str, start: u32) -> (ArtifactKey, ArtifactKey) {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, block);
    let s1 = ArtifactKey::of(&SurfacePage(&tree), start);
    let frame = SourceRoot::new(source)
        .and_then(|root| root.block(block, start))
        .expect("template block frame");
    let lowered = vize_s1_to_s2::lower_source_block(&allocator, &tree, &errors, frame);
    let s2 = ArtifactKey::of(&S2Folio::of(&lowered.root.ops), start);
    (s1, s2)
}

fn keys_of(source: &str) -> Keys {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("fixture SFC");
    let mut keys = Keys::new();
    let mut put = |block: &str, stage: &str, key: ArtifactKey| {
        assert_eq!(
            keys.insert(label(block, stage), key),
            None,
            "{block} {stage}"
        );
    };
    if let Some(template) = &descriptor.template {
        let (block, start) = block_slice(source, template.loc.start, template.loc.end);
        assert_eq!(block, template.content.as_ref());
        let attrs = attr_pairs(template.attrs.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
        put(
            "template",
            "s0",
            source_block_key("template", &attrs, block),
        );
        let (s1, s2) = template_keys(source, block, start);
        put("template", "s1", s1);
        put("template", "s2", s2);
    }
    for (name, script) in [
        ("script", &descriptor.script),
        ("script-setup", &descriptor.script_setup),
    ] {
        if let Some(script) = script {
            let (block, _) = block_slice(source, script.loc.start, script.loc.end);
            let attrs = attr_pairs(script.attrs.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
            put(name, "s0", source_block_key("script", &attrs, block));
        }
    }
    for (index, style) in descriptor.styles.iter().enumerate() {
        let (block, _) = block_slice(source, style.loc.start, style.loc.end);
        let attrs = attr_pairs(style.attrs.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
        let mut name = String::default();
        write!(name, "style[{index}]").expect("string write");
        put(&name, "s0", source_block_key("style", &attrs, block));
    }
    for (index, custom) in descriptor.custom_blocks.iter().enumerate() {
        let (block, _) = block_slice(source, custom.loc.start, custom.loc.end);
        let attrs = attr_pairs(custom.attrs.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
        let mut name = String::default();
        write!(name, "{}[{index}]", custom.block_type).expect("string write");
        put(
            &name,
            "s0",
            source_block_key(&custom.block_type, &attrs, block),
        );
    }
    keys
}

fn listing(keys: &Keys) -> String {
    let mut out = String::default();
    for (label, key) in keys {
        writeln!(out, "{label} {key}").expect("string write");
    }
    out
}

fn changed(before: &Keys, after: &Keys) -> Vec<String> {
    let labels: std::collections::BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    labels
        .into_iter()
        .filter(|label| before.get(*label) != after.get(*label))
        .cloned()
        .collect()
}

/// `base` with the single occurrence of `find` replaced.
fn edit(base: &str, find: &str, replace: &str) -> String {
    assert_eq!(base.matches(find).count(), 1, "edit anchor {find:?}");
    let at = base.find(find).expect("anchor");
    let mut out = String::from(&base[..at]);
    out.push_str(replace);
    out.push_str(&base[at + find.len()..]);
    out
}

/// `base` with the `<template>` block (and the blank line after it) moved
/// to the front of the file.
fn template_first(base: &str) -> String {
    let start = base.find("<template>").expect("template open");
    let close = "</template>\n\n";
    let end = base.find(close).expect("template close") + close.len();
    let mut out = String::from(&base[start..end]);
    out.push_str(&base[..start]);
    out.push_str(&base[end..]);
    out
}

#[test]
fn base_keys_equal_the_committed_golden() {
    let keys = listing(&keys_of(&base()));
    println!("TS-43 keys for fixtures/keys/base.vue:\n{keys}");
    let golden = fixture("base.keys");
    if std::env::var_os("UPDATE_KEY_GOLDENS").is_some() {
        std::fs::write(&golden, keys.as_bytes()).expect("write golden");
    }
    let expected = std::fs::read_to_string(&golden).expect("committed golden");
    assert_eq!(keys.as_str(), expected.as_str());
}

#[test]
fn each_edit_changes_exactly_the_edited_blocks_keys() {
    let base = base();
    let base_keys = keys_of(&base);
    let template = ["template s0", "template s1", "template s2"].as_slice();
    let cases: [(&str, String, &[&str]); 10] = [
        (
            "insert above every block (outside any block)",
            edit(&base, "<script setup", "<!-- header -->\n\n<script setup"),
            &[],
        ),
        (
            "insert inside the script above the template",
            edit(&base, "const label", "const extra = 1\nconst label"),
            &["script-setup s0"],
        ),
        (
            "insert inside the template",
            edit(
                &base,
                "    <p v-else",
                "    <span>new</span>\n    <p v-else",
            ),
            template,
        ),
        (
            "insert inside the i18n block below the template",
            edit(&base, "\"Hello\"", "\"Hello\", \"bye\": \"Bye\""),
            &["i18n[0] s0"],
        ),
        (
            "reorder blocks (template first)",
            template_first(&base),
            &[],
        ),
        (
            "whitespace between blocks",
            edit(&base, "</template>\n", "</template>\n\n\n\n"),
            &[],
        ),
        (
            "whitespace inside the template",
            edit(&base, "    <p v-if", "      <p v-if"),
            template,
        ),
        (
            "header attribute order",
            edit(
                &base,
                "<script setup lang=\"ts\">",
                "<script lang=\"ts\" setup>",
            ),
            &[],
        ),
        (
            "header attribute of the second style",
            edit(&base, "</style>\n\n<style scoped>", "</style>\n\n<style>"),
            &["style[1] s0"],
        ),
        (
            "whitespace inside the first style",
            edit(
                &base,
                "</style>\n\n<style scoped>",
                "\n</style>\n\n<style scoped>",
            ),
            &["style[0] s0"],
        ),
    ];
    for (case, source, expected) in cases {
        assert_ne!(source, base, "{case}: the edit must change the file");
        let expected: Vec<String> = expected.iter().copied().map(String::from).collect();
        assert_eq!(changed(&base_keys, &keys_of(&source)), expected, "{case}");
    }
}

#[test]
fn identical_block_content_keys_identically_at_any_offset() {
    let keys = keys_of(&base());
    // The two `<style scoped>` blocks are byte-identical at different offsets.
    assert_eq!(keys["style[0] s0"], keys["style[1] s0"]);

    // The template lowered with file-absolute spans and rebased keys exactly
    // like the same template lowered as its own root (base-zero spans).
    let base = base();
    let start = base.find("<template>").expect("template") + "<template>".len();
    let end = base.find("</template>").expect("template close");
    let block = &base[start..end];
    let absolute = template_keys(&base, block, u32::try_from(start).expect("u32"));
    let own_root = template_keys(block, block, 0);
    assert_eq!(absolute, own_root);
    assert_eq!((keys["template s1"], keys["template s2"]), absolute);
}
