//! Static facts from the `nuxt.config.*` that owns a project directory.
//!
//! Only literal values count. Anything dynamic reads as unknown so callers can
//! keep their conservative default. The CLI and the language server share
//! this reader so both see the same project.

pub mod auto_imports;

use std::{
    fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::String;

/// The file names Nuxt loads its config from, in its lookup order.
pub const NUXT_CONFIG_NAMES: [&str; 6] = [
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

/// The config file that owns a directory.
pub struct NuxtConfig {
    /// The project root: the directory that holds the config file.
    pub dir: PathBuf,
    /// `None` when the file exists but cannot be read.
    pub source: Option<String>,
}

/// The nearest config at or above `root`.
pub fn find(root: &Path) -> Option<NuxtConfig> {
    let mut directory = Some(root);
    while let Some(dir) = directory {
        if let Some(path) = config_file(dir) {
            return Some(NuxtConfig {
                dir: dir.to_path_buf(),
                source: fs::read_to_string(path).ok().map(String::from),
            });
        }
        if owns_an_application(dir) {
            return None;
        }
        directory = dir.parent();
    }
    None
}

pub(crate) fn config_file(dir: &Path) -> Option<PathBuf> {
    NUXT_CONFIG_NAMES
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.exists())
}

/// A command launched in src/ still belongs to its application. Stop at a
/// nested package or Vite project instead of borrowing an outer app's
/// configuration for a separately owned application.
pub(crate) fn owns_an_application(dir: &Path) -> bool {
    dir.join("package.json").is_file()
        || VITE_CONFIG_NAMES
            .iter()
            .any(|name| dir.join(name).is_file())
}

/// What a config key holds when only literal values count.
pub enum Lookup<'a> {
    Absent,
    Value(&'a Expression<'a>),
    Unknown,
}

/// A config object literal. The root object treats `$env` keys as overrides.
pub struct ConfigObject<'a> {
    object: &'a ObjectExpression<'a>,
    root: bool,
    closed_program: bool,
}

impl<'a> ConfigObject<'a> {
    /// The object literal `expression` holds, if it is one.
    pub fn nested(expression: &'a Expression<'a>) -> Option<Self> {
        match unwrap_expression(expression) {
            Expression::ObjectExpression(object) => Some(Self {
                object,
                root: false,
                closed_program: false,
            }),
            _ => None,
        }
    }

    /// The root object, when the config file holds nothing but this literal.
    pub fn closed_root(&self) -> Option<&'a ObjectExpression<'a>> {
        (self.root && self.closed_program).then_some(self.object)
    }

    /// The literal `key` holds, with environment overrides taken into account.
    pub fn get(&self, key: &str) -> Lookup<'a> {
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
                // `$env` maps custom environment names to override blocks.
                Some("$env") if self.root => {
                    let Some(environments) = Self::nested(&property.value) else {
                        return Lookup::Unknown;
                    };
                    for environment in &environments.object.properties {
                        let ObjectPropertyKind::ObjectProperty(environment) = environment else {
                            return Lookup::Unknown;
                        };
                        if !collect_override(&environment.value, key, &mut overrides) {
                            return Lookup::Unknown;
                        }
                    }
                }
                // Environment blocks such as `$production` can replace root settings.
                Some(name) if self.root && name.starts_with('$') => {
                    if !collect_override(&property.value, key, &mut overrides) {
                        return Lookup::Unknown;
                    }
                }
                Some(name) if name == key => {
                    found = Lookup::Value(unwrap_expression(&property.value))
                }
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

/// Record an environment block's literal `key`; `false` when the block is not static.
fn collect_override<'a>(
    block: &'a Expression<'a>,
    key: &str,
    overrides: &mut Vec<&'a Expression<'a>>,
) -> bool {
    let Some(block) = ConfigObject::nested(block) else {
        return false;
    };
    match block.get(key) {
        Lookup::Absent => true,
        Lookup::Value(value) => {
            overrides.push(value);
            true
        }
        Lookup::Unknown => false,
    }
}

/// Parse `source` and read its default-exported config object.
pub fn read<T>(source: &str, read: impl FnOnce(&ConfigObject<'_>) -> Option<T>) -> Option<T> {
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
    let expression = unwrap_expression(export.declaration.as_expression()?);
    let closed_program = parsed.program.body.len() == 1
        && match expression {
            Expression::ObjectExpression(_) => true,
            Expression::CallExpression(call) => {
                matches!(&call.callee, Expression::Identifier(name) if name.name == "defineNuxtConfig")
                    && !call.optional
                    && call.arguments.len() == 1
                    && call.type_arguments.is_none()
            }
            _ => false,
        };
    let expression = match expression {
        Expression::CallExpression(call) if matches!(&call.callee, Expression::Identifier(name) if name.name == "defineNuxtConfig") =>
        {
            // A locally defined function with this name is not the Nuxt
            // configuration identity. Keep the result unknown in that case.
            if shadows_nuxt_config(&parsed.program) {
                return None;
            }
            unwrap_expression(call.arguments.first()?.as_expression()?)
        }
        expression => expression,
    };
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };
    read(&ConfigObject {
        object,
        root: true,
        closed_program,
    })
}

/// A literal relative or absolute path; `Some(None)` when the key is absent.
pub fn optional_path(lookup: Lookup<'_>) -> Option<Option<String>> {
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

/// The name of a non-computed or string-literal property key.
pub fn static_key<'a>(key: &'a PropertyKey<'a>, computed: bool) -> Option<&'a str> {
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

/// `expression` without parentheses and type assertions around it.
pub fn unwrap_expression<'e, 'a>(mut expression: &'e Expression<'a>) -> &'e Expression<'a> {
    loop {
        expression = match expression {
            Expression::ParenthesizedExpression(inner) => &inner.expression,
            Expression::TSAsExpression(inner) => &inner.expression,
            Expression::TSSatisfiesExpression(inner) => &inner.expression,
            Expression::TSNonNullExpression(inner) => &inner.expression,
            _ => return expression,
        };
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
