//! Reachable authored type contracts, attached to each declaration fingerprint.

mod world;

use super::type_refs::{self, TypeRef};
use crate::Croquis;
use crate::macros::ModelDefinition;
use oxc_ast::ast::{Statement, TSType, TSTypeName};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vize_carton::{Allocator, CompactString, cstr};

/// Reachable authored type declarations, resolved in their declaring modules.
/// `complete` means this declaration closure is known; it does not assert
/// TypeScript assignability or infer a value's type from its implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeEnvironment {
    pub complete: bool,
    pub declarations: Vec<TypeDependency>,
}

/// One authored declaration or an explicitly unresolved external identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeDependency {
    pub name: CompactString,
    pub kind: CompactString,
    pub module: Option<CompactString>,
    pub export: Option<CompactString>,
    pub body: Option<CompactString>,
    pub parameters: Option<CompactString>,
    pub extends: Vec<CompactString>,
    pub unresolved_reason: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PendingTypeRef {
    reference: TypeRef,
    context: Option<CompactString>,
}

impl Croquis {
    pub(super) fn model_type_environment(
        &self,
        model: &ModelDefinition,
        generic: Option<&str>,
    ) -> TypeEnvironment {
        let sources = model
            .model_type
            .as_deref()
            .into_iter()
            .chain(self.macros.model_modifier_type(&model.name));
        let mut environment = self.collect_type_environment(sources, generic);
        environment.complete &= model.model_type.is_some();
        environment
    }
    pub(super) fn emit_type_environment<'s>(
        &self,
        name: &str,
        payloads: impl Iterator<Item = Option<&'s str>>,
        generic: Option<&str>,
    ) -> TypeEnvironment {
        let payloads: Vec<_> = payloads.collect();
        let known = payloads.iter().all(Option::is_some);
        let arguments = (!known)
            .then(|| {
                self.macros
                    .define_emits()
                    .and_then(|call| call.type_args.as_deref())
            })
            .flatten()
            .map(type_arguments_body);
        let validators: Vec<_> = self
            .macros
            .emit_validator_signatures(name)
            .iter()
            .map(|header| cstr!("{{ validator{header} }}"))
            .collect();
        let mut environment = self.collect_type_environment(
            payloads
                .iter()
                .filter_map(|payload| *payload)
                .chain(arguments)
                .chain(validators.iter().map(CompactString::as_str))
                .chain(
                    self.macros
                        .emit_validator_type_annotations(name)
                        .iter()
                        .map(CompactString::as_str)
                        .filter(|annotation| !is_const_assertion_marker(annotation)),
                ),
            generic,
        );
        environment.complete &= known;
        environment
    }

    pub(super) fn prop_type_environment(
        &self,
        source: Option<&str>,
        generic: Option<&str>,
    ) -> TypeEnvironment {
        if let Some(arguments) = self
            .macros
            .define_props()
            .and_then(|call| call.type_args.as_deref())
        {
            let arguments = type_arguments_body(arguments);
            if !type_refs::is_object_type(arguments) {
                // Compatibility prop expansion loses a field's declaring
                // module. Keep the complete scoped macro closure on each
                // expanded row until the producer supplies field origins.
                return self.type_environment(Some(arguments), generic);
            }
        }
        self.type_environment(source, generic)
    }

    pub(super) fn type_environment(
        &self,
        source: Option<&str>,
        generic: Option<&str>,
    ) -> TypeEnvironment {
        let mut environment = self.collect_type_environment(source.into_iter(), generic);
        environment.complete &= source.is_some();
        environment
    }

    pub(super) fn signature_type_environment(&self, generic: Option<&str>) -> TypeEnvironment {
        let sources = self
            .macros
            .exposes()
            .iter()
            .filter_map(|expose| expose.expose_type.as_deref())
            .chain(
                self.macros
                    .define_props()
                    .and_then(|call| call.type_args.as_deref())
                    .map(type_arguments_body),
            )
            .chain(
                self.macros
                    .define_emits()
                    .and_then(|call| call.type_args.as_deref())
                    .map(type_arguments_body),
            )
            .chain(
                self.macros
                    .define_slots()
                    .and_then(|call| call.type_args.as_deref())
                    .map(type_arguments_body),
            );
        self.collect_type_environment(sources, generic)
    }

    fn collect_type_environment<'s>(
        &self,
        sources: impl Iterator<Item = &'s str>,
        generic: Option<&str>,
    ) -> TypeEnvironment {
        let mut pending = BTreeSet::new();
        let mut complete = true;
        let context = self
            .types
            .resolved_world()
            .map(|world| world.root_module.clone());
        for source in sources.chain(Some("unknown")) {
            let references = type_refs::collect(source, None, generic);
            complete &= references.complete;
            pending.extend(references.refs.into_iter().map(|reference| PendingTypeRef {
                reference,
                context: context.clone(),
            }));
        }
        let definitions = self.types.definitions();
        let mut declarations = BTreeMap::new();
        let mut visited = BTreeSet::new();
        while let Some(item) = pending.pop_first() {
            if !visited.insert(item.clone()) {
                continue;
            }
            if let Some((dependency, references, context)) =
                self.world_type_dependency(&item, generic)
            {
                complete &= references.complete;
                pending.extend(references.refs.into_iter().map(|reference| PendingTypeRef {
                    reference,
                    context: context.clone(),
                }));
                declarations.insert(
                    (
                        dependency.module.clone(),
                        dependency.name.clone(),
                        dependency.export.clone(),
                    ),
                    dependency,
                );
                continue;
            }
            let reference = item.reference;
            let local = reference.module.is_none() && reference.root == reference.name;
            let body = local
                .then(|| definitions.resolve(&reference.root))
                .flatten();
            let imported = reference.module.as_deref().or_else(|| {
                definitions
                    .imported_types
                    .get(&reference.root)
                    .map(CompactString::as_str)
            });
            let (kind, module, export, parameters, extends) = if let Some(body) = body {
                let parameters = definitions.type_parameters(&reference.root);
                let extends = definitions.interface_extends(&reference.root);
                for source in Some(body.as_str())
                    .into_iter()
                    .chain(extends.iter().map(CompactString::as_str))
                {
                    let references = type_refs::collect(source, parameters, generic);
                    complete &= references.complete;
                    pending.extend(references.refs.into_iter().map(|reference| PendingTypeRef {
                        reference,
                        context: None,
                    }));
                }
                (
                    if definitions.interfaces.contains_key(&reference.root) {
                        "interface"
                    } else {
                        "alias"
                    },
                    None,
                    None,
                    parameters.map(CompactString::new),
                    extends,
                )
            } else if let Some(module) = imported {
                complete = false;
                let exported = if reference.module.is_some() {
                    reference.root.as_str()
                } else {
                    definitions
                        .imported_type_export(&reference.root)
                        .unwrap_or(reference.root.as_str())
                };
                (
                    "imported",
                    Some(CompactString::new(module)),
                    Some(CompactString::new(exported)),
                    None,
                    Vec::new(),
                )
            } else {
                complete = false;
                ("unresolved", None, None, None, Vec::new())
            };
            declarations.insert(
                (module.clone(), reference.name.clone(), export.clone()),
                TypeDependency {
                    name: reference.name,
                    kind: CompactString::new(kind),
                    module,
                    export,
                    body: body.cloned(),
                    parameters,
                    extends,
                    unresolved_reason: (body.is_none()).then(|| {
                        CompactString::new(if imported.is_some() {
                            "external-world-unavailable"
                        } else {
                            "unbound"
                        })
                    }),
                },
            );
        }
        TypeEnvironment {
            complete,
            declarations: declarations.into_values().collect(),
        }
    }
}

fn type_arguments_body(arguments: &str) -> &str {
    // MacroCall records the parser's complete `<...>` span. Remove only
    // these known outer delimiters; nested literal/type text is unchanged.
    arguments
        .strip_prefix('<')
        .and_then(|body| body.strip_suffix('>'))
        .unwrap_or(arguments)
}

fn is_const_assertion_marker(annotation: &str) -> bool {
    let allocator = Allocator::new();
    let source = cstr!("type $$ = {annotation};");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    // The producer preserves the exact authored marker in the public contract.
    // A bare const assertion is literal preservation, not a lexical type name.
    !parsed.panicked
        && parsed.diagnostics.is_empty()
        && matches!(parsed.program.body.as_slice(),
            [Statement::TSTypeAliasDeclaration(alias)]
                if matches!(&alias.type_annotation, TSType::TSTypeReference(reference)
                    if reference.type_arguments.is_none()
                        && matches!(&reference.type_name, TSTypeName::IdentifierReference(name)
                            if name.name == "const")))
}
