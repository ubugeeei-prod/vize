//! Original outside inputs retain all refusals and the explicit lazy successor.

use super::*;

pub(crate) fn capture() -> (Vec<Value>, Vec<String>) {
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
        let current_lazy = id == "modifier"
            && source
                == "<script setup>let draft='x'</script><template><input v-model.lazy='draft'/></template>";
        let arena = Allocator::default();
        let compilation =
            compile_native_setup_ssr_sfc(&arena, source, NativeSsrSfcCompileOptions::default());
        let state = match compilation.result() {
            Ok(output) => {
                if !current_lazy {
                    failures.push(cstr!("{id}: unsupported Model emitted a whole SSR module"));
                }
                serde_json::json!({"state":if current_lazy {"emitted"} else {"unexpected-emitted"},"code":output.code(),"nativeMapRaw":output.source_map()})
            }
            Err(error) => {
                if current_lazy {
                    failures.push(cstr!("{id}: current lazy source refused"));
                }
                serde_json::json!({"state":"refused","error":cstr!("{error:?}")})
            }
        };
        observations.push(serde_json::json!({"id":id,"source":source,"result":state}));
    }
    (observations, failures)
}
