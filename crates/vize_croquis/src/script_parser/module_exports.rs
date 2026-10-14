//! Optional export admission from the same original script parser result.

use oxc_ast::ast::{
    Declaration, ExportNamedDeclaration, ExportSpecifier, ImportDeclarationSpecifier,
    ModuleExportName, Statement,
};
use oxc_parser::ParserReturn;
use vize_carton::{CompactString, FxHashMap};

use super::parse::{parse_script_plain_with_exports, parse_script_setup_with_exports};
use super::{ScriptParseResult, ScriptParserOptions};

/// One authored named export of a script block, with whether it only names a
/// type. A star re-export of the module can be replaced by exactly these names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredModuleExport {
    pub name: CompactString,
    pub type_only: bool,
}

#[derive(Default)]
pub(crate) struct ModuleExportCapture {
    observed: bool,
    unknown: bool,
    names: Vec<AuthoredModuleExport>,
}

impl ModuleExportCapture {
    #[inline]
    pub(crate) fn observe_if(capture: Option<&mut Self>, parsed: &ParserReturn<'_>) {
        if let Some(capture) = capture {
            capture.observe(parsed);
        }
    }

    #[inline]
    pub(crate) fn observe_statement_if(
        capture: Option<&mut Self>,
        statement: &Statement<'_>,
        setup: bool,
    ) {
        if let Some(capture) = capture {
            capture.observe_statement(statement);
            if setup
                && matches!(
                    statement,
                    Statement::ExportAllDeclaration(_)
                        | Statement::ExportNamedDeclaration(_)
                        | Statement::ExportDefaultDeclaration(_)
                )
            {
                capture.refuse();
            }
        }
    }

    pub(crate) fn observe(&mut self, parsed: &ParserReturn<'_>) {
        self.observed = true;
        self.unknown |= parsed.panicked
            || !parsed.diagnostics.is_empty()
            || !parsed.module_record.star_export_entries.is_empty();
        if self.unknown {
            return;
        }
        let locals = top_level_kinds(&parsed.program.body);
        for statement in &parsed.program.body {
            let proved = match statement {
                Statement::ExportNamedDeclaration(export) => self.record_named(export, &locals),
                Statement::ExportAllDeclaration(_) => false,
                _ => true,
            };
            if !proved {
                self.unknown = true;
                return;
            }
        }
    }

    /// Record the names of one local named export. Returns `false` when a name
    /// or its type/value meaning cannot be proved from this module alone.
    fn record_named(
        &mut self,
        export: &ExportNamedDeclaration<'_>,
        locals: &FxHashMap<&str, LocalKind>,
    ) -> bool {
        if export.source.is_some() {
            return false;
        }
        let whole_type = export.export_kind.is_type();
        if let Some(declaration) = &export.declaration {
            if matches!(
                declaration,
                Declaration::TSModuleDeclaration(_)
                    | Declaration::TSGlobalDeclaration(_)
                    | Declaration::TSImportEqualsDeclaration(_)
            ) {
                return false;
            }
            let type_only = whole_type || is_type_declaration(declaration);
            let Some(names) = declared_names(declaration) else {
                return false;
            };
            for name in names {
                self.push(name, type_only);
            }
            return true;
        }
        export
            .specifiers
            .iter()
            .all(|specifier| self.record_specifier(specifier, whole_type, locals))
    }

    fn record_specifier(
        &mut self,
        specifier: &ExportSpecifier<'_>,
        whole_type: bool,
        locals: &FxHashMap<&str, LocalKind>,
    ) -> bool {
        let exported = match &specifier.exported {
            ModuleExportName::IdentifierName(name) => name.name.as_str(),
            ModuleExportName::IdentifierReference(name) => name.name.as_str(),
            ModuleExportName::StringLiteral(name) if name.value == "default" => "default",
            ModuleExportName::StringLiteral(_) => return false,
        };
        if exported == "default" {
            // A default re-export of an imported binding is indirect.
            return matches!(
                locals.get(specifier.local.name().as_str()),
                Some(LocalKind::Value)
            );
        }
        let type_only = if whole_type || specifier.export_kind.is_type() {
            true
        } else {
            match locals.get(specifier.local.name().as_str()) {
                Some(LocalKind::Type) => true,
                Some(LocalKind::Value) => false,
                Some(LocalKind::Ambiguous) | None => return false,
            }
        };
        self.push(exported, type_only);
        true
    }

    fn push(&mut self, name: &str, type_only: bool) {
        if !self.names.iter().any(|known| known.name == name) {
            self.names.push(AuthoredModuleExport {
                name: CompactString::new(name),
                type_only,
            });
        }
    }

    pub(crate) fn observe_statement(&mut self, statement: &Statement<'_>) {
        self.unknown |= matches!(
            statement,
            Statement::TSExportAssignment(_)
                | Statement::TSNamespaceExportDeclaration(_)
                | Statement::TSModuleDeclaration(_)
                | Statement::TSImportEqualsDeclaration(_)
        );
    }

    pub(crate) fn refuse(&mut self) {
        self.unknown = true;
    }

    pub(crate) fn known_empty(&self) -> bool {
        self.observed && !self.unknown && self.names.is_empty()
    }

    /// Every authored named export, or `None` when the set is not provable.
    pub(crate) fn known_exports(&self) -> Option<Vec<AuthoredModuleExport>> {
        (self.observed && !self.unknown).then(|| self.names.clone())
    }
}

pub(crate) fn parse_script_setup_for_unused<const BUILTIN_TYPES: bool>(
    source: &str,
    generic: Option<&str>,
    jsx: bool,
    unused: bool,
    skip_diagnostics: bool,
    occurrences: bool,
) -> ScriptParseResult {
    parse_script_setup_with_exports::<BUILTIN_TYPES>(
        source,
        generic,
        jsx,
        unused,
        skip_diagnostics,
        occurrences,
        None,
    )
}

pub(crate) fn parse_script_plain(
    source: &str,
    options: ScriptParserOptions,
    jsx: bool,
    skip_diagnostics: bool,
    occurrences: bool,
) -> ScriptParseResult {
    parse_script_plain_with_exports(source, options, jsx, skip_diagnostics, occurrences, None)
}

/// Whether a top-level binding means a type, a value, or cannot be decided
/// without resolving another module.
#[derive(Clone, Copy)]
enum LocalKind {
    Type,
    Value,
    Ambiguous,
}

fn is_type_declaration(declaration: &Declaration<'_>) -> bool {
    matches!(
        declaration,
        Declaration::TSInterfaceDeclaration(_) | Declaration::TSTypeAliasDeclaration(_)
    )
}

/// Classify every top-level binding. A value import's meaning depends on the
/// imported module, so it stays ambiguous and blocks a precise re-export.
fn top_level_kinds<'a>(body: &'a [Statement<'a>]) -> FxHashMap<&'a str, LocalKind> {
    let mut kinds: FxHashMap<&'a str, LocalKind> = FxHashMap::default();
    let mut insert = |name: &'a str, kind: LocalKind| {
        let merged = match (kinds.get(name).copied(), kind) {
            (None, kind) => kind,
            (Some(LocalKind::Type), LocalKind::Type) => LocalKind::Type,
            (Some(LocalKind::Type | LocalKind::Value), LocalKind::Type | LocalKind::Value) => {
                LocalKind::Value
            }
            _ => LocalKind::Ambiguous,
        };
        kinds.insert(name, merged);
    };
    for statement in body {
        let declaration = match statement {
            Statement::ImportDeclaration(import) => {
                for specifier in import.specifiers.iter().flatten() {
                    let (local, type_only) = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                            specifier.local.name.as_str(),
                            import.import_kind.is_type() || specifier.import_kind.is_type(),
                        ),
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            (specifier.local.name.as_str(), import.import_kind.is_type())
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            (specifier.local.name.as_str(), import.import_kind.is_type())
                        }
                    };
                    let kind = if type_only {
                        LocalKind::Type
                    } else {
                        LocalKind::Ambiguous
                    };
                    insert(local, kind);
                }
                continue;
            }
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(declaration) => declaration,
                None => continue,
            },
            _ => match statement.as_declaration() {
                Some(declaration) => declaration,
                None => continue,
            },
        };
        let kind = if is_type_declaration(declaration) {
            LocalKind::Type
        } else {
            LocalKind::Value
        };
        match declared_names(declaration) {
            Some(names) => names.into_iter().for_each(|name| insert(name, kind)),
            None => continue,
        }
    }
    kinds
}

/// The plain identifiers a declaration binds, or `None` for a destructuring or
/// anonymous declaration whose exported names are not stated literally.
fn declared_names<'a>(declaration: &'a Declaration<'a>) -> Option<Vec<&'a str>> {
    if let Declaration::VariableDeclaration(variables) = declaration {
        return variables
            .declarations
            .iter()
            .map(|declarator| match &declarator.id {
                oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
                _ => None,
            })
            .collect();
    }
    declaration.id().map(|id| vec![id.name.as_str()])
}
