//! Module assembly around the `ssrRender` body: the helper-import preamble,
//! the resolved directive locals, and the `_temp` bindings merged-props
//! elements allocate (`@vue/compiler-ssr` declares both once at the top of
//! the render body).

use super::SsrCodegenContext;
use super::element::props::quoted_js_string;
use vize_atelier_core::RuntimeHelper;
use vize_l0::{String, cstr};
use vize_l2::op::vue::asset_identifier;

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

    /// The `_directive_<name>` local for a directive resolved by name. It is
    /// declared once at the top of `ssrRender`, as Vue declares it; Nuxt's
    /// directive auto-import rewrites only that declaration.
    pub(crate) fn resolved_directive(&mut self, name: &str) -> String {
        if !self
            .resolved_directives
            .iter()
            .any(|resolved| resolved == name)
        {
            self.resolved_directives.push(String::from(name));
        }
        asset_identifier("directive", name)
    }

    /// Declare every resolved directive ahead of the CSS-variable prelude and
    /// the temps: `const _directive_focus = _resolveDirective("focus")`.
    pub(super) fn declare_directives(&mut self) {
        if self.resolved_directives.is_empty() {
            return;
        }
        let mut declarations = String::default();
        for name in &self.resolved_directives {
            declarations.push_str("  const ");
            declarations.push_str(&asset_identifier("directive", name));
            declarations.push_str(" = _resolveDirective(");
            declarations.push_str(&quoted_js_string(name));
            declarations.push_str(")\n");
        }
        self.out.insert_str(self.assets_start, &declarations);
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
            preamble.push_str(" } from ");
            vize_l0::js_string::push_js_string_literal(
                &mut preamble,
                self.runtime_module_name.as_str(),
            );
            preamble.push('\n');
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
