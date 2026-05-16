//! Import processing utilities.
//!
//! This module handles processing import statements, including
//! removing TypeScript type-only imports and extracting identifiers.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

use vize_carton::ToCompactString;

/// Process import statement to remove TypeScript type-only imports using OXC
/// Returns None if the entire import should be removed, Some(processed) otherwise
pub fn process_import_for_types(import: &str) -> Option<vize_carton::String> {
    let import = import.trim();

    // Parse the import statement with OXC
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser = Parser::new(&allocator, import, source_type);
    let result = parser.parse();

    if result.errors.is_empty() {
        for stmt in &result.program.body {
            if let Statement::ImportDeclaration(decl) = stmt {
                // Skip type-only imports: import type { ... } from '...'
                if decl.import_kind.is_type() {
                    return None;
                }

                // Check if there are any specifiers
                if let Some(specifiers) = &decl.specifiers {
                    // Filter out type-only specifiers
                    let value_specifiers: Vec<&ImportDeclarationSpecifier> = specifiers
                        .iter()
                        .filter(|spec| match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                !s.import_kind.is_type()
                            }
                            _ => true,
                        })
                        .collect();

                    if value_specifiers.is_empty() {
                        // All specifiers were type imports
                        return None;
                    }

                    if value_specifiers.len() != specifiers.len() {
                        // Some specifiers were filtered out, rebuild the import
                        let source = decl.source.value.as_str();

                        // Separate default/namespace imports from named imports
                        let mut default_part: Option<vize_carton::String> = None;
                        let mut named_parts: Vec<vize_carton::String> = Vec::new();

                        for spec in &value_specifiers {
                            match spec {
                                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                    let imported = s.imported.name().as_str();
                                    let local = s.local.name.as_str();
                                    if imported == local {
                                        named_parts.push(imported.to_compact_string());
                                    } else {
                                        let mut name = vize_carton::String::with_capacity(
                                            imported.len() + local.len() + 4,
                                        );
                                        name.push_str(imported);
                                        name.push_str(" as ");
                                        name.push_str(local);
                                        named_parts.push(name);
                                    }
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                    default_part = Some(s.local.name.to_compact_string());
                                }
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                    let local = s.local.name.as_str();
                                    let mut name =
                                        vize_carton::String::with_capacity(local.len() + 5);
                                    name.push_str("* as ");
                                    name.push_str(local);
                                    default_part = Some(name);
                                }
                            }
                        }

                        let mut new_import = vize_carton::String::with_capacity(source.len() + 30);
                        new_import.push_str("import ");
                        if let Some(ref def) = default_part {
                            new_import.push_str(def);
                            if !named_parts.is_empty() {
                                new_import.push_str(", ");
                            }
                        }
                        if !named_parts.is_empty() {
                            new_import.push_str("{ ");
                            // [CompactString].join() returns std String, convert back
                            let joined = named_parts
                                .iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join(", ");
                            new_import.push_str(&joined);
                            new_import.push_str(" }");
                        }
                        new_import.push_str(" from '");
                        new_import.push_str(source);
                        new_import.push_str("'\n");
                        return Some(new_import);
                    }
                }
            }
        }
    }

    // Regular import or parse failed, return as-is
    let mut result = import.to_compact_string();
    result.push('\n');
    Some(result)
}

/// Extract all identifiers from an import statement (including default imports)
pub fn extract_import_identifiers(import: &str) -> Vec<vize_carton::String> {
    let import = import.trim();
    let mut identifiers = Vec::new();

    // Parse the import statement with OXC
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser = Parser::new(&allocator, import, source_type);
    let result = parser.parse();

    if result.errors.is_empty() {
        for stmt in &result.program.body {
            if let Statement::ImportDeclaration(decl) = stmt {
                // Skip type-only imports
                if decl.import_kind.is_type() {
                    continue;
                }

                if let Some(specifiers) = &decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                // Skip type-only specifiers
                                if !s.import_kind.is_type() {
                                    identifiers.push(s.local.name.to_compact_string());
                                }
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                identifiers.push(s.local.name.to_compact_string());
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                identifiers.push(s.local.name.to_compact_string());
                            }
                        }
                    }
                }
            }
        }
    }

    identifiers
}

#[cfg(test)]
mod tests {
    use super::process_import_for_types;

    #[test]
    fn test_default_import_with_type_named_import() {
        // `import Foo, { type Bar }` should become `import Foo from '...'`
        // NOT `import { Foo } from '...'`
        let input = "import AtriumSegmentedTabs, { type AtriumSegmentedTabConfig } from '../AtriumSegmentedTabs/AtriumSegmentedTabs.vue'";
        let result = process_import_for_types(input);
        let output = result.expect("should produce an import");
        insta::assert_snapshot!(output.as_str());
    }

    #[test]
    fn test_default_import_with_mixed_named_imports() {
        // `import Foo, { type Bar, baz }` should become `import Foo, { baz } from '...'`
        let input = "import Foo, { type Bar, baz } from 'module'";
        let result = process_import_for_types(input);
        let output = result.expect("should produce an import");
        insta::assert_snapshot!(output.as_str());
    }

    #[test]
    fn test_type_only_import_returns_none() {
        let input = "import type { Foo } from 'bar'";
        let result = process_import_for_types(input);
        assert!(result.is_none(), "Type-only import should return None");
    }

    #[test]
    fn test_all_named_type_imports_returns_none() {
        let input = "import { type Foo, type Bar } from 'baz'";
        let result = process_import_for_types(input);
        assert!(
            result.is_none(),
            "All type-only named imports should return None"
        );
    }

    #[test]
    fn test_normal_import_unchanged() {
        let input = "import { foo, bar } from 'module'";
        let result = process_import_for_types(input);
        assert!(result.is_some(), "Normal import should be returned as-is");
    }
}
