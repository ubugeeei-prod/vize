//! Binding metadata conversion and registration for SFC compilation.
//!
//! Handles converting between Croquis and legacy binding formats,
//! and registering bindings from normal `<script>` blocks.

use vize_carton::ToCompactString;

use crate::types::{BindingMetadata, BindingType};

/// Convert Croquis BindingMetadata (CompactString keys) to legacy BindingMetadata (String keys)
pub(super) fn croquis_to_legacy_bindings(
    src: &vize_croquis::analysis::BindingMetadata,
) -> BindingMetadata {
    let mut dst = BindingMetadata::default();
    dst.is_script_setup = src.is_script_setup;
    for (name, bt) in src.iter() {
        dst.bindings.insert(name.to_compact_string(), bt);
    }
    for (local, key) in &src.props_aliases {
        dst.props_aliases
            .insert(local.to_compact_string(), key.to_compact_string());
    }
    dst
}

/// Register bindings from normal `<script>` block into `BindingMetadata`.
///
/// When both `<script>` and `<script setup>` exist, all imports and exported
/// declarations from the normal script are accessible in the template.
/// Uses OXC parser for accurate import extraction (handles `import { Form as PForm }`,
/// default imports, namespace imports, re-exports, etc.).
pub(super) fn register_normal_script_bindings(content: &str, bindings: &mut BindingMetadata) {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Declaration, ImportDeclarationSpecifier, Statement};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return;
    }

    for stmt in ret.program.body.iter() {
        match stmt {
            // Register import bindings: import { Foo, Bar as Baz } from '...'
            Statement::ImportDeclaration(decl) => {
                // Skip type-only imports (import type { ... } from '...')
                if decl.import_kind.is_type() {
                    continue;
                }
                if let Some(specifiers) = &decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                // Skip type-only specifiers
                                if s.import_kind.is_type() {
                                    continue;
                                }
                                let local = s.local.name.to_compact_string();
                                bindings
                                    .bindings
                                    .entry(local)
                                    .or_insert(BindingType::SetupConst);
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                let local = s.local.name.to_compact_string();
                                bindings
                                    .bindings
                                    .entry(local)
                                    .or_insert(BindingType::SetupConst);
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                let local = s.local.name.to_compact_string();
                                bindings
                                    .bindings
                                    .entry(local)
                                    .or_insert(BindingType::SetupConst);
                            }
                        }
                    }
                }
            }
            // Register exported variable declarations: export const foo = ...
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(ref declaration) = decl.declaration {
                    if let Declaration::VariableDeclaration(var_decl) = declaration {
                        for declarator in &var_decl.declarations {
                            if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) =
                                &declarator.id
                            {
                                bindings
                                    .bindings
                                    .entry(id.name.to_compact_string())
                                    .or_insert(BindingType::SetupConst);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
