//! A whole primitive-only setup excludes every original ref-call/setup candidate.

use super::PreferUseTemplateRef;
use crate::native::sfc::{
    NativeSfcLintContext, NativeSfcLintRefusal, NativeSfcSetup, NativeSfcSetupRule,
};

impl NativeSfcSetupRule for PreferUseTemplateRef {
    fn run_on_setup<'a>(
        &self,
        context: &mut NativeSfcLintContext<'_, 'a>,
        setup: &NativeSfcSetup<'_, 'a>,
    ) -> Result<(), NativeSfcLintRefusal> {
        if !core::ptr::eq(context.owner(), setup.owner()) {
            return Err(NativeSfcLintRefusal::SourceMismatch);
        }
        // The sealed original VueSetup certifies the complete direct-primitive
        // Program from the actual File walk. No CallExpression initializer or
        // setup-function body exists, so no candidate needs template membership.
        // The product host still validates the entire original template after
        // this callback; this proof cannot skip a later refusal.
        Ok(())
    }
}
