use super::*;
use vize_atelier_sfc::{native::NativeScssSession, scss::ScssProducts};

#[test]
fn whole_component_reuses_only_the_selected_original_styles_in_document_order() {
    let source = "<template><p/></template><style lang=scss>.a{color:red}</style><style>.ordinary{}</style><style lang=scss>.b{color:blue}</style>";
    let copied = source.to_owned();
    let arena = Allocator::default();
    let options = NativeSfcCompileOptions {
        source_map: true,
        ..Default::default()
    };
    let session = NativeScssSession::new(&arena, source, options);
    let foreign = NativeScssSession::new(&arena, &copied, options);
    let descriptor = session.observation().descriptor().admitted().unwrap();
    let mut styles = descriptor.styles();
    let first = ScssProducts::from_style(styles.next().unwrap()).unwrap();
    let ordinary = ScssProducts::read(styles.next().unwrap().block());
    let last = ScssProducts::from_style(styles.next().unwrap()).unwrap();
    let other = ScssProducts::from_style(
        foreign
            .observation()
            .descriptor()
            .admitted()
            .unwrap()
            .styles()
            .next()
            .unwrap(),
    )
    .unwrap();
    let expected = session.emit([]).unwrap();
    for selection in [&[&first][..], &[&last], &[&first, &last]] {
        let actual = session.emit(selection.iter().copied()).unwrap();
        assert_eq!(actual.document(), expected.document());
        assert_eq!(actual.css_document(), expected.css_document());
        assert_eq!(actual.source_map(), expected.source_map());
        assert_eq!(actual.css_source_map(), expected.css_source_map());
    }
    for selection in [
        &[&other][..],
        &[&last, &first],
        &[&first, &first],
        &[&ordinary],
    ] {
        assert!(matches!(
            session.emit(selection.iter().copied()),
            Err(NativeSfcCompileError::UnusedPreparedScss { .. })
        ));
    }
    let invalid = "<template><p/></template><style lang=scss>.a{color:$unknown}</style>";
    let broken = NativeScssSession::new(&arena, invalid, options);
    let style = broken
        .observation()
        .descriptor()
        .admitted()
        .unwrap()
        .styles()
        .next()
        .unwrap();
    let lowered = ScssProducts::from_style(style).unwrap();
    let expected = broken.emit([]).unwrap_err();
    assert!(matches!(expected, NativeSfcCompileError::Scss { .. }));
    assert_eq!(broken.emit([&lowered]).unwrap_err(), expected);
}

#[test]
#[ignore = "times complete component and editor preparation on one observation"]
fn paired_retained_styles_and_complete_components() {
    mark_installed();
    for count in [1, 8, 48] {
        let mut source =
            "<script setup>let n=1</script><template><p>{{n}}</p></template>".to_owned();
        for i in 0..count {
            source.push_str(&format!(
                "<style lang=scss>$ink:red;.c{i}{{color:$ink;.child{{width:1px}}}}</style>"
            ));
        }
        for mapped in [false, true] {
            let options = NativeSfcCompileOptions {
                source_map: mapped,
                ..Default::default()
            };
            let run = |shared| {
                let arena = Allocator::default();
                let session = NativeScssSession::new(&arena, black_box(&source), options);
                let descriptor = session.observation().descriptor().admitted().unwrap();
                let products: Vec<_> = descriptor
                    .styles()
                    .map(|style| ScssProducts::from_style(style).unwrap())
                    .collect();
                let output = if shared {
                    session.emit(&products)
                } else {
                    session.emit([])
                }
                .unwrap();
                assert!(
                    products
                        .iter()
                        .all(|p| p.classes().unwrap().names().len() == 2)
                );
                drop(black_box(output));
            };
            let (separate, shared) = paired(
                &format!("retained-component styles={count} mapped={mapped}"),
                run,
            );
            assert!(
                shared <= separate,
                "styles={count} mapped={mapped} ns={separate}/{shared}"
            );
        }
    }
}
