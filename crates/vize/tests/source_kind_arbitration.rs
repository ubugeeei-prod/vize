use vize::artifact_graph::{VizeGraphConfig, create_compilation};
use vize::atelier_sfc::{
    SFC_SOURCE_KIND, SfcCompileProduct, SfcCompileRequest, install_sfc_compile_request,
};
use vize::module::ModuleSyntaxProduct;
use vize_atlas::SourceKindInput;

const SCRIPTED: &str = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template><button @click="count++">{{ count }}</button></template>"#;

#[test]
fn explicit_sfc_kind_arbitrates_virtual_ts_and_tsx_with_the_full_registry() {
    for name in ["/virtual/Card.setup.ts", "/virtual/Card.setup.tsx"] {
        let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
        let source = compilation.add_source(name, SCRIPTED).unwrap();
        install_sfc_compile_request(&mut compilation, source, SfcCompileRequest::default())
            .unwrap();

        assert!(
            compilation
                .source_input::<SourceKindInput>(source)
                .is_some_and(|kind| kind.is(SFC_SOURCE_KIND))
        );
        let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
        assert!(compiled.value().code.contains("$setup.count"), "{name}");
        assert!(compiled.plan().contains::<ModuleSyntaxProduct>());
    }
}
