//! Lazy reads for default-bearing slot patterns; ordinary slots keep their path.

use oxc_ast::ast::{BindingPattern, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;
use vize_carton::{SmallVec, String, cstr};

use super::GenerateContext;
use crate::generate::destructure::DestructureBinding;

impl GenerateContext<'_> {
    pub(super) fn resolve_slot_defaults(&mut self, pattern: &str, root: &str) {
        if !expression_is_safe_to_parse(pattern) {
            return;
        }
        let source = cstr!("let {pattern} = __slotProps");
        let allocator = vize_atelier_core::expr_parse_probe::parse_arena();
        let parsed = Parser::new(&allocator, &source, SourceType::ts().with_module(true)).parse();
        if !parsed.diagnostics.is_empty() {
            return;
        }
        let Some(Statement::VariableDeclaration(declaration)) = parsed.program.body.first() else {
            return;
        };
        let Some(declarator) = declaration.declarations.first() else {
            return;
        };
        let mut pending = SmallVec::<[(&BindingPattern<'_>, String); 8]>::new();
        pending.push((&declarator.id, String::from(root)));
        while let Some((pattern, read)) = pending.pop() {
            match pattern {
                BindingPattern::BindingIdentifier(identifier) => {
                    if let Some(scope) = self.slot_scopes.last_mut() {
                        let local = String::from(identifier.name.as_str());
                        if !scope.names.contains(&local) {
                            scope.names.push(local.clone());
                        }
                        scope
                            .read_overrides
                            .push(DestructureBinding { local, path: read });
                    }
                }
                BindingPattern::AssignmentPattern(assignment) => {
                    let span = assignment.right.span();
                    let Some(default) = source.get(span.start as usize..span.end as usize) else {
                        continue;
                    };
                    let default = self.resolve_expression(default);
                    self.use_helper("getDefaultValue");
                    pending.push((
                        &assignment.left,
                        cstr!("_getDefaultValue({read}, () => ({default}))"),
                    ));
                }
                BindingPattern::ObjectPattern(object) => {
                    for property in object.properties.iter().rev() {
                        let segment = if !property.computed
                            && let PropertyKey::StaticIdentifier(identifier) = &property.key
                        {
                            cstr!(".{}", identifier.name)
                        } else {
                            let span = property.key.span();
                            let Some(key) = source.get(span.start as usize..span.end as usize)
                            else {
                                continue;
                            };
                            let key = if property.computed {
                                self.resolve_expression(key)
                            } else {
                                String::from(key)
                            };
                            cstr!("[{key}]")
                        };
                        pending.push((&property.value, cstr!("{read}{segment}")));
                    }
                }
                BindingPattern::ArrayPattern(array) => {
                    for (index, item) in array.elements.iter().enumerate().rev() {
                        if let Some(item) = item {
                            pending.push((item, cstr!("{read}[{index}]")));
                        }
                    }
                }
            }
        }
    }
}
