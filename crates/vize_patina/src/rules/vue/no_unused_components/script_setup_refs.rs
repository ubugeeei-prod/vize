use super::NoUnusedComponents;
use crate::context::LintContext;
use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier, TSType};
use oxc_ast_visit::{Visit, walk::walk_ts_type};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{CompactString, FxHashSet};

pub(super) fn script_setup_component_import_references(
    ctx: &LintContext<'_>,
    candidate_names: &[CompactString],
    import_statement_ranges: &[(u32, u32)],
) -> FxHashSet<CompactString> {
    if candidate_names.is_empty() {
        return FxHashSet::default();
    }

    let Some(script_setup) = ctx
        .sfc_descriptor()
        .and_then(|descriptor| descriptor.script_setup.as_ref())
    else {
        return FxHashSet::default();
    };

    let source = script_setup.content.as_ref();
    if !candidate_names.iter().any(|name| {
        has_non_import_identifier_reference(source, name.as_str(), import_statement_ranges)
    }) {
        return FxHashSet::default();
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("component.ts").unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return FxHashSet::default();
    }

    let mut visitor = ScriptSetupComponentImportVisitor {
        component_imports: FxHashSet::default(),
        referenced_imports: FxHashSet::default(),
        scopes: Vec::new(),
        type_depth: 0,
    };
    visitor.visit_program(&parsed.program);
    visitor.referenced_imports
}

fn has_non_import_identifier_reference(
    source: &str,
    name: &str,
    import_statement_ranges: &[(u32, u32)],
) -> bool {
    source.match_indices(name).any(|(index, _)| {
        let start = index as u32;
        let end = (index + name.len()) as u32;
        !import_statement_ranges
            .iter()
            .any(|(import_start, import_end)| start >= *import_start && end <= *import_end)
            && is_identifier_boundary(source.as_bytes().get(index.wrapping_sub(1)).copied())
            && is_identifier_boundary(source.as_bytes().get(index + name.len()).copied())
    })
}

fn is_identifier_boundary(byte: Option<u8>) -> bool {
    !byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$')
}

struct ScriptSetupComponentImportVisitor {
    component_imports: FxHashSet<CompactString>,
    referenced_imports: FxHashSet<CompactString>,
    scopes: Vec<FxHashSet<CompactString>>,
    type_depth: usize,
}

impl<'a> Visit<'a> for ScriptSetupComponentImportVisitor {
    fn enter_scope(
        &mut self,
        _flags: oxc_syntax::scope::ScopeFlags,
        _scope_id: &std::cell::Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        self.scopes.push(FxHashSet::default());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.import_kind.is_type()
            || !NoUnusedComponents::is_component_import_source(it.source.value.as_str())
        {
            return;
        }

        let Some(specifiers) = &it.specifiers else {
            return;
        };

        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if !specifier.import_kind.is_type() =>
                {
                    self.component_imports
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.component_imports
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.component_imports
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
                _ => {}
            }
        }
    }

    fn visit_binding_identifier(&mut self, it: &oxc_ast::ast::BindingIdentifier<'a>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(CompactString::new(it.name.as_str()));
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.type_depth > 0 {
            return;
        }

        let name = it.name.as_str();
        if self.component_imports.contains(name) && !self.is_shadowed(name) {
            self.referenced_imports.insert(CompactString::new(name));
        }
    }

    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        self.type_depth += 1;
        walk_ts_type(self, it);
        self.type_depth -= 1;
    }
}

impl ScriptSetupComponentImportVisitor {
    fn is_shadowed(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }
}
