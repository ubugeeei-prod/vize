//! Names Nuxt auto-imports into every template of a project.
//!
//! Nuxt writes its import list to `<buildDir>/types/imports.d.ts` and declares
//! each name twice there: as a `declare global` constant for scripts and as a
//! `ComponentCustomProperties` member for templates. Only the names are read;
//! no type is resolved.

use std::{
    fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Statement, TSModuleDeclarationBody, TSSignature};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{FxHashMap, String};

use super::{config_file, optional_path, owns_an_application, read};

/// Nuxt project roots by directory, so a batch probes each directory once.
#[derive(Default)]
pub struct NuxtRoots {
    nearest: FxHashMap<PathBuf, Option<usize>>,
    roots: Vec<PathBuf>,
}

impl NuxtRoots {
    /// Index of the nearest Nuxt project at or above `directory`.
    pub fn nearest(&mut self, directory: &Path) -> Option<usize> {
        let mut visited = Vec::new();
        let mut current = Some(directory);
        let found = loop {
            let Some(dir) = current else {
                break None;
            };
            if let Some(found) = self.nearest.get(dir) {
                break *found;
            }
            visited.push(dir);
            if config_file(dir).is_some() {
                self.roots.push(dir.to_path_buf());
                break Some(self.roots.len() - 1);
            }
            if owns_an_application(dir) {
                break None;
            }
            current = dir.parent();
        };
        for dir in visited {
            self.nearest.insert(dir.to_path_buf(), found);
        }
        found
    }

    /// The project root [`Self::nearest`] numbered `index`.
    pub fn dir(&self, index: usize) -> Option<&Path> {
        self.roots.get(index).map(PathBuf::as_path)
    }

    /// The project roots, in the order [`Self::nearest`] numbered them.
    pub fn into_dirs(self) -> Vec<PathBuf> {
        self.roots
    }
}

/// The names Nuxt auto-imports into the templates of the project at `root`.
///
/// `Err` carries the declaration file the project has not generated yet.
pub fn template_globals(root: &Path) -> Result<Vec<String>, PathBuf> {
    let declarations = generated_dir(root).join("types").join("imports.d.ts");
    match fs::read_to_string(&declarations) {
        Ok(source) => Ok(declared_names(&source)),
        Err(_) => Err(declarations),
    }
}

/// Why `vue/no-undefined-refs` stays off for the project at `root` until Nuxt
/// writes `declarations`, worded once for every tool that skips it.
pub fn missing_declarations_notice(root: &Path, declarations: &Path) -> String {
    vize_l0::cstr!(
        "Skipping vue/no-undefined-refs in {}: Nuxt has not generated {} yet, so auto-imported \
         names cannot be told from undefined ones. Run `nuxi prepare` to generate it.",
        root.display(),
        declarations.display()
    )
}

/// `buildDir` when the config states it literally, else Nuxt's default.
fn generated_dir(root: &Path) -> PathBuf {
    let build_dir = config_file(root)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|source| read(&source, |config| optional_path(config.get("buildDir"))))
        .flatten();
    root.join(build_dir.as_deref().unwrap_or(".nuxt"))
}

/// Every `declare global` constant and `ComponentCustomProperties` member.
fn declared_names(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::d_ts()).parse();
    let mut names = Vec::new();
    if parsed.panicked {
        return names;
    }
    for statement in &parsed.program.body {
        match statement {
            Statement::TSGlobalDeclaration(global) => {
                for statement in &global.body.body {
                    let Statement::VariableDeclaration(declaration) = statement else {
                        continue;
                    };
                    for declarator in &declaration.declarations {
                        if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                            names.push(String::from(id.name.as_str()));
                        }
                    }
                }
            }
            Statement::TSModuleDeclaration(module) => {
                let Some(TSModuleDeclarationBody::TSModuleBlock(block)) = &module.body else {
                    continue;
                };
                for statement in &block.body {
                    let Statement::TSInterfaceDeclaration(interface) = statement else {
                        continue;
                    };
                    if interface.id.name != "ComponentCustomProperties" {
                        continue;
                    }
                    for signature in &interface.body.body {
                        if let TSSignature::TSPropertySignature(property) = signature
                            && let Some(name) = property.key.static_name()
                        {
                            names.push(String::from(name.as_ref()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    names
}

#[cfg(test)]
mod tests;
