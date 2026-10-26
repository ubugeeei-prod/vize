//! Project evidence for whether server rendering is possible.
//!
//! A client build is also part of a universal application. Vite build/compile
//! target flags therefore do not establish a client-only application.

use std::path::Path;

use oxc_ast::ast::Expression;

use super::project_nuxt::{self, Lookup};

pub(super) fn ssr_enabled(root: &Path) -> bool {
    // Unreadable, dynamic or invalid configuration cannot prove that an
    // otherwise reachable browser read never runs on a server.
    project_nuxt::find(root)
        .is_none_or(|config| config.source.as_deref().and_then(nuxt_ssr) != Some(false))
}

fn nuxt_ssr(source: &str) -> Option<bool> {
    project_nuxt::read(source, |config| match config.get("ssr") {
        Lookup::Value(Expression::BooleanLiteral(value)) => Some(value.value),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::{nuxt_ssr, ssr_enabled};

    #[test]
    fn source_directories_inherit_app_policy_but_nested_projects_do_not() {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(
            project.path().join("nuxt.config.ts"),
            "export default defineNuxtConfig({ssr:false})",
        )
        .unwrap();
        let source = project.path().join("src");
        std::fs::create_dir_all(&source).unwrap();
        assert!(!ssr_enabled(&source));
        std::fs::write(source.join("package.json"), "{}").unwrap();
        assert!(ssr_enabled(&source));
    }

    #[test]
    fn only_complete_default_export_configuration_can_disable_ssr() {
        for source in [
            "export default defineNuxtConfig({ ssr: false })",
            "export default ({ 'ssr': false } as const)",
            "export default defineNuxtConfig({ ...base, ssr: false } satisfies NuxtConfig)",
            "export default defineNuxtConfig({ ssr: true, ssr: false })",
            "import { defineNuxtConfig } from 'nuxt/config'; export default defineNuxtConfig({ ssr: false })",
            // #10608: environment blocks that cannot turn SSR back on.
            "export default defineNuxtConfig({ ssr: false, $development: {} })",
            "export default defineNuxtConfig({ ssr: false, $development: { devtools: { enabled: false } } })",
            "export default defineNuxtConfig({ ssr: false, $production: { ssr: false } })",
        ] {
            assert_eq!(nuxt_ssr(source), Some(false), "{source}");
        }
        for source in [
            "export default defineNuxtConfig({ ssr: false, ...base })",
            "export default defineNuxtConfig({ ssr: false, [key]: true })",
            "export default defineNuxtConfig({ ssr: false, $production: { ssr: true } })",
            "export default defineNuxtConfig({ ssr: false, $production: { ...base } })",
            "export default defineNuxtConfig({ ssr: false, $development: overrides })",
            "export default defineNuxtConfig({ $production: { ssr: false } })",
            "export default defineNuxtConfig({ ssr: process.env.SSR })",
            "export default transform({ ssr: false })",
            "const defineNuxtConfig = () => ({ssr:true}); export default defineNuxtConfig({ssr:false})",
            "import { defineNuxtConfig } from 'unrelated'; export default defineNuxtConfig({ssr:false})",
            "export const ssr = false; export default defineNuxtConfig({})",
            "export default defineNuxtConfig({ app: { ssr: false } })",
            "// ssr: false\nexport default defineNuxtConfig({})",
        ] {
            assert_eq!(nuxt_ssr(source), None, "{source}");
        }
        assert_eq!(
            nuxt_ssr("export default defineNuxtConfig({ ssr: false, ssr: true })"),
            Some(true)
        );
    }
}
