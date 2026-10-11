//! Extract public property names without leaving a declaration's module scope.

mod helpers;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Statement, TSType};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, FxHashSet, cstr};

use super::{
    ResolvedTypeWorld, TypeDeclarationId, TypeDeclarationKind, TypeLookup, UnknownTypeReason,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopedTypeProperties {
    pub properties: Vec<ScopedTypeProperty>,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedTypeProperty {
    pub name: CompactString,
    pub prop_type: Option<CompactString>,
    pub optional: bool,
    /// Lexical module of the authored member type, not the importing SFC.
    pub module: CompactString,
}

impl ResolvedTypeWorld {
    pub fn resolve_properties(&self, type_args: &str) -> ScopedTypeProperties {
        self.properties_in(
            &self.root_module,
            type_args,
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        )
    }

    fn properties_in(
        &self,
        module: &str,
        expression: &str,
        bound: &FxHashSet<CompactString>,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeProperties {
        if visiting.len() >= 128 {
            return ScopedTypeProperties::default();
        }
        let source = cstr!("type __VizeProps = {expression}");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            return ScopedTypeProperties::default();
        }
        let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
            return ScopedTypeProperties::default();
        };
        self.properties_ast(module, &source, &alias.type_annotation, bound, visiting)
    }

    fn properties_ast(
        &self,
        module: &str,
        source: &str,
        ty: &TSType<'_>,
        bound: &FxHashSet<CompactString>,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeProperties {
        match ty {
            TSType::TSTypeLiteral(literal) => helpers::members(module, source, &literal.members),
            TSType::TSParenthesizedType(inner) => {
                self.properties_ast(module, source, &inner.type_annotation, bound, visiting)
            }
            TSType::TSIntersectionType(intersection) => {
                let mut output = ScopedTypeProperties {
                    complete: true,
                    ..ScopedTypeProperties::default()
                };
                for part in &intersection.types {
                    helpers::combine(
                        &mut output,
                        self.properties_ast(module, source, part, bound, visiting),
                        false,
                    );
                }
                output
            }
            TSType::TSUnionType(union) => {
                let mut output = ScopedTypeProperties {
                    complete: true,
                    ..ScopedTypeProperties::default()
                };
                for part in &union.types {
                    helpers::combine_union(
                        &mut output,
                        self.properties_ast(module, source, part, bound, visiting),
                    );
                }
                output
            }
            TSType::TSTypeReference(reference) => {
                let name = helpers::text(source, reference.type_name.span());
                if bound.contains(&name) {
                    return ScopedTypeProperties::default();
                }
                match self.resolve(module, &name) {
                    TypeLookup::Found(id) => self.declaration_properties(&id, visiting),
                    TypeLookup::Unknown(unknown)
                        if unknown.reason == UnknownTypeReason::Unbound =>
                    {
                        self.utility_properties(
                            module,
                            source,
                            &name,
                            reference.type_arguments.as_deref(),
                            bound,
                            visiting,
                        )
                    }
                    TypeLookup::Unknown(_) => ScopedTypeProperties::default(),
                }
            }
            TSType::TSImportType(import) => {
                let Some(qualifier) = &import.qualifier else {
                    return ScopedTypeProperties::default();
                };
                let name = helpers::text(source, qualifier.span());
                match self.resolve_import(module, &import.source.value, &name) {
                    TypeLookup::Found(id) => self.declaration_properties(&id, visiting),
                    TypeLookup::Unknown(_) => ScopedTypeProperties::default(),
                }
            }
            _ => ScopedTypeProperties::default(),
        }
    }

    fn declaration_properties(
        &self,
        id: &TypeDeclarationId,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeProperties {
        if visiting.len() >= 128 || !visiting.insert(id.clone()) {
            return ScopedTypeProperties::default();
        }
        let Some(declaration) = self.declaration(id) else {
            visiting.remove(id);
            return ScopedTypeProperties::default();
        };
        let bound = helpers::parameters(declaration.type_parameters.as_deref());
        let mut output = ScopedTypeProperties {
            complete: true,
            ..ScopedTypeProperties::default()
        };
        if declaration.kind == TypeDeclarationKind::Interface {
            for base in &declaration.extends {
                helpers::combine(
                    &mut output,
                    self.properties_in(&id.module, base, &bound, visiting),
                    false,
                );
            }
        }
        helpers::combine(
            &mut output,
            self.properties_in(&id.module, &declaration.body, &bound, visiting),
            declaration.kind == TypeDeclarationKind::Interface,
        );
        visiting.remove(id);
        output
    }

    fn utility_properties(
        &self,
        module: &str,
        source: &str,
        name: &str,
        arguments: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
        bound: &FxHashSet<CompactString>,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeProperties {
        if !matches!(name, "Partial" | "Required" | "Readonly" | "Pick" | "Omit") {
            return ScopedTypeProperties::default();
        }
        let Some(arguments) = arguments else {
            return ScopedTypeProperties::default();
        };
        let Some(base) = arguments.params.first() else {
            return ScopedTypeProperties::default();
        };
        let mut output = self.properties_ast(module, source, base, bound, visiting);
        match name {
            "Partial" => output
                .properties
                .iter_mut()
                .for_each(|prop| prop.optional = true),
            "Required" => output
                .properties
                .iter_mut()
                .for_each(|prop| prop.optional = false),
            "Pick" | "Omit" => {
                let Some(keys) = arguments.params.get(1).and_then(helpers::literal_keys) else {
                    return ScopedTypeProperties::default();
                };
                output
                    .properties
                    .retain(|prop| keys.contains(&prop.name) == (name == "Pick"));
            }
            _ => {}
        }
        output
    }
}

impl crate::types::TypeResolver {
    pub fn resolved_props_complete(&self) -> Option<bool> {
        self.resolved_props_complete
    }

    pub fn resolved_prop_module(&self, name: &str) -> Option<&str> {
        self.resolved_prop_modules
            .get(name)
            .map(CompactString::as_str)
    }

    pub fn record_resolved_properties(&mut self, properties: &ScopedTypeProperties) {
        self.resolved_props_complete = Some(properties.complete);
        self.resolved_prop_modules = properties
            .properties
            .iter()
            .map(|prop| (prop.name.clone(), prop.module.clone()))
            .collect();
    }
}
