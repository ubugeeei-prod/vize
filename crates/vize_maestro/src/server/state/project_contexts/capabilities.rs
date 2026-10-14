//! Advertise the union for composite workspaces; requests retain local gates.

use super::LspFeatureConfig;
use std::path::Path;

pub(super) fn merge(target: &mut LspFeatureConfig, source: LspFeatureConfig) {
    macro_rules! merge { ($($field:ident),* $(,)?) => { $(target.$field |= source.$field;)* }; }
    merge!(
        lint,
        typecheck,
        ecosystem,
        options_api,
        legacy_vue2,
        completion,
        signature_help,
        hover,
        definition,
        references,
        document_symbols,
        workspace_symbols,
        code_actions,
        rename,
        formatting,
        code_lens,
        semantic_tokens,
        document_links,
        folding_ranges,
        inlay_hints,
        file_rename,
        auto_insert,
        cross_file
    );
}

pub(super) fn is_composite_project(root: &Path) -> bool {
    if root.join("pnpm-workspace.yaml").is_file() {
        return true;
    }
    [
        ("package.json", "workspaces"),
        ("tsconfig.json", "references"),
    ]
    .into_iter()
    .any(|(filename, key)| {
        let Ok(source) = std::fs::read_to_string(root.join(filename)) else {
            return false;
        };
        let allocator = oxc_allocator::Allocator::default();
        let Ok(oxc_ast::ast::Expression::ObjectExpression(object)) =
            oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::mjs())
                .parse_expression()
        else {
            return false;
        };
        object.properties.iter().any(|property| {
            let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            property.key.static_name().is_some_and(|name| name == key)
                && match &property.value {
                    oxc_ast::ast::Expression::ArrayExpression(value) => !value.elements.is_empty(),
                    oxc_ast::ast::Expression::ObjectExpression(value) => {
                        !value.properties.is_empty()
                    }
                    _ => false,
                }
        })
    })
}
