//! Outside this Model family, whole public compilation exposes no prefix module.

use super::*;

pub(super) fn capture() -> (Vec<Value>, Vec<String>) {
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    for (id, source) in [
        (
            "const",
            "<script setup>const draft='x'</script><template><input v-model='draft'/></template>",
        ),
        (
            "unresolved",
            "<script setup>let other='x'</script><template><input v-model='draft'/></template>",
        ),
        (
            "member",
            "<script setup>let draft='x'</script><template><input v-model='draft.value'/></template>",
        ),
        (
            "modifier",
            "<script setup>let draft='x'</script><template><input v-model.lazy='draft'/></template>",
        ),
        (
            "argument",
            "<script setup>let draft='x'</script><template><input v-model:value='draft'/></template>",
        ),
        (
            "late-header",
            "<script setup>let draft='x'</script><template><input v-model='draft' :class='draft'/></template>",
        ),
        (
            "sibling",
            "<script setup>let draft='x'</script><template><input v-model='draft'/><b/></template>",
        ),
        (
            "nested",
            "<script setup>let draft='x'</script><template><div><input v-model='draft'/></div></template>",
        ),
        (
            "unknown-initializer",
            "<script setup>let draft</script><template><input v-model='draft'/></template>",
        ),
        (
            "nul-string",
            "<script setup>let draft='a\\0b'</script><template><input v-model='draft'/></template>",
        ),
        (
            "cr-string",
            "<script setup>let draft='a\\rb'</script><template><input v-model='draft'/></template>",
        ),
        (
            "lone-surrogate",
            "<script setup>let draft='\\uD800'</script><template><input v-model='draft'/></template>",
        ),
    ] {
        for (mode, mode_name) in [
            (DomMode::Development, "development"),
            (DomMode::Production, "production"),
        ] {
            let arena = Allocator::default();
            let compilation = compile_native_selected_setup_sfc_dom_with_mode(
                &arena,
                source,
                NativeSelectedSfcDomOptions::default(),
                mode,
            );
            let state = match compilation.result() {
                Ok(output) => {
                    failures.push(cstr!(
                        "{id}/{mode_name}: unsupported Model emitted a whole module"
                    ));
                    serde_json::json!({"state":"unexpected-emitted","code":output.code(),"nativeMapRaw":output.source_map()})
                }
                Err(error) => serde_json::json!({"state":"refused","error":cstr!("{error:?}")}),
            };
            observations
                .push(serde_json::json!({"id":id,"mode":mode_name,"source":source,"result":state}));
        }
    }
    (observations, failures)
}
