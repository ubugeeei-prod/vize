use super::{
    BatchCompileOptionsNapi, BatchFileInputNapi, SfcCompileOptionsNapi, compile_sfc,
    compile_sfc_batch_with_results,
};

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/compiler/page-meta-runtime/Page.vue.txt"
));

#[test]
fn single_and_parallel_batch_page_meta_require_the_same_explicit_nuxt_option() {
    for ssr in [false, true] {
        for vapor in [false, true] {
            for nuxt in [false, true] {
                let single = compile_sfc(
                    SOURCE.into(),
                    Some(SfcCompileOptionsNapi {
                        filename: Some("Page.vue".into()),
                        ssr: Some(ssr),
                        vapor: Some(vapor),
                        is_prod: Some(true),
                        nuxt_page_meta: Some(nuxt),
                        ..Default::default()
                    }),
                )
                .expect("compile original single source");
                let batch = compile_sfc_batch_with_results(
                    vec![BatchFileInputNapi {
                        path: "Page.vue".into(),
                        source: SOURCE.into(),
                    }],
                    Some(BatchCompileOptionsNapi {
                        ssr: Some(ssr),
                        vapor: Some(vapor),
                        is_prod: Some(true),
                        nuxt_page_meta: Some(nuxt),
                        include_macro_artifacts: Some(true),
                        ..Default::default()
                    }),
                )
                .expect("compile original parallel source");
                let file = batch.results.first().expect("original result");
                assert_eq!(single.errors, Vec::<String>::new());
                assert_eq!(file.errors, Vec::<String>::new());
                assert_eq!(single.code, file.code);
                assert_eq!(
                    single.code.contains("definePageMeta("),
                    !nuxt,
                    "{}",
                    single.code
                );
                assert_eq!(single.code.contains("#imports"), !nuxt);
                assert_eq!(single.macro_artifacts.len(), usize::from(nuxt));
                assert_eq!(file.macro_artifacts.len(), usize::from(nuxt));
            }
        }
    }
}
