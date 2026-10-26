//! Static facts from the `nuxt.config.*` that owns a project directory.
//!
//! Only literal values count. Anything dynamic reads as unknown so callers can
//! keep their conservative default.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_croquis_cf::NuxtRenderDirs;
use vize_l0::String;

const CONFIG_NAMES: [&str; 6] = [
    "nuxt.config.ts",
    "nuxt.config.js",
    "nuxt.config.mts",
    "nuxt.config.mjs",
    "nuxt.config.cts",
    "nuxt.config.cjs",
];
const VITE_CONFIG_NAMES: [&str; 6] = [
    "vite.config.ts",
    "vite.config.js",
    "vite.config.mts",
    "vite.config.mjs",
    "vite.config.cts",
    "vite.config.cjs",
];

pub(super) struct NuxtConfig {
    pub(super) dir: PathBuf,
    /// `None` when the file exists but cannot be read.
    pub(super) source: Option<String>,
}

/// The nearest config at or above `root`.
pub(super) fn find(root: &Path) -> Option<NuxtConfig> {
    let mut directory = Some(root);
    while let Some(dir) = directory {
        for name in CONFIG_NAMES {
            let path = dir.join(name);
            if path.exists() {
                return Some(NuxtConfig {
                    dir: dir.to_path_buf(),
                    source: fs::read_to_string(path).ok().map(String::from),
                });
            }
        }
        // A command launched in src/ still belongs to its application. Stop at
        // a nested package or Vite project instead of borrowing an outer app's
        // configuration for a separately owned application.
        if dir.join("package.json").is_file()
            || VITE_CONFIG_NAMES
                .iter()
                .any(|name| dir.join(name).is_file())
        {
            return None;
        }
        directory = dir.parent();
    }
    None
}

pub(super) enum Lookup<'a> {
    Absent,
    Value(&'a Expression<'a>),
    Unknown,
}

/// A config object literal. The root object treats `$env` keys as overrides.
pub(super) struct ConfigObject<'a> {
    object: &'a ObjectExpression<'a>,
    root: bool,
}

impl<'a> ConfigObject<'a> {
    pub(super) fn nested(expression: &'a Expression<'a>) -> Option<Self> {
        match unwrap(expression) {
            Expression::ObjectExpression(object) => Some(Self {
                object,
                root: false,
            }),
            _ => None,
        }
    }

    pub(super) fn get(&self, key: &str) -> Lookup<'a> {
        let mut found = Lookup::Absent;
        let mut overrides = Vec::new();
        for property in &self.object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                // Later literal properties may override a spread's unknown key.
                found = Lookup::Unknown;
                continue;
            };
            match static_key(&property.key, property.computed) {
                None => found = Lookup::Unknown,
                Some(name) if self.root && name.starts_with('$') => {
                    // Environment blocks can replace root settings.
                    let Some(block) = Self::nested(&property.value) else {
                        return Lookup::Unknown;
                    };
                    match block.get(key) {
                        Lookup::Absent => {}
                        Lookup::Value(value) => overrides.push(value),
                        Lookup::Unknown => return Lookup::Unknown,
                    }
                }
                Some(name) if name == key => found = Lookup::Value(unwrap(&property.value)),
                Some(_) => {}
            }
        }
        match found {
            // An override that repeats the root literal cannot change it.
            Lookup::Value(value) if overrides.iter().all(|other| same_literal(value, other)) => {
                found
            }
            Lookup::Absent if overrides.is_empty() => found,
            Lookup::Unknown => found,
            _ => Lookup::Unknown,
        }
    }
}

/// Parse `source` and read its default-exported config object.
pub(super) fn read<T>(
    source: &str,
    read: impl FnOnce(&ConfigObject<'_>) -> Option<T>,
) -> Option<T> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let export = parsed
        .program
        .body
        .iter()
        .find_map(|statement| match statement {
            Statement::ExportDefaultDeclaration(export) => Some(export),
            _ => None,
        })?;
    let expression = unwrap(export.declaration.as_expression()?);
    let expression = match expression {
        Expression::CallExpression(call) if matches!(&call.callee, Expression::Identifier(name) if name.name == "defineNuxtConfig") =>
        {
            // A locally defined function with this name is not the Nuxt
            // configuration identity. Keep the result unknown in that case.
            if shadows_nuxt_config(&parsed.program) {
                return None;
            }
            unwrap(call.arguments.first()?.as_expression()?)
        }
        expression => expression,
    };
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };
    read(&ConfigObject { object, root: true })
}

/// `srcDir`, `dir.pages` and `dir.layouts`, relative to `root` like doctor sources.
pub(super) fn render_dirs(root: &Path) -> Option<NuxtRenderDirs> {
    let config = find(root)?;
    let (src_dir, pages, layouts, builtin_app) = read(config.source.as_deref()?, |object| {
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
        // Pages enable Nuxt's built-in `<NuxtLayout><NuxtPage /></NuxtLayout>` app.
        let builtin_app = match object.get("pages") {
            Lookup::Absent => true,
            Lookup::Value(Expression::BooleanLiteral(value)) => value.value,
            Lookup::Value(Expression::ObjectExpression(_)) => true,
            _ => false,
        };
        Some((src_dir, pages, layouts, builtin_app))
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
        builtin_app,
    })
}

/// A literal relative or absolute path; `Some(None)` when the key is absent.
fn optional_path(lookup: Lookup<'_>) -> Option<Option<String>> {
    let value = match lookup {
        Lookup::Absent => return Some(None),
        Lookup::Value(value) => value,
        Lookup::Unknown => return None,
    };
    let path = match value {
        Expression::StringLiteral(literal) => literal.value.as_str(),
        Expression::TemplateLiteral(template) => template.single_quasi()?.as_str(),
        _ => return None,
    };
    // Nuxt resolves aliases such as `~/` here; leave those unknown.
    (!path.starts_with(['~', '@', '#'])).then(|| Some(path.into()))
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

fn static_key<'a>(key: &'a PropertyKey<'a>, computed: bool) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(name) if !computed => Some(name.name.as_str()),
        PropertyKey::StringLiteral(name) => Some(name.value.as_str()),
        _ => None,
    }
}

fn same_literal(left: &Expression<'_>, right: &Expression<'_>) -> bool {
    match (left, right) {
        (Expression::BooleanLiteral(left), Expression::BooleanLiteral(right)) => {
            left.value == right.value
        }
        (Expression::StringLiteral(left), Expression::StringLiteral(right)) => {
            left.value == right.value
        }
        _ => false,
    }
}

pub(super) fn unwrap<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => unwrap(&inner.expression),
        Expression::TSAsExpression(inner) => unwrap(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrap(&inner.expression),
        Expression::TSNonNullExpression(inner) => unwrap(&inner.expression),
        expression => expression,
    }
}

fn shadows_nuxt_config(program: &oxc_ast::ast::Program<'_>) -> bool {
    use oxc_ast::ast::{BindingIdentifier, ImportDeclaration, ImportDeclarationSpecifier};
    use oxc_ast_visit::Visit;

    #[derive(Default)]
    struct Bindings(bool);
    impl<'a> Visit<'a> for Bindings {
        fn visit_binding_identifier(&mut self, binding: &BindingIdentifier<'a>) {
            self.0 |= binding.name == "defineNuxtConfig";
        }

        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            for specifier in import.specifiers.iter().flatten() {
                if matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if import.source.value == "nuxt/config"
                        && specifier.local.name == "defineNuxtConfig"
                        && specifier.imported.name() == "defineNuxtConfig")
                {
                    continue;
                }
                self.visit_import_declaration_specifier(specifier);
            }
        }
    }
    let mut bindings = Bindings::default();
    bindings.visit_program(program);
    bindings.0
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

    fn dirs(src: &str, pages: &str, layouts: &str, builtin_app: bool) -> Option<NuxtRenderDirs> {
        Some(NuxtRenderDirs {
            src: PathBuf::from(src),
            pages: PathBuf::from(pages),
            layouts: PathBuf::from(layouts),
            builtin_app,
        })
    }

    #[test]
    fn src_dir_and_dir_entries_resolve_like_nuxt() {
        assert_eq!(
            dirs_for(
                r#"export default defineNuxtConfig({ srcDir: "./src", dir: { layouts: "nuxt/layouts", pages: "nuxt/pages", app: "nuxt/app" }, $development: { devtools: { enabled: false } } })"#,
                &[],
            ),
            dirs("src", "src/nuxt/pages", "src/nuxt/layouts", true)
        );
        assert_eq!(
            dirs_for(
                "export default defineNuxtConfig({ pages: false })",
                &["app/pages/a.vue"]
            ),
            dirs("app", "app/pages", "app/layouts", false)
        );
        assert_eq!(
            dirs_for(
                "export default defineNuxtConfig({})",
                &["app.vue", "app/router.options.ts"]
            ),
            dirs("", "pages", "layouts", true)
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
        ] {
            assert_eq!(dirs_for(config, &[]), None, "{config}");
        }
        assert_eq!(render_dirs(Path::new("/definitely/missing/root")), None);
    }
}
