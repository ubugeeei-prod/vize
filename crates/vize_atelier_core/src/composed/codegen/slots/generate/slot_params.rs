// DO NOT EDIT. Generated from crates/vize_atelier_core/src/codegen/slots/generate/slot_params.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
use vize_l0::String;

use super::CodegenContext;

pub(super) fn strip_ctx_prefix_for_slot_params(ctx: &CodegenContext, content: &str) -> String {
    let mut result = String::new(content);
    for param in ctx.slot_params.keys() {
        // Replace _ctx.paramName with paramName
        let mut prefixed = String::with_capacity(5 + param.len());
        prefixed.push_str("_ctx.");
        prefixed.push_str(param);
        let replaced = result.replace(prefixed.as_str(), param.as_str());
        result = String::from(replaced);
    }
    result
}
