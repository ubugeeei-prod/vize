//! Reject unresolved imports reached by a typed props declaration.

use oxc_allocator::Allocator;
use oxc_ast::ast::{TSTypeName, TSTypeReference};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::ScriptCompileContext;

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
        let mut pending = vec![cstr!("type $Props = {type_args};")];
        let mut visited = FxHashSet::default();
        while let Some(source) = pending.pop() {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
            let mut references = References::default();
            references.visit_program(&parsed.program);
            for name in references.0 {
                if !visited.insert(name.clone()) {
                    continue;
                }
                if let Some(body) = self.interfaces.get(&name) {
                    // The context represents interface heritage as an
                    // intersection type, using the same form as its resolver.
                    pending.push(cstr!("type $Type = {body};"));
                } else if let Some(body) = self.type_aliases.get(&name) {
                    pending.push(cstr!("type $Type = {body};"));
                } else if let Some((_, specifier)) = imports.iter().find(|(local, _)| local == name)
                {
                    return Err(cstr!(
                        "Cannot resolve imported macro type `{name}` from `{specifier}`."
                    ));
                }
            }
        }
        Ok(())
    }
}
