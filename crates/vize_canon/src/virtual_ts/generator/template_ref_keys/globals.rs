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
        // Structural equality with `typeof import('vue').useTemplateRef` is not
        // ownership. A separately declared ambient with the same signature must
        // keep its own return type. Value imports use the registry helper.
        append!(*ts, "  type {name}IsVue = false;\n");
        append!(*ts, "  type {name}Global = {name}Source;\n");
    }
}
