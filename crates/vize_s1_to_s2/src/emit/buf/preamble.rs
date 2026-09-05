//! The helper preamble: the function-mode destructure or the module-mode
//! import, then the hoist block.

use vize_s0::{String, ToCompactString};

use super::super::options::{DomEmitMode, DomEmitOptions};
use super::Buf;

impl Buf {
    /// Preamble, helpers in import-rank order — the function-mode
    /// destructure or the module-mode import, per `options.mode` — then
    /// any root static-props hoist (the shipped codegen appends hoists to
    /// the helper preamble in both modes).
    #[cfg(test)]
    pub(in crate::emit) fn preamble(&self, options: &DomEmitOptions<'_>) -> String {
        self.preamble_with_imports_len(options).0
    }

    pub(in crate::emit) fn preamble_with_imports_len(
        &self,
        options: &DomEmitOptions<'_>,
    ) -> (String, usize) {
        let listed = self.ordered_helpers();
        if listed.is_empty() {
            return (String::default(), 0);
        }
        let mut preamble = String::default();
        match options.mode {
            DomEmitMode::Function => {
                preamble.push_str("const { ");
                for (i, helper) in listed.iter().enumerate() {
                    if i > 0 {
                        preamble.push_str(", ");
                    }
                    preamble.push_str(helper.name());
                    preamble.push_str(": ");
                    preamble.push_str(helper.alias());
                }
                preamble.push_str(" } = ");
                preamble.push_str(options.runtime_global_name);
                preamble.push('\n');
            }
            DomEmitMode::Module => {
                preamble.push_str("import { ");
                for (i, helper) in listed.iter().enumerate() {
                    if i > 0 {
                        preamble.push_str(", ");
                    }
                    preamble.push_str(helper.name());
                    preamble.push_str(" as ");
                    preamble.push_str(helper.alias());
                }
                preamble.push_str(" } from \"");
                preamble.push_str(options.runtime_module_name);
                preamble.push_str("\"\n");
            }
        }
        let imports_len = preamble.len();
        if !self.hoists.is_empty() {
            preamble.push('\n');
            for (i, rhs) in self.hoists.iter().enumerate() {
                preamble.push_str("const _hoisted_");
                preamble.push_str((i + 1).to_compact_string().as_str());
                preamble.push_str(" = ");
                preamble.push_str(rhs.as_str());
                preamble.push('\n');
            }
        }
        (preamble, imports_len)
    }
}
