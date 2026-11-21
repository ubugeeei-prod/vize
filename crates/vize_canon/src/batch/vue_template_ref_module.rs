//! Whether a module re-exports Vue's own `useTemplateRef`.
//!
//! Nuxt maps `#imports` to a generated declaration file that re-exports Vue's
//! composables (`export { useTemplateRef } from 'vue'`). An explicit import
//! from that alias is Vue's composable by provider origin, not by structure:
//! a separately authored function with Vue's signature keeps its own contract
//! (#10980), so only a re-export from the `vue` package itself qualifies.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportOrExportKind, ModuleExportName, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Whether `source` (a module or declaration file) exports `useTemplateRef`
/// as a value re-export of the `vue` package.
pub fn reexports_vue_template_ref(source: &str) -> bool {
    if !source.contains("useTemplateRef") && !source.contains("export *") {
        return false;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::d_ts()).parse();
    if parsed.panicked {
        return false;
    }
    exports_template_ref(&parsed.program)
}

#[inline]
pub(super) fn exports_template_ref(program: &oxc_ast::ast::Program<'_>) -> bool {
    program.body.iter().any(|statement| match statement {
        Statement::ExportNamedDeclaration(export) => {
            export.export_kind != ImportOrExportKind::Type
                && export.source.as_ref().is_some_and(|source| {
                    super::vue_template_ref_global::is_vue_package(&source.value)
                })
                && export.specifiers.iter().any(|specifier| {
                    specifier.export_kind != ImportOrExportKind::Type
                        && export_name(&specifier.local) == Some("useTemplateRef")
                        && export_name(&specifier.exported) == Some("useTemplateRef")
                })
        }
        Statement::ExportAllDeclaration(export) => {
            export.exported.is_none()
                && export.export_kind != ImportOrExportKind::Type
                && super::vue_template_ref_global::is_vue_package(&export.source.value)
        }
        _ => false,
    })
}

fn export_name<'a>(name: &'a ModuleExportName<'_>) -> Option<&'a str> {
    match name {
        ModuleExportName::IdentifierName(name) => Some(name.name.as_str()),
        ModuleExportName::IdentifierReference(name) => Some(name.name.as_str()),
        ModuleExportName::StringLiteral(name) => Some(name.value.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::reexports_vue_template_ref;

    #[test]
    fn vue_reexports_are_recognized() {
        for source in [
            "export { useTemplateRef } from 'vue'\n",
            "export { computed, useTemplateRef, watch } from \"vue\";\n",
            "export {\n  ref,\n  useTemplateRef,\n} from '../node_modules/vue'\n",
            "export * from 'vue'\n",
        ] {
            assert!(reexports_vue_template_ref(source), "{source}");
        }
    }

    #[test]
    fn other_modules_and_type_exports_are_not() {
        for source in [
            "export { useTemplateRef } from 'my-vue'\n",
            "export type { useTemplateRef } from 'vue'\n",
            "export { type useTemplateRef } from 'vue'\n",
            "export { useTemplateRefsList } from '@vueuse/core'\n",
            "export { useTemplateRef as useRef } from 'vue'\n",
            "export { ref as useTemplateRef } from 'vue'\n",
            "export * as vue from 'vue'\n",
            "export declare const useTemplateRef: typeof import('vue').useTemplateRef\n",
            "export const useTemplateRef = (key: string) => key\n",
        ] {
            assert!(!reexports_vue_template_ref(source), "{source}");
        }
    }
}
