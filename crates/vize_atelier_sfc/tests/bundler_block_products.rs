#![cfg(all(feature = "bundler-block-products", not(target_arch = "wasm32")))]
#![expect(
    clippy::unwrap_used,
    reason = "the independent descriptors and allocator are checked by this test"
)]
use davinci_harness::alloc;
use std::{alloc::System, hint::black_box};
use vize_atelier_sfc::{
    bundler::{
        BundlerBlockProducts, extract_custom_blocks, extract_src_info, extract_style_blocks,
        has_scoped_style,
    },
    parse_sfc,
};

#[global_allocator]
static ALLOCATOR: alloc::CountingAllocator<System> = alloc::CountingAllocator::system();

const CASES: &[(&str, &str)] = &[
    ("empty", ""),
    ("template", "<template><div/></template>"),
    (
        "mixed",
        "<template src='@/view.html'></template>\r\n<script setup src='setup.ts'/><script src='ordinary.ts'/>\n<style module scoped lang='scss'>.foo{}</style><i18n src='en.json' key bare/>\n<style module='' src='外部.css'/>",
    ),
    (
        "unicode",
        "<template>😀<img src='./nested.png'></template><i18n Z='😀' a='é'>値</i18n><style scoped> .é{} </style>",
    ),
    (
        "invalid",
        "<style scoped>.x{}</style><template/><template/>",
    ),
    (
        "rspack",
        include_str!("../../../npm/builder/rspack/src/test/fixtures/scoped-multi-blocks/App.vue"),
    ),
];

fn separate(source: &str) -> BundlerBlockProducts {
    let name = Some("App.vue");
    BundlerBlockProducts {
        styles: extract_style_blocks(source, name),
        custom_blocks: extract_custom_blocks(source, name),
        src: extract_src_info(source, name),
        has_scoped_style: has_scoped_style(source, name),
    }
}

#[test]
fn selected_descriptor_and_independent_public_operations_match() {
    for &(case, source) in CASES {
        let combined = BundlerBlockProducts::read(source, Some("App.vue"));
        assert_eq!(combined, separate(source), "{case}");
        if let Ok(descriptor) = parse_sfc(source, Default::default()) {
            assert_eq!(
                combined,
                BundlerBlockProducts::from_descriptor(&descriptor),
                "{case}"
            );
        }
    }
    let mixed = BundlerBlockProducts::read(CASES[2].1, None);
    assert_eq!(mixed.src.script_src.as_deref(), Some("ordinary.ts"));
    assert_eq!(mixed.src.template_src.as_deref(), Some("@/view.html"));
    assert!(mixed.has_scoped_style && mixed.styles.len() == 2 && mixed.custom_blocks.len() == 1);
}

#[test]
#[ignore = "process-exclusive allocation windows for all four complete builder products"]
fn bundler_block_product_allocations() {
    alloc::mark_installed();
    for &(case, source) in CASES {
        let independent = alloc::measure(|| separate(black_box(source))).unwrap();
        let combined =
            alloc::measure(|| BundlerBlockProducts::read(black_box(source), Some("App.vue")))
                .unwrap();
        assert!(
            combined.calls <= independent.calls
                && combined.peak_bytes_over_start <= independent.peak_bytes_over_start,
            "{case}"
        );
        eprintln!(
            "bundler_blocks case={case} calls={}/{} peak={}/{}",
            independent.calls,
            combined.calls,
            independent.peak_bytes_over_start,
            combined.peak_bytes_over_start
        );
        if let Ok(descriptor) = parse_sfc(source, Default::default()) {
            let retained =
                alloc::measure(|| BundlerBlockProducts::from_descriptor(black_box(&descriptor)))
                    .unwrap();
            assert!(
                retained.calls <= combined.calls
                    && retained.peak_bytes_over_start <= combined.peak_bytes_over_start,
                "{case}"
            );
            eprintln!(
                "bundler_blocks_retained case={case} calls={} peak={}",
                retained.calls, retained.peak_bytes_over_start
            );
        }
    }
}
