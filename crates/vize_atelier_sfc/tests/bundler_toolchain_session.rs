//! Compare build metadata on the actual open, formatted, damaged and repaired buffers.
#![cfg(all(feature = "bundler-block-products", not(target_arch = "wasm32")))]
use vize_atelier_sfc::{
    bundler::{
        BundlerBlockProducts, extract_custom_blocks, extract_src_info, extract_style_blocks,
        has_scoped_style,
    },
    parse_sfc,
};

#[test]
#[ignore = "uses the actual document history supplied by the editor qualification job"]
fn bundler_metadata_tracks_the_actual_toolchain_revisions() {
    let history =
        std::fs::read_to_string(std::env::var_os("VIZE_TOOLCHAIN_HISTORY").unwrap()).unwrap();
    let revisions: Vec<_> = history.split('\0').collect();
    assert!((4..=6).contains(&revisions.len()));
    let name = Some("Setup雪🌸.vue");
    for (i, text) in revisions.into_iter().enumerate() {
        let descriptor = parse_sfc(text, Default::default()).unwrap();
        let shared = BundlerBlockProducts::from_descriptor(&descriptor);
        let separate = BundlerBlockProducts {
            styles: extract_style_blocks(text, name),
            custom_blocks: extract_custom_blocks(text, name),
            src: extract_src_info(text, name),
            has_scoped_style: has_scoped_style(text, name),
        };
        assert_eq!(shared, separate, "revision {}", i + 1);
        assert_eq!(shared, BundlerBlockProducts::read(text, name));
        assert_eq!(shared.styles.len(), usize::from(i != 3));
        assert_eq!(shared.has_scoped_style, i == 5);
        assert!(shared.custom_blocks.is_empty());
    }
}
