//! Original outside bytes keep typed refusals and the explicit lazy successor.

use super::*;

pub(crate) fn capture() -> (Vec<Value>, Vec<String>) {
    let mut captures = Vec::new();
    let mut failures = Vec::new();
    let pack: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_original_input_model_ssr_contexts_vue_3_5_35.json"
    ))
    .expect("immutable original outside inputs");
    let rows = pack["refusals"]
        .as_array()
        .expect("original twelve refusals");
    for row in rows {
        let id = row["id"].as_str().expect("original id");
        let source = row["source"].as_str().expect("original source");
        let current_lazy = id == "modifier"
            && source
                == "<script setup>let draft='x'</script><template><input v-model.lazy='draft'/></template>";
        let arena = Allocator::default();
        let compilation = compile_native_vapor_setup_sfc(&arena, source, Default::default());
        let result = match compilation.result() {
            Ok(output) => {
                if !current_lazy {
                    failures.push(cstr!(
                        "{id}: unsupported input emitted a whole Vapor module"
                    ));
                }
                serde_json::json!({"state":if current_lazy {"emitted"} else {"unexpected-emitted"},"code":output.code(),"nativeMapRaw":output.source_map()})
            }
            Err(error) => {
                if current_lazy {
                    failures.push(cstr!("{id}: current lazy source refused"));
                }
                let stage = match error {
                    NativeVaporSetupSfcCompileError::Source(_) => "source",
                    NativeVaporSetupSfcCompileError::Vapor(_) => "vapor",
                    NativeVaporSetupSfcCompileError::Analysis(_) => "analysis",
                    NativeVaporSetupSfcCompileError::Orchestration => "orchestration",
                };
                serde_json::json!({"state":"refused","stage":stage,"error":cstr!("{error:?}")})
            }
        };
        captures.push(serde_json::json!({"id":id,"source":source,"result":result}));
    }
    (captures, failures)
}
