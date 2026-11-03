//! Declaration helpers belong to each emitted library, never the global scope.

use std::sync::OnceLock;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, append};

use crate::virtual_ts::DECLARATION_HELPERS_DTS;

// A declaration consumer supplies fallthrough attributes through Vue's
// component contract. Keep declared prop value and required-key checks, while
// the mapper's ordinary construction checks retain their stricter extra keys.
const DECLARATION_INPUT_GUARD: &str = "type $ComponentInputGuardFor<T, _Declared, Authored, _Allowed> = $ComponentMissingInputGuard<T, Authored>;";

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
        let guard = parsed
            .program
            .body
            .iter()
            .find_map(|statement| match statement {
                Statement::TSTypeAliasDeclaration(alias)
                    if alias.id.name == "$ComponentInputGuardFor" =>
                {
                    Some(alias.span)
                }
                _ => None,
            });
        let exports = names.join(", ");
        let mut module = String::from(DECLARATION_HELPERS_DTS);
        if let Some(guard) = guard {
            module.replace_range(
                guard.start as usize..guard.end as usize,
                DECLARATION_INPUT_GUARD,
            );
        }
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
        // `__EmitProps` is referenced through a relative import type so an
        // authored declaration with the same lexical name cannot capture it.
        .filter(|name| name.as_str() != "__EmitProps")
        .filter(|name| identifiers.contains(name.as_str()))
        .map(|name| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let specifier = module_specifier(depth);
    vize_carton::cstr!("import type {{ {names} }} from \"{specifier}\";\n")
}

pub(super) fn module_specifier(depth: usize) -> String {
    let directory = if depth == 0 {
        "./".into()
    } else {
        "../".repeat(depth)
    };
    vize_carton::cstr!("{directory}__vize_helpers.js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declaration_fallthrough_changes_only_the_owned_guard_alias() {
        let allocator = Allocator::default();
        let original = Parser::new(&allocator, DECLARATION_HELPERS_DTS, SourceType::d_ts()).parse();
        let old = original
            .program
            .body
            .iter()
            .find_map(|statement| match statement {
                Statement::TSTypeAliasDeclaration(alias)
                    if alias.id.name == "$ComponentInputGuardFor" =>
                {
                    DECLARATION_HELPERS_DTS.get(alias.span.start as usize..alias.span.end as usize)
                }
                _ => None,
            })
            .unwrap();
        let allocator = Allocator::default();
        let generated = Parser::new(&allocator, module(), SourceType::d_ts()).parse();
        assert_eq!(generated.diagnostics.len(), 0);
        let aliases: Vec<_> = generated
            .program
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::TSTypeAliasDeclaration(alias)
                    if alias.id.name == "$ComponentInputGuardFor" =>
                {
                    module().get(alias.span.start as usize..alias.span.end as usize)
                }
                _ => None,
            })
            .collect();
        assert_eq!(aliases, vec![DECLARATION_INPUT_GUARD]);
        let exports = helpers().0.join(", ");
        let expected =
            vize_carton::cstr!("{DECLARATION_HELPERS_DTS}\nexport type {{ {exports} }};\n");
        assert_eq!(
            module().replacen(DECLARATION_INPUT_GUARD, old, 1),
            expected.as_str()
        );
    }

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
        assert_eq!(
            import_for(
                "type __EmitProps = 'authored'; type Public = import('./__vize_helpers.js').__EmitProps<{}>;",
                0
            ),
            "import type {  } from \"./__vize_helpers.js\";\n"
        );
        assert_eq!(
            helpers()
                .0
                .iter()
                .filter(|name| name.as_str() == "__EmitProps")
                .count(),
            1
        );
    }
}
