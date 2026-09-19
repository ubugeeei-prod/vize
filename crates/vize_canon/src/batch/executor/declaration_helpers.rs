//! Declaration helpers belong to each emitted library, never the global scope.

use std::sync::OnceLock;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, append};

use crate::virtual_ts::DECLARATION_HELPERS_DTS;

fn helpers() -> &'static (Vec<CompactString>, String) {
    static HELPERS: OnceLock<(Vec<CompactString>, String)> = OnceLock::new();
    HELPERS.get_or_init(|| {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, DECLARATION_HELPERS_DTS, SourceType::d_ts()).parse();
        let names: Vec<CompactString> = parsed
            .program
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::TSTypeAliasDeclaration(alias) => Some(alias.id.name.as_str().into()),
                Statement::TSInterfaceDeclaration(interface) => {
                    Some(interface.id.name.as_str().into())
                }
                _ => None,
            })
            .collect();
        let exports = names.join(", ");
        let mut module = String::from(DECLARATION_HELPERS_DTS);
        append!(module, "\nexport type {{ {exports} }};\n");
        (names, module)
    })
}

pub(super) fn module() -> &'static str {
    helpers().1.as_str()
}

pub(super) fn import_for(declaration: &str, depth: usize) -> String {
    // Keep only identifiers used by this output. This bounds declaration size
    // without relying on the private helper list remaining fixed over time.
    let identifiers: FxHashSet<_> = declaration
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .collect();
    let names = helpers()
        .0
        .iter()
        .filter(|name| identifiers.contains(name.as_str()))
        .map(|name| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let directory = if depth == 0 {
        "./".into()
    } else {
        "../".repeat(depth)
    };
    vize_carton::cstr!("import type {{ {names} }} from \"{directory}__vize_helpers.js\";\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_are_parseable_module_exports_and_imports_preserve_depth() {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, module(), SourceType::d_ts()).parse();
        assert!(!parsed.panicked && parsed.diagnostics.is_empty());
        assert!(helpers().0.len() > 50);
        assert_eq!(
            import_for(
                "type Public = __EmitFn<{ save: [] }>; type __EmitFnSuffix = never;",
                2
            ),
            "import type { __EmitFn } from \"../../__vize_helpers.js\";\n"
        );
        assert_eq!(
            import_for("export const value: number;", 0),
            "import type {  } from \"./__vize_helpers.js\";\n"
        );
    }
}
