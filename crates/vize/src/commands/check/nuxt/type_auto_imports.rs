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
        // A Windows path resolved against the generated directory keeps its
        // separators as `/`, which TypeScript reads the same way and which
        // the virtual project's mirror lookup still matches. Elsewhere a
        // backslash is part of a file name and is escaped when rendered.
        let specifier = if std::path::MAIN_SEPARATOR == '\\' {
            specifier.replace('\\', "/").to_compact_string()
        } else {
            specifier.to_compact_string()
        };
        let export = if local == exported {
            exported.to_compact_string()
        } else {
            cstr!("{local} as {exported}")
        };
        match self
            .modules
            .iter_mut()
            .find(|(module, _)| *module == specifier)
        {
            Some((_, exports)) => exports.push(export),
            None => self.modules.push((specifier, vec![export])),
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
            let literal = string_literal_body(specifier);
            body.push_str("  export type { ");
            for (index, export) in exports.iter().enumerate() {
                if index > 0 {
                    body.push_str(", ");
                }
                body.push_str(export.as_str());
            }
            body.push_str(cstr!(" }} from {quote}{literal}{quote}\n").as_str());
            body.push_str(cstr!("  import({quote}{literal}{quote})\n").as_str());
        }
        body.push_str("}\n");
        Some(render_module_augmentation_stub(body.as_str()))
    }
}

/// `specifier` spelled as the inside of a string literal. The parsed
/// declaration hands over the decoded value, so a Windows path's backslashes
/// would otherwise start escape sequences, and `\u` is not even a valid one.
/// Both quote characters stay as they are: the caller picks the delimiter the
/// specifier does not hold.
fn string_literal_body(specifier: &str) -> String {
    let mut literal = String::with_capacity(specifier.len());
    for ch in specifier.chars() {
        match ch {
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            ch if ch.is_control() => {
                literal.push_str(cstr!("\\u{:04x}", u32::from(ch)).as_str());
            }
            ch => literal.push(ch),
        }
    }
    literal
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

    #[test]
    fn backslashes_and_control_characters_are_escaped() {
        let mut types = TypeAutoImports::default();
        types.reexport("C:\\repo\\types\\user", "User", "User");
        types.reexport("/odd\tname\u{1b}/it's", "Odd", "Odd");
        // A Windows separator becomes `/`; elsewhere it is a file name byte.
        let user = if std::path::MAIN_SEPARATOR == '\\' {
            "C:/repo/types/user"
        } else {
            "C:\\\\repo\\\\types\\\\user"
        };

        assert_eq!(
            types.render_stub().as_deref(),
            Some(
                vize_l0::cstr!(
                    "// @vize-module-augmentation\n\
                     declare global {{\n  \
                     export type {{ User }} from '{user}'\n  \
                     import('{user}')\n  \
                     export type {{ Odd }} from \"/odd\\tname\\u001b/it's\"\n  \
                     import(\"/odd\\tname\\u001b/it's\")\n\
                     }}\n"
                )
                .as_str()
            )
        );
    }
}
