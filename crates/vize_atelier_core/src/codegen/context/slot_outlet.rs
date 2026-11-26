//! Final arguments for a generated slot outlet.

use super::CodegenContext;

impl CodegenContext {
    #[inline(always)]
    pub(in crate::codegen) fn finish_slot_outlet(
        &mut self,
        default_suffix: &'static str,
        has_fallback: bool,
        has_props: bool,
    ) {
        if self.no_slotted {
            self.finish_no_slotted_outlet(has_fallback, has_props);
        } else {
            self.push(default_suffix);
        }
    }

    #[cold]
    #[inline(never)]
    fn finish_no_slotted_outlet(&mut self, has_fallback: bool, has_props: bool) {
        if has_fallback {
            self.push("]");
        }
        if !has_fallback {
            if !has_props {
                self.push(", {}");
            }
            self.push(", undefined");
        }
        self.push(", true");
        self.push(")");
    }
}
