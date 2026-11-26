//! Auto-imported types.
//!
//! Nuxt puts an auto-imported type in scope by re-exporting it from inside a
//! global augmentation:
//!
//! ```ts
//! declare global {
//!   export type { Ref } from 'vue'
//!   import('vue')
//! }
//! ```
//!
//! The stub built here keeps that form because it is the only one that keeps
//! the type's own parameters. An alias such as
//! `type Ref<T = any> = import('vue').Ref<T>` has to restate them, and is wrong
//! for every type whose real parameter list differs. The bare `import('…')`
//! stays as well: the checker does not load a module that only such a
//! re-export names, and the type then degrades to `any` without a diagnostic.

use vize_l0::{FxHashSet, String, ToCompactString, cstr};

use super::plugins::render_module_augmentation_stub;

#[derive(Default)]
pub(super) struct TypeAutoImports {
    /// Every global type name that already has its one declaration.
    names: FxHashSet<String>,
    /// Export specifiers per module, in the order the modules were first seen.
    modules: Vec<(String, Vec<String>)>,
}

impl TypeAutoImports {
    /// Reserve `name` for a declaration made elsewhere. Returns `false` when
    /// the name is already taken, so the caller drops its own declaration.
    pub(super) fn claim(&mut self, name: &str) -> bool {
        self.names.insert(name.to_compact_string())
    }

    /// Make `exported` a global type that is `local` of the module `specifier`.
    /// The first declaration of a name wins.
    pub(super) fn reexport(&mut self, specifier: &str, local: &str, exported: &str) {
        // A specifier holding both quote characters cannot be written as a
        // plain string literal, and one malformed line would take every other
        // auto-imported type down with it.
        if specifier.contains('\'') && specifier.contains('"') {
            return;
        }
        if !self.claim(exported) {
            return;
        }
        let export = if local == exported {
            exported.to_compact_string()
        } else {
            cstr!("{local} as {exported}")
        };
        match self
            .modules
            .iter_mut()
            .find(|(module, _)| module.as_str() == specifier)
        {
            Some((_, exports)) => exports.push(export),
            None => self
                .modules
                .push((specifier.to_compact_string(), vec![export])),
        }
    }

    /// The module-augmentation stub declaring every re-export, or `None` when
    /// there is nothing to declare.
    pub(super) fn render_stub(&self) -> Option<String> {
        if self.modules.is_empty() {
            return None;
        }
        let mut body = String::from("declare global {\n");
        for (specifier, exports) in &self.modules {
            let quote = if specifier.contains('\'') { '"' } else { '\'' };
            body.push_str("  export type { ");
            for (index, export) in exports.iter().enumerate() {
                if index > 0 {
                    body.push_str(", ");
                }
                body.push_str(export.as_str());
            }
            body.push_str(cstr!(" }} from {quote}{specifier}{quote}\n").as_str());
            body.push_str(cstr!("  import({quote}{specifier}{quote})\n").as_str());
        }
        body.push_str("}\n");
        Some(render_module_augmentation_stub(body.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::TypeAutoImports;

    #[test]
    fn renders_one_reexport_per_module_in_first_seen_order() {
        let mut types = TypeAutoImports::default();
        types.reexport("vue", "Ref", "Ref");
        types.reexport("/app/composables/states", "ColorName", "ColorName");
        types.reexport("vue", "ComputedRef", "ComputedRef");
        types.reexport("/app/composables/states", "Box", "Crate");

        assert_eq!(
            types.render_stub().as_deref(),
            Some(
                "// @vize-module-augmentation\n\
                 declare global {\n  \
                 export type { Ref, ComputedRef } from 'vue'\n  \
                 import('vue')\n  \
                 export type { ColorName, Box as Crate } from '/app/composables/states'\n  \
                 import('/app/composables/states')\n\
                 }\n"
            )
        );
    }

    #[test]
    fn first_declaration_of_a_name_wins() {
        let mut types = TypeAutoImports::default();
        assert!(types.claim("Ref"));
        types.reexport("vue", "Ref", "Ref");
        types.reexport("/app/types/a", "User", "User");
        types.reexport("/app/types/b", "User", "User");
        assert!(!types.claim("User"));

        assert_eq!(
            types.render_stub().as_deref(),
            Some(
                "// @vize-module-augmentation\n\
                 declare global {\n  \
                 export type { User } from '/app/types/a'\n  \
                 import('/app/types/a')\n\
                 }\n"
            )
        );
    }

    #[test]
    fn nothing_to_declare_renders_no_stub() {
        let mut types = TypeAutoImports::default();
        assert!(types.claim("Ref"));
        types.reexport("vue", "Ref", "Ref");
        types.reexport("/both'\"quotes", "A", "A");

        assert_eq!(types.render_stub(), None);
    }

    #[test]
    fn a_quoted_specifier_switches_the_delimiter() {
        let mut types = TypeAutoImports::default();
        types.reexport("/it's/types", "A", "A");

        assert_eq!(
            types.render_stub().as_deref(),
            Some(
                "// @vize-module-augmentation\n\
                 declare global {\n  \
                 export type { A } from \"/it's/types\"\n  \
                 import(\"/it's/types\")\n\
                 }\n"
            )
        );
    }
}
