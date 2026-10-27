//! A bounded declaration export graph. All reads belong to its capture owner.

use std::path::{Path, PathBuf};

use oxc_ast::ast::{
    Declaration, Function, ImportDeclarationSpecifier, Statement, TSType, TSTypeAliasDeclaration,
    TSTypeParameterDeclaration,
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use sha2::{Digest, Sha256};
use vize_carton::{FxHashMap, FxHashSet, String, cstr};

use super::manifest::Refusal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Parameter {
    pub(super) string_constraint: bool,
    pub(super) unconstrained: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    Alias,
    Function,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DeclarationIdentity {
    pub(super) path: PathBuf,
    pub(super) sha256: String,
    pub(super) start_byte: u32,
    pub(super) end_byte: u32,
    pub(super) kind: Kind,
    pub(super) parameters: Vec<Parameter>,
}

#[derive(Clone)]
enum Binding {
    Local(String),
    Remote { specifier: String, name: String },
    Declaration(DeclarationIdentity),
    Unsupported,
}

#[derive(Default)]
struct Module {
    locals: FxHashMap<String, Binding>,
    exports: FxHashMap<String, Binding>,
    stars: Vec<String>,
}

#[derive(Default, Debug)]
pub(super) struct Counts {
    pub(super) parsed_files: usize,
    pub(super) parsed_bytes: usize,
    pub(super) edges: usize,
    pub(super) aliases: usize,
}

pub(super) struct ExportGraph<R> {
    package_root: PathBuf,
    read: R,
    modules: FxHashMap<PathBuf, Module>,
    active: FxHashSet<(PathBuf, String, bool)>,
    resolved: FxHashMap<(PathBuf, String), PathBuf>,
    pub(super) counts: Counts,
}

impl<R> ExportGraph<R>
where
    R: FnMut(&Path) -> Result<Option<String>, Refusal>,
{
    pub(super) fn new(package_root: &Path, read: R) -> Self {
        Self {
            package_root: package_root.to_path_buf(),
            read,
            modules: FxHashMap::default(),
            active: FxHashSet::default(),
            resolved: FxHashMap::default(),
            counts: Counts::default(),
        }
    }

    pub(super) fn exported(
        &mut self,
        path: &Path,
        name: &str,
    ) -> Result<Option<DeclarationIdentity>, Refusal> {
        self.find(path, name, true)
    }

    fn find(
        &mut self,
        path: &Path,
        name: &str,
        exported: bool,
    ) -> Result<Option<DeclarationIdentity>, Refusal> {
        if self.active.len() >= 128 || self.counts.edges >= 1024 {
            return Err(Refusal::new(
                "export-graph-budget",
                "declaration export graph exceeds the bounded capture budget",
            ));
        }
        let key = (path.to_path_buf(), String::from(name), exported);
        if !self.active.insert(key.clone()) {
            return Err(Refusal::new(
                "cyclic-export-graph",
                "the requested declaration export graph is cyclic",
            ));
        }
        self.counts.edges += 1;
        let result = self.find_active(path, name, exported);
        self.active.remove(&key);
        result
    }

    fn find_active(
        &mut self,
        path: &Path,
        name: &str,
        exported: bool,
    ) -> Result<Option<DeclarationIdentity>, Refusal> {
        self.load(path)?;
        let module = self.modules.get(path).ok_or_else(|| {
            Refusal::new(
                "missing-captured-module",
                "the capture has no parsed module",
            )
        })?;
        let binding = if exported {
            module.exports.get(name)
        } else {
            module.locals.get(name)
        }
        .cloned();
        let stars = if exported && binding.is_none() {
            module.stars.clone()
        } else {
            Vec::new()
        };
        if let Some(binding) = binding {
            return match binding {
                Binding::Declaration(identity) => Ok(Some(identity)),
                Binding::Local(local) => {
                    self.counts.aliases += 1;
                    self.find(path, &local, false)
                }
                Binding::Remote { specifier, name } => {
                    self.counts.aliases += 1;
                    let target = self.relative_target(path, &specifier)?;
                    self.find(&target, &name, true)
                }
                Binding::Unsupported => Err(Refusal::new(
                    "unsupported-export-binding",
                    "the requested export has an unsupported declaration meaning",
                )),
            };
        }
        if !exported {
            return Err(Refusal::new(
                "unresolved-export-alias",
                cstr!(
                    "cannot resolve exported local '{name}' in {}",
                    path.display()
                ),
            ));
        }
        let mut found = None;
        for specifier in stars {
            let target = self.relative_target(path, &specifier)?;
            if let Some(identity) = self.find(&target, name, true)? {
                if found.as_ref().is_some_and(|previous| previous != &identity) {
                    return Err(Refusal::new(
                        "ambiguous-star-export",
                        "multiple declarations supply the requested star export",
                    ));
                }
                found = Some(identity);
            }
        }
        Ok(found)
    }

    fn load(&mut self, path: &Path) -> Result<(), Refusal> {
        if self.modules.contains_key(path) {
            return Ok(());
        }
        if self.modules.len() >= 64 {
            return Err(Refusal::new(
                "export-file-budget",
                "declaration export graph exceeds the bounded file budget",
            ));
        }
        let source = (self.read)(path)?.ok_or_else(|| {
            Refusal::new(
                "missing-declaration",
                cstr!("declaration {} is unavailable", path.display()),
            )
        })?;
        self.capture_module(path, &source)
    }

    fn capture_module(&mut self, path: &Path, source: &str) -> Result<(), Refusal> {
        if self.modules.contains_key(path) {
            return Ok(());
        }
        if self.modules.len() >= 64 {
            return Err(Refusal::new(
                "export-file-budget",
                "declaration export graph exceeds the bounded file budget",
            ));
        }
        self.counts.parsed_files += 1;
        self.counts.parsed_bytes += source.len();
        self.modules
            .insert(path.to_path_buf(), parse_module(path, source)?);
        Ok(())
    }

    fn relative_target(&mut self, origin: &Path, specifier: &str) -> Result<PathBuf, Refusal> {
        let key = (origin.to_path_buf(), String::from(specifier));
        if let Some(target) = self.resolved.get(&key) {
            return Ok(target.clone());
        }
        if !(specifier.starts_with("./") || specifier.starts_with("../"))
            || specifier.contains(['\\', '?', '#'])
        {
            return Err(Refusal::new(
                "unsupported-reexport-specifier",
                "only explicit package-local declaration re-exports are admitted",
            ));
        }
        let parent = origin.parent().ok_or_else(|| {
            Refusal::new(
                "missing-declaration-parent",
                "declaration has no parent directory",
            )
        })?;
        let base =
            super::super::super::tsconfig_paths::normalize_path_lexically(&parent.join(specifier));
        if !base.starts_with(&self.package_root) {
            return Err(Refusal::new(
                "external-reexport-target",
                "the declaration export graph escapes its captured package",
            ));
        }
        let text = base
            .to_str()
            .ok_or_else(|| Refusal::new("non-utf8-declaration", "declaration path is not UTF-8"))?;
        let (stem, suffixes): (&str, &[&str]) = if let Some(stem) = text.strip_suffix(".js") {
            (stem, &[".ts", ".tsx", ".d.ts"])
        } else if let Some(stem) = text.strip_suffix(".mjs") {
            (stem, &[".mts", ".d.mts"])
        } else if let Some(stem) = text.strip_suffix(".cjs") {
            (stem, &[".cts", ".d.cts"])
        } else {
            return Err(Refusal::new(
                "unsupported-reexport-extension",
                "the bounded resolver requires an explicit JavaScript declaration edge",
            ));
        };
        for suffix in suffixes {
            let candidate = PathBuf::from(cstr!("{stem}{suffix}"));
            if let Some(source) = (self.read)(&candidate)? {
                if !suffix.starts_with(".d.") {
                    return Err(Refusal::new(
                        "source-shadows-declaration",
                        "a source file wins before the expected declaration target",
                    ));
                }
                self.capture_module(&candidate, &source)?;
                self.resolved.insert(key, candidate.clone());
                return Ok(candidate);
            }
        }
        Err(Refusal::new(
            "missing-reexport-target",
            cstr!("no declaration target resolves from '{specifier}'"),
        ))
    }
}

fn parameters(parameters: Option<&TSTypeParameterDeclaration<'_>>) -> Vec<Parameter> {
    parameters.map_or_else(Vec::new, |parameters| {
        parameters
            .params
            .iter()
            .map(|parameter| Parameter {
                string_constraint: matches!(
                    parameter.constraint.as_ref(),
                    Some(TSType::TSStringKeyword(_))
                ),
                unconstrained: parameter.constraint.is_none(),
            })
            .collect()
    })
}

fn identity(
    path: &Path,
    digest: &str,
    span: Span,
    kind: Kind,
    params: Vec<Parameter>,
) -> DeclarationIdentity {
    DeclarationIdentity {
        path: path.to_path_buf(),
        sha256: digest.into(),
        start_byte: span.start,
        end_byte: span.end,
        kind,
        parameters: params,
    }
}

fn alias(path: &Path, digest: &str, declaration: &TSTypeAliasDeclaration<'_>) -> Binding {
    Binding::Declaration(identity(
        path,
        digest,
        declaration.span,
        Kind::Alias,
        parameters(declaration.type_parameters.as_deref()),
    ))
}

fn function(path: &Path, digest: &str, declaration: &Function<'_>) -> Binding {
    Binding::Declaration(identity(
        path,
        digest,
        declaration.span,
        Kind::Function,
        parameters(declaration.type_parameters.as_deref()),
    ))
}

fn insert(
    map: &mut FxHashMap<String, Binding>,
    name: &str,
    binding: Binding,
) -> Result<(), Refusal> {
    if map.insert(name.into(), binding).is_some() {
        return Err(Refusal::new(
            "duplicate-declaration-binding",
            cstr!("the declaration binding '{name}' is ambiguous"),
        ));
    }
    Ok(())
}

fn insert_local(map: &mut FxHashMap<String, Binding>, name: &str, binding: Binding) {
    if map.insert(name.into(), binding).is_some() {
        map.insert(name.into(), Binding::Unsupported);
    }
}

fn parse_module(path: &Path, source: &str) -> Result<Module, Refusal> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::d_ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Err(Refusal::new(
            "invalid-declaration-syntax",
            cstr!("cannot parse declaration {}", path.display()),
        ));
    }
    let digest = super::super::super::identity::encode_digest(Sha256::digest(source.as_bytes()));
    let mut module = Module::default();
    for statement in &parsed.program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                for specifier in import.specifiers.iter().flatten() {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => insert_local(
                            &mut module.locals,
                            specifier.local.name.as_str(),
                            Binding::Remote {
                                specifier: import.source.value.as_str().into(),
                                name: specifier.imported.name().as_str().into(),
                            },
                        ),
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            insert_local(
                                &mut module.locals,
                                specifier.local.name.as_str(),
                                Binding::Unsupported,
                            )
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            insert_local(
                                &mut module.locals,
                                specifier.local.name.as_str(),
                                Binding::Unsupported,
                            )
                        }
                    }
                }
            }
            Statement::TSTypeAliasDeclaration(declaration) => insert_local(
                &mut module.locals,
                declaration.id.name.as_str(),
                alias(path, &digest, declaration),
            ),
            Statement::FunctionDeclaration(declaration) => {
                if let Some(id) = &declaration.id {
                    insert_local(
                        &mut module.locals,
                        id.name.as_str(),
                        function(path, &digest, declaration),
                    );
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    match declaration {
                        Declaration::TSTypeAliasDeclaration(declaration) => {
                            insert_local(
                                &mut module.locals,
                                declaration.id.name.as_str(),
                                alias(path, &digest, declaration),
                            );
                            insert(
                                &mut module.exports,
                                declaration.id.name.as_str(),
                                Binding::Local(declaration.id.name.as_str().into()),
                            )?;
                        }
                        Declaration::FunctionDeclaration(declaration) => {
                            if let Some(id) = &declaration.id {
                                insert_local(
                                    &mut module.locals,
                                    id.name.as_str(),
                                    function(path, &digest, declaration),
                                );
                                insert(
                                    &mut module.exports,
                                    id.name.as_str(),
                                    Binding::Local(id.name.as_str().into()),
                                )?;
                            }
                        }
                        // Unknown requested declaration kinds must not become a
                        // known-absent proof merely because this parser omits them.
                        _ => {
                            return Err(Refusal::new(
                                "unsupported-export-declaration",
                                "an exported declaration kind is outside the bounded graph",
                            ));
                        }
                    }
                }
                for specifier in &export.specifiers {
                    let binding = export.source.as_ref().map_or_else(
                        || Binding::Local(specifier.local.name().as_str().into()),
                        |source| Binding::Remote {
                            specifier: source.value.as_str().into(),
                            name: specifier.local.name().as_str().into(),
                        },
                    );
                    insert(
                        &mut module.exports,
                        specifier.exported.name().as_str(),
                        binding,
                    )?;
                }
            }
            Statement::ExportAllDeclaration(export) => {
                if let Some(name) = &export.exported {
                    insert(
                        &mut module.exports,
                        name.name().as_str(),
                        Binding::Unsupported,
                    )?;
                } else {
                    module.stars.push(export.source.value.as_str().into());
                }
            }
            Statement::TSExportAssignment(_) => {
                return Err(Refusal::new(
                    "commonjs-declaration-export",
                    "CommonJS declaration assignment requires native semantic resolution",
                ));
            }
            _ => {}
        }
    }
    Ok(module)
}
