use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_syntax::symbol::{SymbolFlags, SymbolId};

/// Erased ambient/type-import declarations do not provide a runtime global.
pub(super) fn runtime_shadow(semantic: &Semantic<'_>, symbol: SymbolId) -> bool {
    if semantic
        .scoping()
        .symbol_flags(symbol)
        .contains(SymbolFlags::TypeImport)
    {
        return false;
    }
    let declaration = semantic.symbol_declaration(symbol);
    !std::iter::once(declaration.kind())
        .chain(semantic.nodes().ancestor_kinds(declaration.id()))
        .any(|kind| match kind {
            AstKind::VariableDeclaration(declaration) => declaration.declare,
            AstKind::Function(function) => function.declare,
            AstKind::Class(class) => class.declare,
            AstKind::TSModuleDeclaration(module) => module.declare,
            _ => false,
        })
}
