//! Reject unresolved imports that determine the typed props object shape.

use oxc_allocator::Allocator;
use oxc_ast::ast::{TSTypeName, TSTypeReference};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString, cstr};

use super::ScriptCompileContext;
use crate::script::type_resolution::unresolved_property_shape_references;

#[derive(Default)]
struct References(Vec<String>);

impl<'a> Visit<'a> for References {
    fn visit_ts_type_reference(&mut self, reference: &TSTypeReference<'a>) {
        let mut name = &reference.type_name;
        while let TSTypeName::QualifiedName(qualified) = name {
            name = &qualified.left;
        }
        if let TSTypeName::IdentifierReference(identifier) = name {
            self.0.push(identifier.name.to_compact_string());
        }
        walk::walk_ts_type_reference(self, reference);
    }
}

impl ScriptCompileContext {
    pub(crate) fn validate_imported_macro_types(
        &self,
        imports: &[(String, String)],
    ) -> Result<(), String> {
        if imports.is_empty() {
            return Ok(());
        }
        let Some(type_args) = self
            .macros
            .define_props
            .as_ref()
            .and_then(|props| props.type_args.as_ref())
        else {
            return Ok(());
        };
        for reference in
            unresolved_property_shape_references(type_args, &self.interfaces, &self.type_aliases)
        {
            // Only an unresolved property-shape expression needs AST reference discovery.
            // A resolved object body and its value types never enter this fallback.
            let source = cstr!("type $Props = {reference};");
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
            let mut references = References::default();
            references.visit_program(&parsed.program);
            for name in references.0 {
                if let Some((_, specifier)) = imports.iter().find(|(local, _)| local == name) {
                    return Err(cstr!(
                        "Cannot resolve imported macro type `{name}` from `{specifier}`."
                    ));
                }
            }
        }
        Ok(())
    }
}
