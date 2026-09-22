//! One parsed script body with the bindings the provider follows.
//!
//! Every identifier the provider or the typing rule resolves goes through
//! OXC's semantic scoping — a reference either resolves to the symbol a
//! `const`, an `import` or an export binds, or it is not followed at all. A
//! parameter or local that shadows `routes`, `router` or `useRouter` can
//! therefore never be mistaken for the binding it shadows.

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, IdentifierReference, ModuleExportName, Program,
    Statement, VariableDeclarationKind,
};
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder, SymbolId};
use vize_carton::Span;

use crate::providers::ScriptBlock;

/// What an import binding imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Imported<'a> {
    Named(&'a str),
    Default,
    Namespace,
}

/// An import binding: `import { imported as local } from 'source'`.
#[derive(Debug, Clone, Copy)]
pub(super) struct Import<'a> {
    pub source: &'a str,
    pub imported: Imported<'a>,
}

/// What an export name stands for.
#[derive(Debug, Clone, Copy)]
pub(super) enum Export<'a> {
    /// A module-local binding.
    Local(SymbolId),
    /// `export default <expression>`.
    Expr(&'a Expression<'a>),
    /// `export { name } from 'source'`.
    From { source: &'a str, name: &'a str },
}

/// A parsed, scope-resolved script body.
pub(super) struct Script<'a> {
    pub program: &'a Program<'a>,
    semantic: Semantic<'a>,
    exports: Vec<(&'a str, Export<'a>)>,
    offset: u32,
}

impl<'a> Script<'a> {
    /// The symbol `ident` resolves to; `None` for a global.
    pub fn symbol(&self, ident: &IdentifierReference<'a>) -> Option<SymbolId> {
        let reference = ident.reference_id.get()?;
        self.semantic.scoping().get_reference(reference).symbol_id()
    }

    /// The import binding `symbol`.
    pub fn import_of(&self, symbol: SymbolId) -> Option<Import<'a>> {
        let node = self.semantic.symbol_declaration(symbol);
        let imported = match node.kind() {
            AstKind::ImportSpecifier(named) => Imported::Named(named.imported.name().as_str()),
            AstKind::ImportDefaultSpecifier(_) => Imported::Default,
            AstKind::ImportNamespaceSpecifier(_) => Imported::Namespace,
            _ => return None,
        };
        let AstKind::ImportDeclaration(declaration) = self.semantic.nodes().parent_kind(node.id())
        else {
            return None;
        };
        Some(Import {
            source: declaration.source.value.as_str(),
            imported,
        })
    }

    /// The import `ident` resolves to.
    pub fn import(&self, ident: &IdentifierReference<'a>) -> Option<Import<'a>> {
        self.symbol(ident).and_then(|symbol| self.import_of(symbol))
    }

    /// Whether `ident` resolves to `import { name } from 'source'`.
    pub fn is_import_of(&self, ident: &IdentifierReference<'a>, source: &str, name: &str) -> bool {
        self.import(ident).is_some_and(|import| {
            import.source == source && import.imported == Imported::Named(name)
        })
    }

    /// The initializer of the `const` binding `symbol` (`const x = init`).
    pub fn const_of(&self, symbol: SymbolId) -> Option<&'a Expression<'a>> {
        let AstKind::VariableDeclarator(declarator) =
            self.semantic.symbol_declaration(symbol).kind()
        else {
            return None;
        };
        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
            return None;
        };
        (declarator.kind == VariableDeclarationKind::Const
            && binding.symbol_id.get() == Some(symbol))
        .then_some(declarator.init.as_ref())
        .flatten()
    }

    /// The initializer of the `const` `ident` resolves to.
    pub fn const_init(&self, ident: &IdentifierReference<'a>) -> Option<&'a Expression<'a>> {
        self.symbol(ident).and_then(|symbol| self.const_of(symbol))
    }

    /// Every export name `symbol` is reachable under.
    pub fn export_names(&self, symbol: SymbolId) -> impl Iterator<Item = &'a str> + '_ {
        self.exports
            .iter()
            .filter_map(move |(name, export)| match export {
                Export::Local(local) if *local == symbol => Some(*name),
                Export::Expr(Expression::Identifier(ident))
                    if self.symbol(ident) == Some(symbol) =>
                {
                    Some(*name)
                }
                _ => None,
            })
    }

    /// The export named `name`.
    pub fn export(&self, name: &str) -> Option<Export<'a>> {
        self.exports
            .iter()
            .find(|(exported, _)| *exported == name)
            .map(|(_, export)| *export)
    }

    /// A body-relative OXC span as a whole-file span.
    pub fn span(&self, span: oxc_span::Span) -> Span {
        Span::new(span.start + self.offset, span.end + self.offset)
    }
}

/// Parse `block`, resolve its scopes, and run `f` over it. `None` when the
/// body does not parse cleanly: a provider never reads a recovered AST.
pub(super) fn with_script<R>(
    block: ScriptBlock<'_>,
    f: impl FnOnce(&Script<'_>) -> R,
) -> Option<R> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, block.text, block.source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let program = allocator.alloc(parsed.program);
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
        .semantic;
    let mut script = Script {
        program,
        semantic,
        exports: Vec::new(),
        offset: block.offset,
    };
    script.exports = collect_exports(&script);
    Some(f(&script))
}

fn collect_exports<'a>(script: &Script<'a>) -> Vec<(&'a str, Export<'a>)> {
    let mut exports = Vec::new();
    for statement in &script.program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
                    for declarator in &declaration.declarations {
                        if let BindingPattern::BindingIdentifier(binding) = &declarator.id
                            && let Some(symbol) = binding.symbol_id.get()
                        {
                            exports.push((binding.name.as_str(), Export::Local(symbol)));
                        }
                    }
                }
                for specifier in &export.specifiers {
                    let exported = specifier.exported.name().as_str();
                    if let Some(source) = &export.source {
                        let name = specifier.local.name().as_str();
                        let source = source.value.as_str();
                        exports.push((exported, Export::From { source, name }));
                    } else if let ModuleExportName::IdentifierReference(local) = &specifier.local
                        && let Some(symbol) = script.symbol(local)
                    {
                        exports.push((exported, Export::Local(symbol)));
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(expression) = export.declaration.as_expression() {
                    exports.push(("default", Export::Expr(expression)));
                }
            }
            _ => {}
        }
    }
    exports
}
