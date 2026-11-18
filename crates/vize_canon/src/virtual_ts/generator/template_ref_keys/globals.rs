//! Refine real Vue globals without replacing authored ambient signatures.

use super::TemplateRefUsage;
use vize_carton::append;

impl TemplateRefUsage {
    pub(in crate::virtual_ts::generator) fn emit_inference_helpers(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
    ) {
        let name = self.helper_name();
        if self.calls.iter().any(|call| call.imported) {
            append!(
                *ts,
                "  const {name} = (undefined as unknown as $UseTemplateRef);\n"
            );
        }
        // A binding imported through another module (`#imports`, a `paths`
        // alias, a re-export chain) is Vue's exactly when its type is Vue's
        // own `useTemplateRef`; a same-name user function keeps its signature.
        if self.calls.iter().any(|call| call.owner_checked) {
            append!(
                *ts,
                "  type {name}Owns<_F> = (<_G>() => _G extends _F ? 1 : 2) extends (<_G>() => _G extends typeof import('vue').useTemplateRef ? 1 : 2) ? true : false;\n  type {name}For<_F> = {name}Owns<_F> extends true ? $UseTemplateRef : _F;\n"
            );
        }
        if self
            .calls
            .iter()
            .all(|call| call.imported || call.owner_checked)
        {
            return;
        }
        // Collection admits globals only with explicit Vue ownership. Retain
        // their mapped callee so a missing value is diagnosed at the authored
        // call, without a second lookup in an unmapped helper declaration.
        append!(*ts, "  type {name}IsVue = true;\n");
        append!(*ts, "  type {name}Global = $UseTemplateRef;\n");
    }
}
