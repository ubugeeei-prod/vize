//! Keep both public responses and each gallery generator qualified on the same source.
#![expect(
    clippy::unwrap_used,
    reason = "fixtures and serialization are verified in this test"
)]

use vize_musea::{
    Allocator, ArtParseOptions, ArtSourceProducts,
    docs::{CatalogEntry, DocOptions, generate_component_doc},
    parse_art, parse_art_products, parse_art_status_warnings, transform_to_csf, transform_to_vue,
};

mod profile;

fn original<'a>(arena: &'a Allocator, source: &'a str, name: &str) -> ArtSourceProducts<'a> {
    let descriptor = parse_art(
        arena,
        source,
        ArtParseOptions {
            filename: name.into(),
        },
    );
    let status_warnings = parse_art_status_warnings(arena, source, name);
    ArtSourceProducts {
        descriptor,
        status_warnings,
    }
}

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    descriptors: usize,
    variants: usize,
    warnings: usize,
    error_with_warning: usize,
    descriptor_bytes: usize,
    generated_bytes: usize,
}

fn compare(source: &str, name: &str, counts: &mut Counts) {
    let base = Allocator::default();
    let head = Allocator::default();
    let actual = parse_art_products(
        &head,
        source,
        ArtParseOptions {
            filename: name.into(),
        },
    );
    let expected = original(&base, source, name);
    counts.files += 1;
    counts.warnings += actual.status_warnings.len();
    assert_eq!(
        actual.status_warnings.as_slice(),
        expected.status_warnings.as_slice(),
        "{name}"
    );
    match (actual.descriptor, expected.descriptor) {
        (Ok(a), Ok(b)) => {
            counts.descriptors += 1;
            counts.variants += a.variants.len();
            assert!(std::ptr::eq(a.source, source), "{name}");
            let options = DocOptions {
                include_source: true,
                include_metadata: true,
                include_templates: true,
                include_toc: true,
                toc_threshold: 2,
                ..DocOptions::default()
            };
            let preview_a = transform_to_vue(&a);
            let preview_b = transform_to_vue(&b);
            assert_eq!(
                (&preview_a.code, &preview_a.metadata_code),
                (&preview_b.code, &preview_b.metadata_code),
                "{name}"
            );
            let exports = |art: &_| {
                (
                    serde_json::to_vec(&transform_to_csf(art)).unwrap(),
                    serde_json::to_vec(&generate_component_doc(art, &options)).unwrap(),
                    serde_json::to_vec(&CatalogEntry::from_descriptor(art, "./components"))
                        .unwrap(),
                )
            };
            let products = exports(&a);
            assert_eq!(products, exports(&b), "{name}");
            counts.generated_bytes += products.0.len()
                + products.1.len()
                + products.2.len()
                + preview_a.code.len()
                + preview_a.metadata_code.len();
            let serialized = serde_json::to_vec(&a.into_owned()).unwrap();
            assert_eq!(
                serialized,
                serde_json::to_vec(&b.into_owned()).unwrap(),
                "{name}"
            );
            counts.descriptor_bytes += serialized.len();
        }
        (Err(a), Err(b)) => {
            assert_eq!(
                std::mem::discriminant(&a),
                std::mem::discriminant(&b),
                "{name}"
            );
            assert_eq!(a.to_string(), b.to_string(), "{name}");
            counts.error_with_warning += usize::from(!actual.status_warnings.is_empty());
        }
        _ => panic!("descriptor acceptance differs on {name}"),
    }
}

const DIRECTED: &[&str] = &[
    "<art status='wip'><variant name='Primary'><span/></variant></art>",
    "<art status='wip'>\n<variant>missing name</variant></art>",
    "<script setup>import A from './A.vue';import B from './B.vue';</script><art status='broken'><variant name='Both'><A/><B/></variant></art>",
    "<script setup>defineArtMeta({ title: 'Macro', status: 'wip' })</script><art><variant name='Main'/></art>",
    "<script setup>defineArtMeta({ status: 'wip' })</script><art deprecated status='READY'/>",
    "<script setup>defineArtMeta({ status: 'ready' })</script><art status='bad'/>",
    "<!--雪🌱-->\r\n<template><span/></template>\r\n<art draft><variant name='🌱' default><span title='x'/></variant></art><style scoped>.a{}</style>",
    "<art:x status='bad'></art:x>",
    "<art status='invalid'>",
    "<script>let a</script><script>let b</script><art status='bad'/>",
    "<style lang='scss'>.x{}</style>",
    "",
];

#[test]
fn warnings_and_descriptor_failures_are_independent_and_exports_match() {
    let mut c = Counts::default();
    for &source in DIRECTED {
        for name in ["", "views/雪🌱.art.vue"] {
            compare(source, name, &mut c);
        }
    }
    assert!(
        c.descriptors >= 8 && c.variants >= 4 && c.warnings >= 6 && c.error_with_warning >= 2,
        "{c:?}"
    );
}

fn originals(mut read: impl FnMut(usize, &str, &str)) -> usize {
    let manifest = std::fs::read(std::env::var_os("VIZE_ART_SOURCE_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut invalid = 0;
    for (i, entry) in manifest
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => read(i, name, source),
            Err(_) => invalid += 1,
        }
    }
    invalid
}

#[test]
#[ignore = "original Vue source manifest including colocated Art"]
fn descriptor_warnings_and_complete_gallery_products_on_original_files() {
    let mut c = Counts::default();
    let invalid = originals(|_, name, source| compare(source, name, &mut c));
    assert!(c.files > 0);
    eprintln!("art_source_products {c:?} invalid={invalid}");
}
