use super::*;
use vize_l0::Span;
use vize_l4::write::source_map::SourceMapIndex;

#[test]
#[ignore = "optimized paired measurements on dense ASCII, Unicode and short lines"]
fn mapped_targets_share_reverse_order_unicode_coordinates() {
    mark_installed();
    for (label, part) in [
        ("ascii", "abcdefghijklmnop"),
        ("astral", "雪🌸é𐀀"),
        ("lines", "雪🌸\r\n"),
        ("separators", "é\u{2028}𐀀\u{2029}"),
    ] {
        let source = part.repeat(192);
        let mut a = EmitDocument::new(true);
        let mut b = EmitDocument::new(true);
        for i in 0..192 {
            let at = u32::try_from(i * part.len()).unwrap();
            let rev = u32::try_from((191 - i) * part.len()).unwrap();
            a.push_linked("雪a🦀", Span::new(rev, rev));
            b.push_linked("x\r\n", Span::new(at, at));
        }
        let maps = |indexed| {
            if indexed {
                let index = SourceMapIndex::new(&source);
                (
                    index.source_map(&a, FILENAME),
                    index.source_map(&b, FILENAME),
                )
            } else {
                (
                    a.source_map(FILENAME, &source),
                    b.source_map(FILENAME, &source),
                )
            }
        };
        assert_eq!(maps(false), maps(true));
        paired(&format!("reverse-links {label}"), |indexed| {
            drop(black_box(maps(indexed)))
        });
    }
}

#[test]
#[ignore = "compares the complete tracked or upstream Vue input manifest"]
fn raw_maps_match_on_original_components_and_positions_on_refused_sources() {
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_SCSS_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut stock_admitted, mut modules, mut styles) = (0, 0, 0, 0);
    // Both controls must emit whole JS and CSS maps even when the supplied
    // upstream corpus contains no complete component in the supported SCSS subset.
    const REQUIRED: [(&str, &str); 2] = [
        (
            "crates/vize_atelier_sfc/tests/fixtures/native_scss_maps/variable.vue.txt",
            ".card {\n  color: red;\n}",
        ),
        (
            "crates/vize_atelier_sfc/tests/fixtures/native_scss_maps/mixed.vue.txt",
            ".plain{color:blue}\n.outer .child {\n  padding: 4px;\n}",
        ),
    ];
    let mut required_scss = 0;
    let mut non_utf8 = 0;
    let corpus = manifest
        .split_terminator('\0')
        .filter(|name| !name.ends_with(".scss") && !REQUIRED.iter().any(|(path, _)| path == name));
    for name in REQUIRED.iter().map(|(name, _)| *name).chain(corpus) {
        let source = match std::fs::read_to_string(root.join(name)) {
            Ok(source) => source,
            Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
                eprintln!("source_map_corpus rejected non-UTF-8 input {name}");
                non_utf8 += 1;
                continue;
            }
            Err(error) => panic!("{name}: {error}"),
        };
        let arena = Allocator::default();
        let index = SourceMapIndex::new(&source);
        let options = NativeSfcCompileOptions {
            filename: name,
            source_map: true,
            ..Default::default()
        };
        let stock = compile_native_sfc(&arena, &source, options);
        let native = compile_native_sfc_with_scss(&arena, &source, options);
        if let Some((_, css)) = REQUIRED.iter().find(|(path, _)| *path == name) {
            let output = native
                .result()
                .unwrap_or_else(|err| panic!("{name}: {err:?}"));
            assert_eq!(output.css(), Some(*css), "{name}");
            assert!(output.css_document().is_some(), "{name}");
            required_scss += 1;
        }
        if let Ok(before) = stock.result() {
            let after = native.result().unwrap();
            assert_eq!(after.document(), before.document(), "{name}");
            assert_eq!(after.css_document(), before.css_document(), "{name}");
            assert_eq!(after.source_map(), before.source_map(), "{name}");
            assert_eq!(after.css_source_map(), before.css_source_map(), "{name}");
            stock_admitted += 1;
        }
        if let Ok(product) = native.result() {
            let js = product.document().source_map(name, &source);
            assert_eq!(product.source_map(), Some(js.as_str()), "{name}");
            modules += 1;
            if let Some(doc) = product.css_document() {
                let css = doc.source_map(name, &source);
                assert_eq!(product.css_source_map(), Some(css.as_str()), "{name}");
                styles += 1;
            }
        }
        // Broken and unsupported components still have original coordinates.
        // Exercise backwards links, including byte offsets inside UTF-8 scalars.
        let mut probe = EmitDocument::new(true);
        for i in (0..=32).rev() {
            let at = u32::try_from(source.len() * i / 32).unwrap();
            probe.push_linked("雪🦀\r\nx", Span::new(at, at));
        }
        assert_eq!(
            index.source_map(&probe, name),
            probe.source_map(name, &source),
            "{name}"
        );
        files += 1;
    }
    assert_eq!(required_scss, REQUIRED.len());
    assert!(files > 0 && stock_admitted > 0);
    println!(
        "source_map_corpus utf8={files} non_utf8={non_utf8} stock_admitted={stock_admitted} complete_js={modules} complete_css={styles} required_scss={required_scss} position_probes={files}"
    );
}
