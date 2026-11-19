//! Module assembly around the `ssrRender` body: the helper-import preamble
//! and the `_temp` bindings merged-props elements allocate
//! (`@vue/compiler-ssr` declares them once at the top of the render body).

use super::SsrCodegenContext;
use vize_atelier_core::RuntimeHelper;
use vize_l0::{String, cstr};
use vize_l4::runtime::{Helper, VUE_SERVER_RENDERER, server_renderer};

/// The server-rendering vocabulary module imported as itself; every other
/// module is the host's Vue runtime.
const SERVER_RENDERER_MODULE: &str = "@vue/server-renderer";

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

    /// The import preamble: one `import { … }` per vocabulary module the body
    /// used, `@vue/server-renderer` before the Vue runtime, each listing its
    /// helpers in vocabulary order.
    pub(super) fn build_preamble(&self) -> String {
        let mut preamble = String::default();
        let mut open: Option<&str> = None;
        for helper in self.helpers.in_index_order() {
            let Some((module, name)) = VUE_SERVER_RENDERER.export(helper) else {
                continue;
            };
            if open == Some(module) {
                preamble.push_str(", ");
            } else {
                if let Some(previous) = open {
                    self.close_import(&mut preamble, previous);
                }
                preamble.push_str("import { ");
                open = Some(module);
            }
            preamble.push_str(name);
            preamble.push_str(" as _");
            preamble.push_str(name);
        }
        if let Some(module) = open {
            self.close_import(&mut preamble, module);
        }
        preamble
    }

    /// End the import list from `module`; the Vue runtime import names the
    /// host's runtime module.
    fn close_import(&self, preamble: &mut String, module: &str) {
        if module == SERVER_RENDERER_MODULE {
            preamble.push_str(" } from \"@vue/server-renderer\"\n");
        } else {
            preamble.push_str(" } from ");
            vize_l0::js_string::push_js_string_literal(preamble, self.runtime_module_name.as_str());
            preamble.push('\n');
        }
    }
}

/// The legacy walker's `helper` in the server-rendering vocabulary. Both
/// tables list Vue's exports in the same order, so the index is arithmetic on
/// the discriminant. `resolveFilter`, which Vue 3 does not export, has none;
/// SSR never requests it.
pub(crate) fn legacy_helper(helper: RuntimeHelper) -> Option<Helper> {
    let id = helper as u8;
    let ssr = RuntimeHelper::SsrInterpolate as u8;
    let filter = RuntimeHelper::ResolveFilter as u8;
    let vue = server_renderer::FRAGMENT.index();
    let index = if id >= ssr {
        id - ssr
    } else if id < filter {
        vue + id
    } else if id > filter {
        vue + id - 1
    } else {
        return None;
    };
    Helper::from_index(index)
}

#[cfg(test)]
mod tests;
