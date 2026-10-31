//! Refine real Vue globals without replacing authored ambient signatures.

use super::TemplateRefUsage;
use vize_carton::{String, append};

impl TemplateRefUsage {
    pub(in crate::virtual_ts::generator) fn emit_inference_helpers(&self, ts: &mut String) {
        let name = self.helper_name();
        if self.calls.iter().any(|call| call.imported) {
            append!(
                *ts,
                "  const {name} = (undefined as unknown as $UseTemplateRef);\n"
            );
        }
        if self.calls.iter().all(|call| call.imported) {
            return;
        }
        // Collection admits globals only with explicit Vue ownership. Retain
        // their mapped callee so a missing value is diagnosed at the authored
        // call, without a second lookup in an unmapped helper declaration.
        append!(*ts, "  type {name}IsVue = true;\n");
        append!(*ts, "  type {name}Global = $UseTemplateRef;\n");
    }
}
