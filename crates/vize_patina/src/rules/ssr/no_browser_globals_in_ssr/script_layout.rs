//! Model Vue's module and setup lexical frames using the same authored AST.
//! Only private rule-local nodes are rearranged; reference spans stay physical.

use super::BROWSER_GLOBALS;
use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::{
    ast::{
        Expression, FormalParameterKind, FormalParameters, FunctionBody, ImportDeclaration,
        ImportDeclarationSpecifier, Program, Statement,
    },
    builder::{AstBuilder, NONE},
};
use oxc_span::{GetSpan, SourceType, Span};
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::String;

pub(super) struct Source {
    pub text: String,
    pub source_type: SourceType,
    module: Option<Span>,
    setup: Option<Span>,
}

pub(super) fn candidate(source: &str) -> bool {
    source.contains('\\') || BROWSER_GLOBALS.iter().any(|name| source.contains(name))
}

pub(super) fn prepare(descriptor: &SfcDescriptor<'_>) -> Option<Source> {
    let blocks: Vec<_> = descriptor
        .script
        .iter()
        .chain(descriptor.script_setup.iter())
        .collect();
    if blocks.iter().all(|block| !candidate(&block.content)) {
        return None;
    }
    let mut bytes = vec![b' '; descriptor.source.len()];
    let mut language = None;
    let mut spans = Vec::new();
    for block in blocks {
        let lang = block.lang.as_deref().unwrap_or("js");
        if block.src.is_some()
            || !matches!(lang, "js" | "ts" | "jsx" | "tsx")
            || language.is_some_and(|previous| previous != lang)
        {
            return None;
        }
        language = Some(lang);
        let start = block.loc.start;
        let end = start.checked_add(block.content.len())?;
        if end != block.loc.end || descriptor.source.get(start..end) != Some(block.content.as_ref())
        {
            return None;
        }
        bytes
            .get_mut(start..end)?
            .copy_from_slice(block.content.as_bytes());
        spans.push(Span::new(
            u32::try_from(start).ok()?,
            u32::try_from(end).ok()?,
        ));
    }
    let module = descriptor
        .script
        .as_ref()
        .and_then(|_| spans.first().copied());
    let setup = descriptor
        .script_setup
        .as_ref()
        .and_then(|_| spans.last().copied());
    if let (Some(module), Some(setup)) = (module, setup)
        && module.start < setup.end
        && setup.start < module.end
    {
        return None;
    }
    let path = match language? {
        "ts" => "component.ts",
        "tsx" => "component.tsx",
        "jsx" => "component.jsx",
        _ => "component.js",
    };
    Some(Source {
        text: String::from_utf8(bytes).ok()?,
        source_type: SourceType::from_path(path).ok()?,
        module,
        setup,
    })
}

impl Source {
    /// The real Vue compiler nests setup locals but hoists setup imports. This
    /// private anonymous IIFE expresses that relation without reparsing text.
    pub(super) fn apply<'a>(
        &self,
        program: &mut Program<'a>,
        allocator: &'a Allocator,
    ) -> Option<()> {
        let Some(setup_span) = self.setup else {
            return Some(());
        };
        let builder = AstBuilder::new(allocator);
        let mut setup = ArenaVec::new_in(&builder);
        let mut setup_imports = ArenaVec::new_in(&builder);
        let statements = std::mem::replace(&mut program.body, ArenaVec::new_in(&builder));
        for statement in statements {
            let span = statement.span();
            if contains(setup_span, span) {
                match statement {
                    Statement::ImportDeclaration(_) => setup_imports.push(statement),
                    Statement::ExportNamedDeclaration(_)
                    | Statement::ExportDefaultDeclaration(_)
                    | Statement::ExportAllDeclaration(_) => return None,
                    _ => setup.push(statement),
                }
            } else if self.module.is_some_and(|module| contains(module, span)) {
                program.body.push(statement);
            } else {
                // ASI/continuations must not create a statement across blocks.
                return None;
            }
        }
        let mut setup_directives = ArenaVec::new_in(&builder);
        let directives = std::mem::replace(&mut program.directives, ArenaVec::new_in(&builder));
        for directive in directives {
            if contains(setup_span, directive.span) {
                setup_directives.push(directive);
            } else if self
                .module
                .is_some_and(|module| contains(module, directive.span))
            {
                program.directives.push(directive);
            } else {
                return None;
            }
        }
        let mut imports = Vec::new();
        for statement in &program.body {
            if let Statement::ImportDeclaration(import) = statement {
                import_keys(import, &mut imports)?;
            }
        }
        for statement in setup_imports {
            let Statement::ImportDeclaration(mut import) = statement else {
                return None;
            };
            if import.phase.is_some() || import.with_clause.is_some() {
                return None;
            }
            if let Some(specifiers) = import.specifiers.take() {
                let mut retained = ArenaVec::new_in(&builder);
                for specifier in specifiers {
                    let key = import_key(&specifier, import.source.value.as_str());
                    if let Some(previous) = imports.iter().find(|previous| previous.0 == key.0) {
                        if *previous != key {
                            return None;
                        }
                        // Vue deduplicates the same source/imported/local binding;
                        // preserve its first authored type/value declaration.
                        continue;
                    }
                    imports.push(key);
                    retained.push(specifier);
                }
                import.specifiers = Some(retained);
            }
            program.body.push(Statement::ImportDeclaration(import));
        }
        let params = FormalParameters::boxed(
            setup_span,
            FormalParameterKind::ArrowFormalParameters,
            ArenaVec::new_in(&builder),
            NONE,
            &builder,
        );
        let body = FunctionBody::boxed(setup_span, setup_directives, setup, &builder);
        let arrow = Expression::new_arrow_function_expression(
            setup_span, false, true, NONE, params, NONE, body, &builder,
        );
        let call = Expression::new_call_expression(
            setup_span,
            arrow,
            NONE,
            ArenaVec::new_in(&builder),
            false,
            &builder,
        );
        program.body.push(Statement::new_expression_statement(
            setup_span, call, &builder,
        ));
        Some(())
    }
}

fn contains(owner: Span, span: Span) -> bool {
    owner.start <= span.start && span.end <= owner.end
}

type ImportKey = (String, String, String);

fn import_keys(import: &ImportDeclaration<'_>, keys: &mut Vec<ImportKey>) -> Option<()> {
    if import.phase.is_some() || import.with_clause.is_some() {
        return None;
    }
    for specifier in import.specifiers.iter().flatten() {
        keys.push(import_key(specifier, import.source.value.as_str()));
    }
    Some(())
}

fn import_key(specifier: &ImportDeclarationSpecifier<'_>, source: &str) -> ImportKey {
    let (local, imported) = match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
            specifier.local.name.as_str(),
            specifier.imported.name().as_str(),
        ),
        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
            (specifier.local.name.as_str(), "default")
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
            (specifier.local.name.as_str(), "*")
        }
    };
    (
        String::from(local),
        String::from(source),
        String::from(imported),
    )
}
