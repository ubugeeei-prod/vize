//! Class bodies: computed keys, members and static blocks.

use oxc_ast::ast::{Class, ClassElement};
use vize_carton::{FxHashMap, String, ToCompactString};

use super::{
    collect_from_expression, collect_from_function, collect_from_property_key,
    collect_from_statement,
};

pub(super) fn collect_from_class<'a>(
    class: &Class<'a>,
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    local_bindings: &FxHashMap<String, bool>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    let mut inner_bindings = local_bindings.clone();
    if let Some(id) = &class.id {
        inner_bindings.insert(id.name.to_compact_string(), true);
    }
    if let Some(super_class) = &class.super_class {
        collect_from_expression(super_class, source, local_to_key, &inner_bindings, rewrites);
    }
    for elem in class.body.body.iter() {
        match elem {
            ClassElement::MethodDefinition(method) => {
                if method.computed {
                    collect_from_property_key(
                        &method.key,
                        source,
                        local_to_key,
                        &inner_bindings,
                        rewrites,
                    );
                }
                collect_from_function(
                    &method.value,
                    source,
                    local_to_key,
                    &inner_bindings,
                    rewrites,
                );
            }
            ClassElement::PropertyDefinition(prop) => {
                if prop.computed {
                    collect_from_property_key(
                        &prop.key,
                        source,
                        local_to_key,
                        &inner_bindings,
                        rewrites,
                    );
                }
                if let Some(value) = &prop.value {
                    collect_from_expression(value, source, local_to_key, &inner_bindings, rewrites);
                }
            }
            ClassElement::AccessorProperty(prop) => {
                if prop.computed {
                    collect_from_property_key(
                        &prop.key,
                        source,
                        local_to_key,
                        &inner_bindings,
                        rewrites,
                    );
                }
                if let Some(value) = &prop.value {
                    collect_from_expression(value, source, local_to_key, &inner_bindings, rewrites);
                }
            }
            ClassElement::StaticBlock(block) => {
                let mut block_bindings = inner_bindings.clone();
                for stmt in block.body.iter() {
                    collect_from_statement(
                        stmt,
                        source,
                        local_to_key,
                        &mut block_bindings,
                        rewrites,
                    );
                }
            }
            ClassElement::TSIndexSignature(_) => {}
        }
    }
}
