//! Refine real Vue globals without replacing authored ambient signatures.

use super::TemplateRefUsage;
use vize_carton::{String, append};

impl TemplateRefUsage {
    pub(in crate::virtual_ts::generator) fn emit_inference_helpers(&self, ts: &mut String) {
        let name = self.helper_name();
        if self.calls.iter().any(|call| call.imported) {
            append!(
                *ts,
                "  const {name} = (undefined as unknown as __VizeUseTemplateRef);\n"
            );
        }
        if self.calls.iter().all(|call| call.imported) {
            return;
        }
        // The source callee is retained in the mapped expression. An absent
        // runtime import is diagnosed there; this unowned lookup cannot create
        // an ambient value or make a missing name valid.
        append!(*ts, "  type {name}Source = typeof useTemplateRef;\n");
        // Function assignability alone ignores readonly and generic defaults.
        // Bidirectional identity plus return identity preserve custom mutable,
        // defaulted and overloaded ambient contracts without `any` fallbacks.
        append!(
            *ts,
            "  type {name}Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? (<T>() => T extends B ? 1 : 2) extends (<T>() => T extends A ? 1 : 2) ? true : false : false;\n"
        );
        append!(
            *ts,
            "  type {name}Result<F> = F extends (...args: never[]) => infer R ? R : never;\n"
        );
        append!(
            *ts,
            "  type {name}IsVue = 0 extends (1 & {name}Source) ? false : {name}Equal<{name}Source, typeof import('vue').useTemplateRef> extends true ? {name}Equal<{name}Result<{name}Source>, {name}Result<typeof import('vue').useTemplateRef>> : false;\n"
        );
        append!(
            *ts,
            "  type {name}Global = {name}IsVue extends true ? __VizeUseTemplateRef : {name}Source;\n"
        );
    }
}
