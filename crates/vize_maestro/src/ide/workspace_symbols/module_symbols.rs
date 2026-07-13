//! Workspace symbols projected from the persistent neutral module frontend.

use tower_lsp::lsp_types::{Location, Range, SymbolInformation, SymbolKind, Url};
use vize_module::{
    ModuleBindingKind, ModuleDeclaration, ModuleDocument, ModuleExpressionKind,
    ModuleOperationKind, ModuleSyntax,
};

pub(super) fn collect(
    uri: &Url,
    source: &str,
    document: &ModuleDocument,
    croquis: &vize_croquis::Croquis,
    query: &str,
    symbols: &mut Vec<SymbolInformation>,
) {
    for module in &document.modules {
        let container = if module.name.ends_with("#script-setup") {
            "script setup"
        } else {
            "script"
        };
        for declaration in &module.declarations {
            let name = declaration.name.as_ref();
            if !name.to_lowercase().contains(query) || is_import(module, name) {
                continue;
            }
            let start = super::WorkspaceSymbolsService::offset_to_position(
                source,
                declaration.span.start as usize,
            );
            let end = super::WorkspaceSymbolsService::offset_to_position(
                source,
                declaration.span.end as usize,
            );
            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: symbol_kind(module, declaration, croquis),
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range: Range { start, end },
                },
                container_name: Some(container.to_string()),
            });
        }
    }
}

fn is_import(module: &ModuleSyntax, name: &str) -> bool {
    module
        .imports
        .iter()
        .flat_map(|import| &import.bindings)
        .any(|binding| binding.local.as_ref() == name)
}

fn symbol_kind(
    module: &ModuleSyntax,
    declaration: &ModuleDeclaration,
    croquis: &vize_croquis::Croquis,
) -> SymbolKind {
    if let Some(kind) = declaration_syntax_kind(module, declaration) {
        return kind;
    }
    if module.operations.functions.iter().any(|function| {
        function.name.as_deref() == Some(declaration.name.as_ref())
            && contains(function.span, declaration)
    }) {
        return SymbolKind::FUNCTION;
    }
    if let Some(operation) = module.operations.operations.iter().find(|operation| {
        contains(operation.span, declaration)
            && matches!(operation.kind, ModuleOperationKind::Binding { .. })
    }) && let ModuleOperationKind::Binding {
        kind, initializer, ..
    } = &operation.kind
    {
        if initializer
            .as_ref()
            .is_some_and(|value| matches!(value.kind, ModuleExpressionKind::Function { .. }))
        {
            return SymbolKind::FUNCTION;
        }
        return match kind {
            ModuleBindingKind::Let | ModuleBindingKind::Var | ModuleBindingKind::Other => {
                SymbolKind::VARIABLE
            }
            ModuleBindingKind::Const if croquis.reactivity.lookup(&declaration.name).is_some() => {
                SymbolKind::VARIABLE
            }
            ModuleBindingKind::Const => SymbolKind::CONSTANT,
        };
    }
    SymbolKind::VARIABLE
}

fn declaration_syntax_kind(
    module: &ModuleSyntax,
    declaration: &ModuleDeclaration,
) -> Option<SymbolKind> {
    let local = declaration.span.start.saturating_sub(module.base_offset) as usize;
    let prefix = module.source.get(..local)?;
    let line = prefix.rsplit_once('\n').map_or(prefix, |(_, line)| line);
    let line = line.trim_start();
    if line.ends_with("class ") || line.ends_with("abstract class ") {
        Some(SymbolKind::CLASS)
    } else if line.ends_with("interface ") {
        Some(SymbolKind::INTERFACE)
    } else if line.ends_with("type ") {
        Some(SymbolKind::TYPE_PARAMETER)
    } else if line.ends_with("enum ") {
        Some(SymbolKind::ENUM)
    } else if line.ends_with("function ") {
        Some(SymbolKind::FUNCTION)
    } else {
        None
    }
}

fn contains(span: vize_module::ModuleSpan, declaration: &ModuleDeclaration) -> bool {
    span.start <= declaration.span.start && declaration.span.end <= span.end
}
