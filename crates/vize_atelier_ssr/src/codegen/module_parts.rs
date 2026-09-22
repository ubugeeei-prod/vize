//! Module assembly around the `ssrRender` body: the helper-import preamble
//! and the `_temp` bindings merged-props elements allocate
//! (`@vue/compiler-ssr` declares them once at the top of the render body).

use super::SsrCodegenContext;
use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, cstr};

impl SsrCodegenContext<'_> {
    /// A fresh `_tempN` binding, declared at the top of `ssrRender`.
    pub(crate) fn alloc_temp(&mut self) -> String {
        let temp = cstr!("_temp{}", self.temps);
        self.temps += 1;
        temp
    }

    /// Record where the render body starts, after the signature and the
    /// CSS-variable prelude.
    pub(super) fn mark_body_start(&mut self) {
        self.body_start = self.out.len();
    }

    /// Declare every allocated temp at the body start: `let _temp0, _temp1`.
    pub(super) fn declare_temps(&mut self) {
        if self.temps == 0 {
            return;
        }
        let names = (0..self.temps)
            .map(|index| cstr!("_temp{index}"))
            .collect::<std::vec::Vec<_>>()
            .join(", ");
        let declaration = cstr!("  let {names}\n");
        // Links recorded after the body start move with the text they cover.
        self.out.insert_str(self.body_start, &declaration);
    }

    /// Build the preamble with imports
    pub(super) fn build_preamble(&self) -> String {
        let mut preamble = String::default();

        // SSR helpers from @vue/server-renderer
        if !self.ssr_helpers.is_empty() {
            preamble.push_str("import { ");
            let mut ssr_helpers: Vec<_> = self.ssr_helpers.iter().copied().collect();
            ssr_helpers.sort();
            push_helper_imports(&mut preamble, &ssr_helpers);
            preamble.push_str(" } from \"@vue/server-renderer\"\n");
        }

        // Core helpers from vue
        if !self.core_helpers.is_empty() {
            preamble.push_str("import { ");
            let mut core_helpers: Vec<_> = self.core_helpers.iter().copied().collect();
            core_helpers.sort();
            push_helper_imports(&mut preamble, &core_helpers);
            preamble.push_str(" } from \"vue\"\n");
        }

        preamble
    }
}

fn push_helper_imports(out: &mut String, helpers: &[RuntimeHelper]) {
    for (index, helper) in helpers.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let name = helper.name();
        out.push_str(name);
        out.push_str(" as _");
        out.push_str(name);
    }
}
