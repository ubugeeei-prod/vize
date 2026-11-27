//! Static facts from the `nuxt.config.*` that owns a project directory.
//!
//! Only literal values count. Anything dynamic reads as unknown so callers can
//! keep their conservative default.

pub(crate) mod host_authority;
mod static_render;

use std::path::{Component, Path, PathBuf};

use oxc_ast::ast::Expression;
pub(super) use vize_carton::nuxt::{
    ConfigObject, Lookup, NUXT_CONFIG_NAMES, find, read, unwrap_expression,
};
use vize_carton::nuxt::{optional_path, static_key};
use vize_croquis_cf::NuxtRenderDirs;

/// `srcDir`, `dir.pages` and `dir.layouts`, relative to `root` like doctor sources.
pub(super) fn render_dirs(root: &Path) -> Option<NuxtRenderDirs> {
    let config = find(root)?;
    let (src_dir, pages, layouts, pages_enabled, static_render_config) =
        read(config.source.as_deref()?, |object| {
            let src_dir = optional_path(object.get("srcDir"))?;
            let (pages, layouts) = match object.get("dir") {
                Lookup::Absent => (None, None),
                Lookup::Value(value) => {
                    let dir = ConfigObject::nested(value)?;
                    (
                        optional_path(dir.get("pages"))?,
                        optional_path(dir.get("layouts"))?,
                    )
                }
                Lookup::Unknown => return None,
            };
            let pages_enabled = match object.get("pages") {
                Lookup::Absent | Lookup::Value(Expression::ObjectExpression(_)) => true,
                Lookup::Value(Expression::BooleanLiteral(value)) => value.value,
                _ => return None,
            };
            Some((
                src_dir,
                pages,
                layouts,
                pages_enabled,
                static_render::admitted(object),
            ))
        })?;
    let src = match src_dir {
        Some(src_dir) => config.dir.join(src_dir),
        // Nuxt 4 defaults `srcDir` to `app/` unless `app.vue` stays at the root.
        None if ["app.vue", "App.vue"]
            .iter()
            .any(|name| config.dir.join(name).is_file()) =>
        {
            config.dir.clone()
        }
        None if config.dir.join("app").is_dir() => config.dir.join("app"),
        None => config.dir.clone(),
    };
    let relative = |path: PathBuf| {
        normalize(&path)
            .strip_prefix(root)
            .ok()
            .map(Path::to_path_buf)
    };
    Some(NuxtRenderDirs {
        pages: relative(src.join(pages.as_deref().unwrap_or("pages")))?,
        layouts: relative(src.join(layouts.as_deref().unwrap_or("layouts")))?,
        src: relative(src)?,
        pages_enabled,
        static_render_config: static_render_config && config.dir.as_path() == root,
    })
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::render_dirs;
    use std::path::{Path, PathBuf};
    use vize_croquis_cf::NuxtRenderDirs;

    fn dirs_for(config: &str, files: &[&str]) -> Option<NuxtRenderDirs> {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("nuxt.config.ts"), config).unwrap();
        for file in files {
            let path = project.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
        render_dirs(project.path())
    }

    fn dirs(
        src: &str,
        pages: &str,
        layouts: &str,
        pages_enabled: bool,
        static_render_config: bool,
    ) -> Option<NuxtRenderDirs> {
        Some(NuxtRenderDirs {
            src: PathBuf::from(src),
            pages: PathBuf::from(pages),
            layouts: PathBuf::from(layouts),
            pages_enabled,
            static_render_config,
        })
    }

    #[test]
    fn src_dir_and_dir_entries_resolve_like_nuxt() {
        assert_eq!(
            dirs_for(
                r#"export default defineNuxtConfig({ srcDir: "./src", dir: { layouts: "nuxt/layouts", pages: "nuxt/pages", app: "nuxt/app" }, $development: { devtools: { enabled: false } } })"#,
                &[],
            ),
            dirs("src", "src/nuxt/pages", "src/nuxt/layouts", true, false)
        );
        assert_eq!(
            dirs_for(
                "export default defineNuxtConfig({ pages: false })",
                &["app/pages/a.vue"]
            ),
            dirs("app", "app/pages", "app/layouts", false, true)
        );
        assert_eq!(
            dirs_for(
                "export default defineNuxtConfig({})",
                &["app.vue", "app/router.options.ts"]
            ),
            dirs("", "pages", "layouts", true, true)
        );
    }

    #[test]
    fn dynamic_aliased_or_overridden_dirs_stay_unknown() {
        for config in [
            "export default defineNuxtConfig({ dir: dirs })",
            "export default defineNuxtConfig({ dir: { ...base } })",
            "export default defineNuxtConfig({ srcDir: process.env.SRC })",
            r#"export default defineNuxtConfig({ dir: { pages: "~/routes" } })"#,
            r#"export default defineNuxtConfig({ $development: { dir: { pages: "dev" } } })"#,
            r#"export default defineNuxtConfig({ srcDir: "../outside" })"#,
            r#"export default defineNuxtConfig({ srcDir: "src", $env: { staging: { srcDir: "app" } } })"#,
            "export default defineNuxtConfig({ pages: process.env.PAGES })",
        ] {
            assert_eq!(dirs_for(config, &[]), None, "{config}");
        }
        assert_eq!(render_dirs(Path::new("/definitely/missing/root")), None);
    }
}
