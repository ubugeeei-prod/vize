//! Classify the imports of one script block and report the mismatches.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    ImportDeclaration, ImportDeclarationSpecifier, Program, Statement, TSModuleDeclarationBody,
};
use oxc_span::Span;
use vize_l0::{CompactString, String, cstr};

use super::rewrite::{Placement, inline_type_keyword, rewrite, type_placement};
use super::uses::ComponentUses;
use super::{ConsistentTypeImportsOptions, FixStyle, META, TypeImportPreference};
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use crate::linter::sfc_scripts::BlockSemantic;
use crate::rules::script::{ScriptLintResult, SfcScriptContext, UnboundRead};

const TYPE_HELP: &str =
    "A type-only import is erased at build time, so the module is not loaded for it.";
const VALUE_HELP: &str =
    "A type-only import does not exist at runtime. Remove `type` to import the value.";

/// The script block a declaration is reported and rewritten in.
#[derive(Clone, Copy)]
struct Site<'p, 'a> {
    program: &'p Program<'a>,
    source: &'p str,
    /// The block's position in the linted file.
    offset: usize,
}

impl Site<'_, '_> {
    fn at(self, position: u32) -> u32 {
        self.offset as u32 + position
    }

    fn warn(self, span: Span, message: impl Into<CompactString>) -> LintDiagnostic {
        LintDiagnostic::warn(META.name, message, self.at(span.start), self.at(span.end))
    }

    fn replacement(self, span: Span, title: &'static str, text: String) -> Fix {
        Fix::new(
            title,
            TextEdit::new(self.at(span.start), self.at(span.end), text),
        )
    }

    fn rewrite(
        self,
        declaration: &ImportDeclaration<'_>,
        placements: &[Placement],
    ) -> Option<String> {
        rewrite(self.program, self.source, declaration, placements)
    }
}

pub(super) fn check<'a>(
    options: &ConsistentTypeImportsOptions,
    program: &'a Program<'a>,
    source: &str,
    offset: usize,
    sfc: SfcScriptContext<'a>,
    result: &mut ScriptLintResult,
) {
    let site = Site {
        program,
        source,
        offset,
    };
    let imports = imports(program);
    let annotations = options.disallow_type_annotations && source.contains("import");
    if imports.is_empty() && !annotations {
        return;
    }
    let prefers_types = options.prefer == TypeImportPreference::TypeImports;
    let built = BlockSemantic::of(sfc.semantic_of(program), program, true);
    let Some(mut uses) = ComponentUses::new(built, sfc, options.emit_decorator_metadata) else {
        return;
    };
    if annotations {
        for node in uses.semantic().nodes().iter() {
            if let AstKind::TSImportType(annotation) = node.kind() {
                result.add_diagnostic(
                    site.warn(
                        annotation.span,
                        "`import()` type annotations are forbidden.",
                    )
                    .with_help("Import the type with `import type` and refer to it by name."),
                );
            }
        }
    }
    if !prefers_types {
        for (declaration, shared) in imports {
            avoid_type_import(site, declaration, shared, &uses, result);
        }
        return;
    }
    for (declaration, shared) in imports {
        prefer_type_import(
            options.fix_style,
            site,
            declaration,
            shared,
            &mut uses,
            result,
        );
    }
}

/// Each import declaration, and whether it is in the module scope the other
/// blocks of the component share. An ambient `declare module "name" { ... }`
/// block imports into a scope of its own.
fn imports<'p, 'a>(program: &'p Program<'a>) -> Vec<(&'p ImportDeclaration<'a>, bool)> {
    let mut found = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(declaration) => found.push((&**declaration, true)),
            Statement::TSModuleDeclaration(module) => {
                let Some(TSModuleDeclarationBody::TSModuleBlock(block)) = &module.body else {
                    continue;
                };
                for statement in &block.body {
                    if let Statement::ImportDeclaration(declaration) = statement {
                        found.push((&**declaration, false));
                    }
                }
            }
            _ => {}
        }
    }
    found
}

/// `prefer: "type-imports"`: one declaration against the component's uses.
fn prefer_type_import(
    style: FixStyle,
    site: Site<'_, '_>,
    declaration: &ImportDeclaration<'_>,
    shared: bool,
    uses: &mut ComponentUses<'_>,
    result: &mut ScriptLintResult,
) {
    let Some(specifiers) = declaration.specifiers.as_deref() else {
        return;
    };
    // `import source x` and `import defer * as x` have no type-only form.
    if specifiers.is_empty() || declaration.phase.is_some() {
        return;
    }
    let declared_type = declaration.import_kind.is_type();
    let mut placements = Vec::with_capacity(specifiers.len());
    // Value specifiers whose every use is a type.
    let mut type_only: Vec<&str> = Vec::new();
    // Type specifiers the template or a style `v-bind()` evaluates.
    let mut read_as_value: Vec<(&str, Span, UnboundRead)> = Vec::new();
    let mut keeps_runtime_specifier = false;
    let mut changes_metadata = false;
    for specifier in specifiers {
        let local = specifier.local();
        let name = local.name.as_str();
        let inline_type = matches!(
            specifier,
            ImportDeclarationSpecifier::ImportSpecifier(named) if named.import_kind.is_type()
        );
        if declared_type || inline_type {
            if let Some(read) = uses.unbound_read(name).filter(|_| shared) {
                changes_metadata |= uses.metadata_reads(local, shared);
                read_as_value.push((name, local.span, read));
                keeps_runtime_specifier = true;
                placements.push(Placement::Value);
            } else if declared_type {
                placements.push(type_placement(specifier, style));
            } else {
                placements.push(Placement::InlineType);
            }
            continue;
        }
        let Some(symbol) = local.symbol_id.get() else {
            return;
        };
        let found = uses.of(symbol, name, shared);
        if !found.value && found.ty {
            type_only.push(name);
            placements.push(type_placement(specifier, style));
        } else {
            keeps_runtime_specifier = true;
            placements.push(Placement::Value);
        }
    }
    if type_only.is_empty() && read_as_value.is_empty() {
        return;
    }
    let fixed = if changes_metadata {
        None
    } else {
        site.rewrite(declaration, &placements)
    };
    // Import attributes cannot follow `import type`, so a declaration that
    // would become one whole is left as written.
    if !type_only.is_empty() && (keeps_runtime_specifier || declaration.with_clause.is_none()) {
        let message = if keeps_runtime_specifier {
            some_are_types(&type_only)
        } else {
            String::from(
                "All imports in the declaration are only used as types. Use `import type`.",
            )
        };
        let title = match style {
            FixStyle::SeparateTypeImports => "Use `import type`",
            FixStyle::InlineTypeImports => "Use a type-only import",
        };
        let diagnostic = site.warn(declaration.span, message).with_help(TYPE_HELP);
        result.add_diagnostic(match &fixed {
            Some(text) => {
                diagnostic.with_fix(site.replacement(declaration.span, title, text.clone()))
            }
            None => diagnostic,
        });
    }
    for (name, span, read) in read_as_value {
        let reader = match read {
            UnboundRead::Template => "the template",
            UnboundRead::StyleBind => "a `<style>` `v-bind()`",
        };
        let diagnostic = site
            .warn(
                span,
                cstr!("\"{name}\" is imported as a type, but {reader} reads it as a value."),
            )
            .with_help(VALUE_HELP);
        result.add_diagnostic(match &fixed {
            Some(text) => diagnostic.with_fix(site.replacement(
                declaration.span,
                "Import the value",
                text.clone(),
            )),
            None => diagnostic,
        });
    }
}

/// `Import "A" is only used as a type.` / `Imports "A", "B", and "C" are ...`.
fn some_are_types(names: &[&str]) -> String {
    let mut list = String::default();
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            list.push_str(match (names.len(), index + 1 == names.len()) {
                (2, _) => " and ",
                (_, true) => ", and ",
                _ => ", ",
            });
        }
        list.push_str(&cstr!("\"{name}\""));
    }
    if names.len() == 1 {
        cstr!("Import {list} is only used as a type.")
    } else {
        cstr!("Imports {list} are only used as types.")
    }
}

/// `prefer: "no-type-imports"`: every `type` keyword of the declaration.
fn avoid_type_import(
    site: Site<'_, '_>,
    declaration: &ImportDeclaration<'_>,
    shared: bool,
    uses: &ComponentUses<'_>,
    result: &mut ScriptLintResult,
) {
    const MESSAGE: &str = "Use an `import` instead of an `import type`.";
    const TITLE: &str = "Remove `type`";
    let specifiers = declaration
        .specifiers
        .as_ref()
        .map_or(&[][..], |specifiers| specifiers.as_slice());
    if declaration.import_kind.is_type() {
        let diagnostic = site.warn(declaration.span, MESSAGE);
        if specifiers
            .iter()
            .any(|specifier| uses.metadata_reads(specifier.local(), shared))
        {
            result.add_diagnostic(diagnostic);
            return;
        }
        let placements = vec![Placement::Value; specifiers.len()];
        result.add_diagnostic(match site.rewrite(declaration, &placements) {
            Some(text) => diagnostic.with_fix(site.replacement(declaration.span, TITLE, text)),
            None => diagnostic,
        });
        return;
    }
    for specifier in specifiers {
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            continue;
        };
        if !named.import_kind.is_type() {
            continue;
        }
        let diagnostic = site.warn(named.span, MESSAGE);
        if uses.metadata_reads(&named.local, shared) {
            result.add_diagnostic(diagnostic);
            continue;
        }
        // One small edit per specifier, so a declaration is fixed in one pass.
        result.add_diagnostic(match inline_type_keyword(site.source, named.span) {
            Some(keyword) => {
                diagnostic.with_fix(site.replacement(keyword, TITLE, String::default()))
            }
            None => diagnostic,
        });
    }
}
