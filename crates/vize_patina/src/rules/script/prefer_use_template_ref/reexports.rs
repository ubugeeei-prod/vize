//! Whether a callee imported from a non-Vue module is Vue's `ref` /
//! `shallowRef`, decided by following the module's re-exports.
//!
//! Projects re-export Vue's factories through their own modules
//! (`export { ref } from 'vue'`, `import { ref } from 'vue'; export { ref }`,
//! `export * from 'vue'`). The walk resolves each specifier the way imported
//! macro types resolve (relative paths, tsconfig `paths`, package entries)
//! until it reaches a module that provides Vue's factories. A module that
//! cannot be resolved or read, defines the name itself, or exports it through
//! two different stars is not Vue's: no report rather than a guess.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use oxc_ecmascript::BoundNames;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::declarations::vue_factory;
use crate::linter::run_type_sources::{ModuleReader, with_module_reader};
use crate::rules::script::ImportingFile;
use crate::rules::script::composition_api::{
    is_composition_api_module, reexports_vue_ref_factories,
};

/// Modules one decision may read. Barrel files fan out through `export *`, so
/// the walk is bounded by work rather than by depth alone.
const MAX_MODULES: usize = 32;

/// What `export` resolves to in Vue's ref-factory terms.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Vue(&'static str),
    /// The module was read and does not export the name.
    Absent,
    /// Anything else: a local definition, an unreadable module, an ambiguity.
    Other,
}

/// The Vue factory `export` of `specifier`, imported by `file`, re-exports.
pub(super) fn imported_factory(
    file: ImportingFile<'_>,
    specifier: &str,
    export: &str,
) -> Option<&'static str> {
    with_module_reader(file.sources, file.filename, |importer, modules| {
        let mut walk = Walk {
            modules,
            budget: MAX_MODULES,
        };
        match walk.origin(importer, specifier, export) {
            Origin::Vue(factory) => Some(factory),
            Origin::Absent | Origin::Other => None,
        }
    })
}

struct Walk<'s> {
    modules: ModuleReader<'s>,
    budget: usize,
}

impl Walk<'_> {
    fn origin(&mut self, importer: &Path, specifier: &str, export: &str) -> Origin {
        if is_composition_api_module(specifier) || reexports_vue_ref_factories(specifier) {
            return vue_factory(export).map_or(Origin::Other, Origin::Vue);
        }
        if self.budget == 0 {
            return Origin::Other;
        }
        self.budget -= 1;
        let Some(path) = self.modules.resolve(importer, specifier) else {
            return Origin::Other;
        };
        if path.extension().is_some_and(|extension| extension == "vue") {
            return Origin::Other;
        }
        let Ok(source_type) = SourceType::from_path(&path) else {
            return Origin::Other;
        };
        let modules = self.modules;
        modules
            .with_source(&path, |source| {
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, source, source_type).parse();
                if parsed.panicked || !parsed.diagnostics.is_empty() {
                    return Origin::Other;
                }
                self.module_export(&path, &parsed.program, export)
            })
            .unwrap_or(Origin::Other)
    }

    /// What `program`, read from `path`, exports as `export`.
    fn module_export(&mut self, path: &Path, program: &Program<'_>, export: &str) -> Origin {
        let mut stars = Vec::new();
        for statement in &program.body {
            match statement {
                Statement::ExportNamedDeclaration(declaration)
                    if !declaration.export_kind.is_type() =>
                {
                    if let Some(declared) = &declaration.declaration {
                        let mut defines = false;
                        declared.bound_names(&mut |binding| defines |= binding.name == export);
                        if defines {
                            return Origin::Other;
                        }
                        continue;
                    }
                    let Some(specifier) = declaration.specifiers.iter().find(|specifier| {
                        !specifier.export_kind.is_type() && specifier.exported.name() == export
                    }) else {
                        continue;
                    };
                    let local = specifier.local.name();
                    return match &declaration.source {
                        Some(source) => self.origin(path, source.value.as_str(), local.as_str()),
                        None => self.imported_binding(path, program, local.as_str()),
                    };
                }
                Statement::ExportAllDeclaration(declaration)
                    if declaration.exported.is_none() && !declaration.export_kind.is_type() =>
                {
                    stars.push(declaration.source.value.as_str());
                }
                // `export =` replaces the module's named exports.
                Statement::TSExportAssignment(_) => return Origin::Other,
                _ => {}
            }
        }
        // An explicit export wins over every star; two stars providing the
        // name are ambiguous unless they reach the same Vue factory.
        let mut found = Origin::Absent;
        for star in stars {
            match self.origin(path, star, export) {
                Origin::Absent => {}
                Origin::Vue(factory) if matches!(found, Origin::Absent) => {
                    found = Origin::Vue(factory);
                }
                origin @ Origin::Vue(_) if origin == found => {}
                Origin::Vue(_) | Origin::Other => return Origin::Other,
            }
        }
        found
    }

    /// `export { local }` without a source: Vue's factory only when `local` is
    /// itself imported from a module that provides it.
    fn imported_binding(&mut self, path: &Path, program: &Program<'_>, local: &str) -> Origin {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                if specifier.local().name != local {
                    continue;
                }
                return match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type() =>
                    {
                        self.origin(
                            path,
                            import.source.value.as_str(),
                            specifier.imported.name().as_str(),
                        )
                    }
                    _ => Origin::Other,
                };
            }
        }
        Origin::Other
    }
}
