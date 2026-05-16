//! Import deduplication for function-mode script compilation.
//!
//! Handles removing duplicate import specifiers from the same source module
//! to avoid "Identifier has already been declared" errors.

use vize_carton::{FxHashSet, String, ToCompactString};

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::super::import_utils::process_import_for_types;

/// Deduplicate imports by removing duplicate specifiers from the same source.
/// This avoids "Identifier has already been declared" errors.
pub fn dedupe_imports(imports: &[String], is_ts: bool) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut seen_specifiers: FxHashSet<String> = FxHashSet::default();

    for import in imports {
        let processed = if is_ts {
            // In TS mode, preserve type imports as-is (TypeScript handles them)
            let mut s = import.trim().to_compact_string();
            s.push('\n');
            s
        } else {
            let Some(p) = process_import_for_types(import) else {
                continue;
            };
            p
        };
        let trimmed = processed.trim();
        if trimmed.is_empty() {
            continue;
        }

        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, trimmed, SourceType::ts());
        let parse_result = parser.parse();

        if !parse_result.errors.is_empty() {
            if seen_specifiers.insert(trimmed.to_compact_string()) {
                let mut s = trimmed.to_compact_string();
                s.push('\n');
                result.push(s);
            }
            continue;
        }

        let mut handled = false;

        for stmt in &parse_result.program.body {
            if let Statement::ImportDeclaration(decl) = stmt {
                let source = decl.source.value.as_str();

                let is_type_only_import = decl.import_kind.is_type();

                if decl.specifiers.is_none() {
                    // Side-effect import: import 'module';
                    let mut key = String::with_capacity(source.len() + 13);
                    key.push_str(source);
                    key.push_str("::side-effect");
                    if seen_specifiers.insert(key) {
                        let mut line = String::with_capacity(source.len() + 12);
                        line.push_str("import '");
                        line.push_str(source);
                        line.push_str("'\n");
                        result.push(line);
                    }
                    handled = true;
                    break;
                }

                let mut default_spec: Option<String> = None;
                let mut namespace_spec: Option<String> = None;
                let mut named_specs: Vec<String> = Vec::new();

                if let Some(specifiers) = &decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let local = s.local.name.as_str();
                                let mut key = String::with_capacity(source.len() + local.len() + 2);
                                key.push_str(source);
                                key.push_str("::");
                                key.push_str(local);
                                if !seen_specifiers.insert(key) {
                                    continue;
                                }

                                let imported = s.imported.name().as_str();
                                let type_prefix = if is_ts && s.import_kind.is_type() {
                                    "type "
                                } else {
                                    ""
                                };
                                if imported == local {
                                    let mut spec =
                                        String::with_capacity(type_prefix.len() + imported.len());
                                    spec.push_str(type_prefix);
                                    spec.push_str(imported);
                                    named_specs.push(spec);
                                } else {
                                    let mut spec = String::with_capacity(
                                        type_prefix.len() + imported.len() + 4 + local.len(),
                                    );
                                    spec.push_str(type_prefix);
                                    spec.push_str(imported);
                                    spec.push_str(" as ");
                                    spec.push_str(local);
                                    named_specs.push(spec);
                                }
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                let local = s.local.name.as_str();
                                let mut key = String::with_capacity(source.len() + local.len() + 2);
                                key.push_str(source);
                                key.push_str("::");
                                key.push_str(local);
                                if seen_specifiers.insert(key) {
                                    default_spec = Some(local.to_compact_string());
                                }
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                let local = s.local.name.as_str();
                                let mut key = String::with_capacity(source.len() + local.len() + 2);
                                key.push_str(source);
                                key.push_str("::");
                                key.push_str(local);
                                if seen_specifiers.insert(key) {
                                    namespace_spec = Some(local.to_compact_string());
                                }
                            }
                        }
                    }
                }

                if default_spec.is_none() && namespace_spec.is_none() && named_specs.is_empty() {
                    handled = true;
                    break;
                }

                let mut parts: Vec<String> = Vec::new();
                if let Some(def) = default_spec {
                    parts.push(def);
                }
                if let Some(ns) = namespace_spec {
                    let mut name = String::with_capacity(ns.len() + 5);
                    name.push_str("* as ");
                    name.push_str(&ns);
                    parts.push(name);
                }
                if !named_specs.is_empty() {
                    let joined = named_specs
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let mut part = String::with_capacity(joined.len() + 4);
                    part.push_str("{ ");
                    part.push_str(&joined);
                    part.push_str(" }");
                    parts.push(part);
                }

                let joined = parts
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut line = String::with_capacity(joined.len() + source.len() + 18);
                if is_ts && is_type_only_import {
                    line.push_str("import type ");
                } else {
                    line.push_str("import ");
                }
                line.push_str(&joined);
                line.push_str(" from '");
                line.push_str(source);
                line.push_str("'\n");
                result.push(line);
                handled = true;
                break;
            }
        }

        if !handled && seen_specifiers.insert(trimmed.to_compact_string()) {
            let mut s = trimmed.to_compact_string();
            s.push('\n');
            result.push(s);
        }
    }

    result
}
